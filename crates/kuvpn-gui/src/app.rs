use futures::SinkExt;
use iced::{Subscription, Task};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tray_icon::{menu::MenuEvent, TrayIcon, TrayIconEvent};

use crate::tray::TrayMenuItems;

use crate::config::GuiSettings;
use crate::provider::{GuiInteraction, GuiProvider};
use crate::types::{
    log_level_from_slider, login_mode_flags, ConnectionStatus, InputRequest, InputRequestWrapper,
    Message, Tab,
};
use kuvpn::{ErrorCategory, SessionConfig, VpnSession};
use std::time::Duration;

/// Returns the window settings used for every window open call.
pub(super) fn window_settings(use_csd: bool) -> iced::window::Settings {
    // The application_id must match the basename of our .desktop file
    // (kuvpn.desktop) so GNOME/Ubuntu can associate the running window with
    // the desktop entry — otherwise the taskbar shows a generic gear icon.
    #[cfg(target_os = "linux")]
    let platform_specific = iced::window::settings::PlatformSpecific {
        application_id: "kuvpn".to_string(),
        ..Default::default()
    };

    iced::window::Settings {
        exit_on_close_request: false,
        size: iced::Size::new(580.0, 650.0),
        min_size: Some(iced::Size::new(560.0, 580.0)),
        max_size: Some(iced::Size::new(580.0, 650.0)),
        resizable: false,
        decorations: !use_csd,
        transparent: use_csd,
        icon: crate::load_window_icon(),
        #[cfg(target_os = "linux")]
        platform_specific,
        ..Default::default()
    }
}

pub struct KuVpnGui {
    // Settings
    pub settings: GuiSettings,

    // UI State
    pub current_tab: Tab,
    pub logs: Vec<String>,
    pub status: ConnectionStatus,
    pub pending_request: Option<InputRequest>,
    pub pending_email: Option<String>,
    pub current_input: String,
    pub show_password_held: bool,
    pub mfa_info: Option<String>,
    pub status_message: String,
    pub error_message: Option<String>,
    pub error_category: Option<ErrorCategory>,
    pub rotation: f32,
    pub oc_test_result: Option<bool>,
    pub vpnc_script_test_result: Option<bool>,
    pub automation_warning: Option<String>,
    /// Set when the Test button resolves a different path than what was entered.
    pub oc_path_notification: Option<String>,
    /// True after the first (startup) auto-test completes; used to suppress
    /// the replacement notification for the initial auto-detection pass.
    pub oc_startup_tested: bool,
    /// Result of the last Wipe Session action: Some(true) = success, Some(false) = failed.
    pub session_wipe_result: Option<bool>,
    /// True immediately after Reset Defaults is pressed; shows confirmation in the view.
    pub reset_notification: bool,
    /// Drives the fade animation for action notifications (1.0 = fully visible → 0.0 = gone).
    pub notif_fade: f32,

    // Connection history
    pub history: Vec<kuvpn::ConnectionEvent>,

    // VPN Session
    pub session: Option<Arc<VpnSession>>,

    // Tray & Window state
    pub tray_icon: Option<TrayIcon>,
    pub tray_menu: Option<TrayMenuItems>,
    pub window_id: Option<iced::window::Id>,
    pub is_visible: bool,
    pub is_minimized: bool,
    pub window_close_pending: bool,
    pub window_open_pending: bool,
    pub last_tray_click: Option<std::time::Instant>,
    pub connection_start: Option<Instant>,
    /// Detected name of the active VPN interface (e.g. "kuvpn0" on Linux, "utun3" on macOS).
    /// `None` when not connected or on platforms where it cannot be determined (Windows).
    pub active_interface: Option<String>,
    /// Privilege escalation tools found on this system (e.g. ["pkexec", "sudo"]).
    /// Empty on Windows. Empty on Unix means no tool is installed — VPN cannot start.
    #[cfg_attr(windows, allow(dead_code))]
    pub available_escalation_tools: Vec<&'static str>,
    /// True when the window was auto-shown from a hidden state for a prompt.
    /// Used to auto-hide the window again after the prompt resolves.
    pub was_shown_for_prompt: bool,
    /// Path to the most recently saved automation diagnostic bundle, if any.
    /// Shown as an "Open folder" button inside the automation warning card.
    pub last_diagnostic_path: Option<String>,
    /// Whether the console log should snap to the bottom on new entries.
    /// Disabled when the user manually scrolls up; re-enabled when they
    /// scroll back to within ~1 % of the bottom.
    pub console_auto_scroll: bool,
    /// Persistent rotating log file for post-mortem debugging.
    pub log_file: Option<kuvpn::FileLogger>,
}

impl KuVpnGui {
    pub fn theme(&self, _id: iced::window::Id) -> iced::Theme {
        if self.settings.theme.dark {
            iced::Theme::Dark
        } else {
            iced::Theme::Light
        }
    }

    /// Resolve the current theme settings into a ready-to-use `Styler`.
    pub fn styler(&self) -> crate::styles::Styler {
        let app_theme = self.settings.theme.resolve();
        crate::styles::Styler {
            p: app_theme.palette,
            rounding: app_theme.rounding,
            shadow: app_theme.shadow,
        }
    }

    fn save_settings(&self) {
        if let Err(e) = self.settings.save() {
            log::error!("Failed to save settings: {}", e);
        }
    }

    fn is_transitioning(&self) -> bool {
        matches!(
            self.status,
            ConnectionStatus::Connecting | ConnectionStatus::Disconnecting
        )
    }

    /// Shows the window if hidden, or brings it to the foreground if already visible.
    /// Call this whenever the app needs the user's attention (MFA prompt, input request, etc.)
    fn show_or_focus_window(&mut self) -> Task<Message> {
        if !self.is_visible && !self.window_close_pending && !self.window_open_pending {
            self.update(Message::ToggleVisibility {
                from_close_request: false,
            })
        } else if self.is_visible {
            if let Some(id) = self.window_id {
                // Unminimize first (no-op if already visible), then focus.
                return Task::batch(vec![
                    iced::window::minimize(id, false),
                    iced::window::gain_focus(id),
                ]);
            }
            Task::none()
        } else {
            Task::none()
        }
    }

    fn maybe_auto_hide_task(&mut self) -> Task<Message> {
        if self.was_shown_for_prompt && self.settings.auto_hide_after_prompt {
            self.was_shown_for_prompt = false;
            return Task::perform(
                async { tokio::time::sleep(Duration::from_millis(400)).await },
                |_| Message::AutoHideWindow,
            );
        }
        Task::none()
    }

    fn restart_window_task(&mut self, delay_ms: u64) -> Task<Message> {
        if let Some(id) = self.window_id {
            self.is_visible = false;
            self.window_close_pending = true;
            return Task::batch(vec![
                iced::window::close(id),
                Task::perform(
                    async move { tokio::time::sleep(Duration::from_millis(delay_ms)).await },
                    |_| Message::ToggleVisibility {
                        from_close_request: false,
                    },
                ),
            ]);
        }
        Task::none()
    }

    fn build_connection_stream(
        &self,
        session: Arc<VpnSession>,
        log_rx: crossbeam_channel::Receiver<String>,
        mut gui_rx: tokio::sync::mpsc::Receiver<String>,
    ) -> Task<Message> {
        Task::stream(iced::stream::channel(
            100,
            move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
                let (interaction_tx, mut interaction_rx) = tokio::sync::mpsc::channel(10);

                let provider = Arc::new(GuiProvider {
                    interaction_tx,
                    cancel_token: kuvpn::utils::CancellationToken::new(),
                    page_guard: std::sync::Mutex::new(None),
                });

                let session_c = Arc::clone(&session);
                let _join_handle = session.connect(provider);

                loop {
                    // Poll logs from session
                    while let Ok(log_msg) = log_rx.try_recv() {
                        let _ = output.send(Message::LogAppended(log_msg)).await;
                    }

                    // Poll logs from global logger
                    while let Ok(log_msg) = gui_rx.try_recv() {
                        let _ = output.send(Message::LogAppended(log_msg)).await;
                    }

                    // Poll status
                    let current_status = session_c.status();
                    let _ = output.send(Message::StatusChanged(current_status)).await;

                    if session_c.is_finished() {
                        break;
                    }

                    // Poll interactions
                    match interaction_rx.try_recv() {
                        Ok(GuiInteraction::Request(req)) => {
                            let _ = output
                                .send(Message::RequestInput(Arc::new(InputRequestWrapper(
                                    Mutex::new(Some(req)),
                                ))))
                                .await;
                        }
                        Ok(GuiInteraction::MfaPush(code)) => {
                            let _ = output.send(Message::MfaPushReceived(code)).await;
                        }
                        Ok(GuiInteraction::MfaComplete) => {
                            let _ = output.send(Message::MfaCompleteReceived).await;
                        }
                        Ok(GuiInteraction::DismissPrompt) => {
                            let _ = output.send(Message::DismissPrompt).await;
                        }
                        Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
                        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {}
                    }

                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
                let _ = output
                    .send(Message::ConnectionFinished(
                        session_c.last_error(),
                        session_c.error_category(),
                    ))
                    .await;
            },
        ))
    }

    // ── Private message handlers ──────────────────────────────────────────────

    /// Update tray menu item states to match `status`. Call this any time
    /// `self.status` is changed directly (not via `handle_status_changed`).
    fn sync_tray_menu_items(&self, status: ConnectionStatus) {
        let is_idle = matches!(
            status,
            ConnectionStatus::Disconnected | ConnectionStatus::Error
        );
        if let Some(menu) = &self.tray_menu {
            menu.connect.set_enabled(is_idle);
            menu.disconnect
                .set_enabled(status == ConnectionStatus::Connected);
            menu.wipe.set_enabled(is_idle);
            menu.status.set_text(crate::tray::status_label(status));
        }
    }

    fn handle_connect_pressed(&mut self) -> Task<Message> {
        if self.status != ConnectionStatus::Disconnected && self.status != ConnectionStatus::Error {
            return Task::none();
        }

        // Warn if a stale openconnect process is already running before we start a new session.
        if kuvpn::is_openconnect_running() && self.status == ConnectionStatus::Disconnected {
            log::warn!(
                "An OpenConnect process is already running. \
                 It will be monitored or replaced when the new session starts."
            );
        }

        // Manual mode requires a tested, valid vpnc-script path before connecting.
        if self.settings.is_manual_mode() {
            let empty = self.settings.vpnc_script.trim().is_empty();
            let invalid = self.vpnc_script_test_result == Some(false);
            let untested = self.vpnc_script_test_result.is_none();
            if empty || invalid || untested {
                self.error_message = Some(if empty {
                    "Manual mode requires a vpnc-script path. Enter a path and click Test."
                        .to_string()
                } else if invalid {
                    "The vpnc-script path is invalid or the file does not exist. Fix it and click Test.".to_string()
                } else {
                    "Please click Test to verify your vpnc-script path before connecting."
                        .to_string()
                });
                self.status = ConnectionStatus::Error;
                return Task::none();
            }
        }

        self.automation_warning = None;
        self.last_diagnostic_path = None;
        self.error_message = None;
        self.error_category = None;
        self.status_message = "Initializing...".to_string();
        self.connection_start = None;
        self.status = ConnectionStatus::Connecting;
        if let Some(tray) = &self.tray_icon {
            crate::tray::update_tray_icon(tray, self.status);
        }
        self.sync_tray_menu_items(ConnectionStatus::Connecting);

        let (headless, no_auto_login) = login_mode_flags(self.settings.login_mode_val);
        let config = SessionConfig {
            url: self.settings.url.clone(),
            domain: self.settings.domain.clone(),
            user_agent: "Mozilla/5.0".to_string(),
            headless,
            no_auto_login,
            email: if self.settings.email.is_empty() {
                None
            } else {
                Some(self.settings.email.clone())
            },
            openconnect_path: if self.settings.openconnect_path.is_empty() {
                "openconnect".to_string()
            } else {
                self.settings.openconnect_path.clone()
            },
            escalation_tool: Some(self.settings.escalation_tool.clone()),
            interface_name: "kuvpn0".to_string(),
            tunnel_mode: if self.settings.is_manual_mode() {
                kuvpn::TunnelMode::Manual(if self.settings.vpnc_script.is_empty() {
                    None
                } else {
                    Some(self.settings.vpnc_script.clone())
                })
            } else {
                kuvpn::TunnelMode::Full
            },
        };

        let session = Arc::new(VpnSession::new(config));
        self.session = Some(Arc::clone(&session));

        let (log_tx, log_rx) = crossbeam_channel::unbounded();
        session.set_logs_tx(log_tx);

        crate::logger::LOGGER_INIT.call_once(|| {
            let _ = log::set_logger(&crate::logger::GUI_LOGGER);
            log::set_max_level(log::LevelFilter::Trace);
        });

        let (gui_tx, gui_rx) = tokio::sync::mpsc::channel(100);
        crate::logger::GUI_LOGGER.set_tx(gui_tx);

        self.build_connection_stream(session, log_rx, gui_rx)
    }

    fn handle_connection_finished(
        &mut self,
        err: Option<String>,
        category: Option<kuvpn::ErrorCategory>,
    ) -> Task<Message> {
        self.status = if err.is_some() {
            ConnectionStatus::Error
        } else {
            ConnectionStatus::Disconnected
        };
        self.mfa_info = None;
        self.connection_start = None;
        self.active_interface = None;
        // Discard any staged email — it was never confirmed by a successful connection.
        self.pending_email = None;

        if let Some(tray) = &self.tray_icon {
            crate::tray::update_tray_icon(tray, self.status);
        }
        self.sync_tray_menu_items(self.status);

        if let Some(e) = err {
            self.error_category = category;

            // Auto-recover from a stale DSID caused by a previous force-quit.
            // OpenConnect exiting immediately before the tunnel is established
            // often means the server still holds an active session and rejected
            // the new cookie. Wipe the cached browser session data and retry
            // once — the guard is that has_session_data() returns false after
            // a successful wipe, so a second failure won't loop.
            if matches!(category, Some(kuvpn::ErrorCategory::Connection))
                && e.contains("OpenConnect process exited before tunnel was established")
                && kuvpn::has_session_data()
                && kuvpn::wipe_user_data_dir().is_ok()
            {
                self.logs.push(
                    "[INF] Stale session detected — session data cleared. Retrying...".to_string(),
                );
                self.status = ConnectionStatus::Disconnected;
                self.sync_tray_menu_items(self.status);
                if let Some(tray) = &self.tray_icon {
                    crate::tray::update_tray_icon(tray, self.status);
                }
                let history_task = Task::perform(
                    async { kuvpn::load_events().unwrap_or_default() },
                    Message::HistoryLoaded,
                );
                return Task::batch(vec![history_task, Task::done(Message::AutoRetryConnect)]);
            }

            let is_automation_failure =
                matches!(category, Some(kuvpn::ErrorCategory::Authentication))
                    && (e.contains("Full Auto mode unable to complete login")
                        || e.contains("Could not find a handler"));

            if is_automation_failure {
                self.automation_warning = Some(
                    "Full Auto mode was unable to complete the login flow.\n\n\
                     What to do:\n\
                     • Switch to Manual mode and complete login yourself\n\
                     • Try clearing session data (Wipe Session button)\n\
                     • Use Visual Auto mode to record a video for bug reporting\n\n\
                     Check console/logs for technical details."
                        .to_string(),
                );
                self.error_message = None;
            } else {
                self.error_message = Some(e);
            }
        }

        // Reload history — the session thread just appended a disconnect/error
        // event to disk, so refresh the in-memory list to reflect it.
        Task::perform(
            async { kuvpn::load_events().unwrap_or_default() },
            Message::HistoryLoaded,
        )
    }

    fn handle_status_changed(&mut self, status: ConnectionStatus) -> Task<Message> {
        if self.status == status {
            return Task::none();
        }
        self.status = status;

        self.sync_tray_menu_items(status);

        if let Some(tray) = &self.tray_icon {
            crate::tray::update_tray_icon(tray, status);
        }

        if status == ConnectionStatus::Connected {
            self.connection_start = Some(Instant::now());
            // Connection succeeded — persist the staged email if one was captured.
            if let Some(email) = self.pending_email.take() {
                self.settings.email = email;
                self.save_settings();
            }
            #[cfg(unix)]
            {
                self.active_interface = kuvpn::get_vpn_interface_name("kuvpn0");
            }
            // Reload history — the session thread just appended a connect event
            // to disk; pull it into the in-memory list immediately.
            return Task::perform(
                async { kuvpn::load_events().unwrap_or_default() },
                Message::HistoryLoaded,
            );
        } else if matches!(
            status,
            ConnectionStatus::Disconnected | ConnectionStatus::Error
        ) {
            self.active_interface = None;
        }

        Task::none()
    }

    // ── Iced update ───────────────────────────────────────────────────────────

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowOpened(id) => {
                log::info!("Window opened with ID: {:?}", id);
                self.window_id = Some(id);
                self.is_visible = true;
                self.is_minimized = false;
                self.window_close_pending = false;
                self.window_open_pending = false;
                // Explicitly focus the new window. winit's focus_window() is gated on
                // is_visible; calling it here (after the window is created) ensures the
                // app is brought to front on every platform, including macOS accessory
                // processes where the window may open without stealing focus.
                iced::window::gain_focus(id)
            }
            Message::WindowClosed(id) => {
                log::info!("Window closed with ID: {:?}", id);
                if self.window_id == Some(id) {
                    self.window_id = None;
                    self.is_visible = false;
                    self.is_minimized = false;
                    self.window_close_pending = false;
                }
                Task::none()
            }
            Message::WindowFocused => {
                self.is_minimized = false;
                Task::none()
            }
            Message::ResetClosePending => {
                log::info!("Resetting window_close_pending (safety timeout)");
                self.window_close_pending = false;
                Task::none()
            }
            #[cfg(target_os = "linux")]
            Message::GtkTick => {
                while gtk::events_pending() {
                    gtk::main_iteration();
                }
                Task::none()
            }
            Message::TrayEvent(_event) => Task::none(),
            Message::MenuEvent(event) => match event.id.as_ref() {
                "quit" => self.update(Message::QuitRequested),
                "show" => {
                    let now = std::time::Instant::now();
                    if let Some(last) = self.last_tray_click {
                        if now.duration_since(last) < std::time::Duration::from_millis(500) {
                            return Task::none();
                        }
                    }
                    self.last_tray_click = Some(now);
                    log::info!("Menu 'show' clicked, toggling visibility");
                    self.update(Message::ToggleVisibility {
                        from_close_request: false,
                    })
                }
                "connect" => self.update(Message::ConnectPressed),
                "disconnect" => self.update(Message::DisconnectPressed),
                "wipe" => self.update(Message::ClearSessionPressed),
                "copy_logs" => self.update(Message::CopyLogs),
                "settings" => {
                    self.current_tab = Tab::Settings;
                    self.show_or_focus_window()
                }
                _ => Task::none(),
            },
            Message::CloseToTrayToggled(v) => {
                self.settings.close_to_tray = v;
                self.save_settings();
                Task::none()
            }
            Message::AutoHideAfterPromptToggled(v) => {
                self.settings.auto_hide_after_prompt = v;
                self.save_settings();
                Task::none()
            }
            Message::TunnelModeChanged(v) => {
                self.settings.tunnel_mode_val = v;
                self.save_settings();
                Task::none()
            }
            Message::VpncScriptChanged(v) => {
                self.settings.vpnc_script = v;
                self.vpnc_script_test_result = None;
                self.save_settings();
                Task::none()
            }
            Message::AdvancedModeToggled(v) => {
                self.settings.advanced_mode = v;
                self.save_settings();
                Task::none()
            }
            Message::AutoHideWindow => {
                if self.is_visible {
                    return self.update(Message::ToggleVisibility {
                        from_close_request: false,
                    });
                }
                Task::none()
            }
            Message::ClientDecorationsToggled(v) => {
                self.settings.use_client_decorations = v;
                self.save_settings();
                // Apply decoration change by closing and reopening window
                self.restart_window_task(100)
            }
            Message::ToggleVisibility { from_close_request } => {
                log::info!(
                    "ToggleVisibility called. visible={}, minimized={}, close_to_tray={}, from_close_request={}, close_pending={}, open_pending={}",
                    self.is_visible,
                    self.is_minimized,
                    self.settings.close_to_tray,
                    from_close_request,
                    self.window_close_pending,
                    self.window_open_pending
                );

                // Ignore toggles while a close or open is in-flight
                if self.window_close_pending || self.window_open_pending {
                    log::info!("Ignoring toggle - operation in flight");
                    return Task::none();
                }

                // If the window is minimized, restore it instead of hiding
                if self.is_visible && self.is_minimized && !from_close_request {
                    log::info!("Window is minimized — restoring instead of hiding");
                    self.is_minimized = false;
                    if let Some(id) = self.window_id {
                        return Task::batch(vec![
                            iced::window::minimize(id, false),
                            iced::window::gain_focus(id),
                        ]);
                    }
                    return Task::none();
                }

                if self.is_visible {
                    if from_close_request && !self.settings.close_to_tray {
                        log::info!("Exiting application due to close request");
                        return self.update(Message::QuitRequested);
                    }
                    log::info!("Closing window to hide");
                    self.is_visible = false;
                    self.window_close_pending = true;
                    if let Some(id) = self.window_id {
                        return Task::batch(vec![
                            iced::window::close(id),
                            Task::perform(
                                async {
                                    tokio::time::sleep(std::time::Duration::from_millis(500)).await
                                },
                                |_| Message::ResetClosePending,
                            ),
                        ]);
                    }
                } else {
                    log::info!("Opening window to show");
                    // Set flags to prevent operations during open
                    self.is_visible = true;
                    self.window_open_pending = true;
                    let use_csd = self.settings.use_client_decorations;
                    let (id, task) = iced::window::open(window_settings(use_csd));
                    self.window_id = Some(id);
                    return task.map(Message::WindowOpened);
                }
                Task::none()
            }
            Message::UrlChanged(url) => {
                self.settings.url = url;
                self.save_settings();
                Task::none()
            }
            Message::DomainChanged(domain) => {
                self.settings.domain = domain;
                self.save_settings();
                Task::none()
            }
            #[cfg(not(windows))]
            Message::EscalationToolChanged(tool) => {
                self.settings.escalation_tool = tool;
                self.save_settings();
                Task::none()
            }
            Message::LogLevelSliderChanged(val) => {
                self.settings.log_level_val = val;
                crate::logger::GUI_LOGGER.set_level(log_level_from_slider(val));
                self.save_settings();
                Task::none()
            }
            Message::OpenConnectPathChanged(p) => {
                self.settings.openconnect_path = p;
                self.oc_test_result = None;
                self.oc_path_notification = None;
                self.save_settings();
                Task::none()
            }
            Message::EmailChanged(e) => {
                self.settings.email = e;
                self.save_settings();
                Task::none()
            }
            Message::LoginModeChanged(val) => {
                self.settings.login_mode_val = val;
                self.save_settings();
                Task::none()
            }
            Message::Tick => {
                if self.is_transitioning() {
                    self.rotation += 0.1;
                }
                // Resolve the interface name as soon as possible after connecting.
                // Tick fires every 16 ms, so this catches the name far sooner than
                // the 100 ms session-poll loop that drives StatusChanged.
                #[cfg(unix)]
                if self.status == ConnectionStatus::Connected && self.active_interface.is_none() {
                    self.active_interface = kuvpn::get_vpn_interface_name("kuvpn0");
                }
                Task::none()
            }
            Message::ConnectPressed => {
                self.logs.clear();
                self.handle_connect_pressed()
            }
            Message::AutoRetryConnect => {
                // Auto-retry (e.g. stale session cleared): preserve logs and add a separator
                // banner so the user can see what happened before the retry started.
                self.logs
                    .push("──────────── auto-retry: stale session cleared ────────────".to_string());
                self.handle_connect_pressed()
            }
            Message::DisconnectPressed => {
                if let Some(session) = &self.session {
                    session.cancel();
                }
                Task::none()
            }
            Message::LogAppended(raw_log) => {
                if let Some(path) = raw_log.strip_prefix("Diagnostic|") {
                    self.last_diagnostic_path = Some(path.to_string());
                    return Task::none();
                }

                if let Some(parsed) = kuvpn::ParsedLog::parse(&raw_log) {
                    // Update status message for info/warn (errors come via ConnectionFinished).
                    if matches!(parsed.level, log::Level::Info | log::Level::Warn) {
                        self.status_message = parsed.message.clone();
                    }

                    let user_filter = crate::logger::GUI_LOGGER.get_level();

                    if parsed.level <= user_filter {
                        self.logs
                            .push(format!("[{}] {}", parsed.prefix(), parsed.message));
                        if let Some(ref mut f) = self.log_file {
                            f.write_line(&raw_log);
                        }
                        if self.logs.len() > 5000 {
                            self.logs.remove(0);
                        }
                    }
                }
                if self.console_auto_scroll {
                    iced::widget::operation::snap_to(
                        crate::view::console::CONSOLE_SCROLL_ID.clone(),
                        iced::widget::operation::RelativeOffset::END,
                    )
                } else {
                    Task::none()
                }
            }

            Message::ConsoleScrolled(offset) => {
                // Re-engage auto-scroll when the user scrolls back within ~1% of the bottom.
                self.console_auto_scroll = offset.y >= 0.99;
                Task::none()
            }

            Message::MfaPushReceived(code) => {
                self.mfa_info = Some(code.clone());
                self.current_tab = Tab::Connection;
                log::info!("MFA received - bringing window to front");
                if !self.is_visible && !self.window_close_pending && !self.window_open_pending {
                    self.was_shown_for_prompt = true;
                }
                self.show_or_focus_window()
            }
            Message::MfaCompleteReceived => {
                self.mfa_info = None;
                self.maybe_auto_hide_task()
            }
            Message::RequestInput(wrapper) => {
                if let Ok(mut guard) = wrapper.0.lock() {
                    if let Some(req) = guard.take() {
                        self.pending_request = Some(req);
                        self.current_input = String::new();
                        log::info!("Input requested - bringing window to front");
                        if !self.is_visible
                            && !self.window_close_pending
                            && !self.window_open_pending
                        {
                            self.was_shown_for_prompt = true;
                        }
                        return self.show_or_focus_window();
                    }
                }
                Task::none()
            }
            Message::InputChanged(val) => {
                self.current_input = val;
                Task::none()
            }
            Message::SubmitInput => {
                if let Some(req) = self.pending_request.take() {
                    // Stage the email for saving — it will be persisted to
                    // settings only if the connection succeeds (Connected status).
                    if req.is_email && self.settings.email.is_empty() {
                        self.pending_email = Some(self.current_input.clone());
                    }
                    let _ = req.response_tx.send(self.current_input.clone());
                    self.current_input = String::new();
                    self.show_password_held = false;
                }
                self.maybe_auto_hide_task()
            }
            Message::DismissPrompt => {
                // Page changed while a prompt was visible — retract it
                self.pending_request = None;
                self.current_input = String::new();
                self.show_password_held = false;
                self.maybe_auto_hide_task()
            }
            Message::ShowPasswordHeld(held) => {
                self.show_password_held = held;
                Task::none()
            }
            Message::ClearSessionPressed => {
                self.reset_notification = false;
                if !kuvpn::has_session_data() {
                    self.logs.push("No active session found.".to_string());
                    self.session_wipe_result = Some(true);
                } else {
                    match kuvpn::wipe_user_data_dir() {
                        Ok(()) => {
                            self.logs.push("Saved session data wiped.".to_string());
                            self.session_wipe_result = Some(true);
                        }
                        Err(e) => {
                            self.logs.push(format!("Failed to clear session: {}", e));
                            self.session_wipe_result = Some(false);
                        }
                    }
                }
                self.notif_fade = 1.0;
                Task::none()
            }
            Message::ConnectionFinished(err, category) => {
                self.handle_connection_finished(err, category)
            }
            Message::StatusChanged(status) => self.handle_status_changed(status),
            Message::ResetSettings => {
                let old_use_csd = self.settings.use_client_decorations;
                self.settings = GuiSettings::default();
                // Auto-select first available tool if the default (pkexec) isn't installed
                #[cfg(unix)]
                if !self.available_escalation_tools.is_empty()
                    && !self
                        .available_escalation_tools
                        .contains(&self.settings.escalation_tool.as_str())
                {
                    self.settings.escalation_tool = self.available_escalation_tools[0].to_string();
                }
                self.save_settings();
                self.oc_test_result = None;
                self.oc_path_notification = None;
                self.oc_startup_tested = false; // treat the next test as a fresh startup
                self.session_wipe_result = None;
                self.reset_notification = true;
                self.notif_fade = 1.0;

                // If window decoration setting changed, refresh window
                if old_use_csd != self.settings.use_client_decorations {
                    return self.restart_window_task(100);
                }

                // Also resolve the openconnect path with the reset defaults
                let path = self.settings.openconnect_path.clone();
                Task::perform(
                    async move {
                        kuvpn::locate_openconnect(&path).map(|p| p.to_string_lossy().into_owned())
                    },
                    Message::OpenConnectTestResult,
                )
            }
            Message::TestOpenConnect => {
                let path = self.settings.openconnect_path.clone();
                Task::perform(
                    async move {
                        kuvpn::locate_openconnect(&path).map(|p| p.to_string_lossy().into_owned())
                    },
                    Message::OpenConnectTestResult,
                )
            }
            Message::OpenConnectTestResult(resolved) => {
                let old_path = self.settings.openconnect_path.trim().to_string();
                self.oc_test_result = Some(resolved.is_some());
                if let Some(new_path) = resolved {
                    // Show a notification when the user explicitly tested a path and
                    // it was replaced (skip on the silent startup auto-test).
                    if self.oc_startup_tested
                        && new_path.trim().to_lowercase() != old_path.to_lowercase()
                    {
                        self.oc_path_notification = Some(format!(
                            "'{}' was not found or invalid — auto-resolved to: {}",
                            old_path, new_path
                        ));
                    } else {
                        self.oc_path_notification = None;
                    }
                    self.settings.openconnect_path = new_path;
                    self.save_settings();
                } else {
                    self.oc_path_notification = None;
                }
                self.oc_startup_tested = true;
                Task::none()
            }
            Message::TestVpncScript => {
                let path = self.settings.vpnc_script.trim().to_string();
                Task::perform(
                    async move {
                        let p = std::path::Path::new(&path);
                        if !path.is_empty() && p.is_file() {
                            Some(path)
                        } else {
                            None
                        }
                    },
                    Message::VpncScriptTestResult,
                )
            }
            Message::VpncScriptTestResult(result) => {
                self.vpnc_script_test_result = Some(result.is_some());
                Task::none()
            }
            Message::HistoryLoaded(events) => {
                self.history = events;
                Task::none()
            }
            Message::ClearHistory => {
                let _ = kuvpn::clear_events();
                self.history.clear();
                Task::none()
            }
            Message::ActionNotifTick => {
                // Decrement fade: 1.0 → 0.0 over 3 seconds (60 ticks × 50ms)
                self.notif_fade -= 1.0 / 60.0;
                if self.notif_fade <= 0.0 {
                    self.notif_fade = 0.0;
                    self.session_wipe_result = None;
                    self.reset_notification = false;
                }
                Task::none()
            }
            Message::ThemeFamilyChanged(family) => {
                self.settings.theme.family = family;
                self.save_settings();
                Task::none()
            }
            Message::ThemeToneChanged(dark) => {
                self.settings.theme.dark = dark;
                self.save_settings();
                Task::none()
            }
            Message::ThemeRoundingChanged(rounding) => {
                self.settings.theme.rounding = Some(rounding);
                self.save_settings();
                Task::none()
            }
            Message::ThemeShadowChanged(shadow) => {
                self.settings.theme.shadow = Some(shadow);
                self.save_settings();
                Task::none()
            }
            Message::OpenDiagnosticsFolder => {
                if let Some(ref path_str) = self.last_diagnostic_path {
                    let parent = std::path::Path::new(path_str)
                        .parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| std::path::PathBuf::from(path_str));
                    let _ = open::that(parent);
                }
                Task::none()
            }
            Message::CopyLogs => {
                let logs_text = self.logs.join("\n");
                Task::perform(
                    async move {
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            let _ = clipboard.set_text(logs_text);
                            // Keep clipboard alive for 500ms to allow clipboard managers to access it
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        }
                    },
                    |_| Message::Tick,
                )
            }
            Message::TabChanged(tab) => {
                self.current_tab = tab;
                // Hot-reload history whenever the tab is opened so it's always current.
                if tab == Tab::History {
                    return Task::perform(
                        async { kuvpn::load_events().unwrap_or_default() },
                        Message::HistoryLoaded,
                    );
                }
                Task::none()
            }
            Message::DragWindow => {
                if let Some(id) = self.window_id {
                    iced::window::drag(id)
                } else {
                    Task::none()
                }
            }
            Message::MinimizeWindow => {
                if let Some(id) = self.window_id {
                    self.is_minimized = true;
                    iced::window::minimize(id, true)
                } else {
                    Task::none()
                }
            }
            Message::QuitRequested => {
                log::info!("Quit requested - cleaning up");
                // Disconnect VPN if connected
                if let Some(session) = &self.session {
                    if self.status == ConnectionStatus::Connected
                        || self.status == ConnectionStatus::Connecting
                    {
                        log::info!("Disconnecting VPN before quit");
                        session.cancel();
                        let session_clone = Arc::clone(session);
                        // Wait for disconnection to complete, with timeout
                        return Task::perform(
                            async move {
                                let start = std::time::Instant::now();
                                let timeout = std::time::Duration::from_secs(5);

                                // Wait for session to finish AND verify OpenConnect is stopped
                                while start.elapsed() < timeout {
                                    if session_clone.is_finished() {
                                        log::info!(
                                            "Session finished, waiting for OpenConnect to stop..."
                                        );
                                        // Give extra time for OpenConnect process to be killed
                                        tokio::time::sleep(std::time::Duration::from_millis(500))
                                            .await;

                                        // Verify OpenConnect is actually stopped
                                        if !kuvpn::is_openconnect_running() {
                                            log::info!("OpenConnect stopped successfully");
                                            break;
                                        }
                                        log::warn!("OpenConnect still running, waiting...");
                                    }
                                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                                }

                                if kuvpn::is_openconnect_running() {
                                    log::error!(
                                        "OpenConnect still running after timeout, force killing..."
                                    );
                                    if let Some(pid) = kuvpn::get_openconnect_pid() {
                                        let _ = kuvpn::kill_process(pid);
                                        tokio::time::sleep(std::time::Duration::from_millis(500))
                                            .await;
                                    }
                                }

                                log::info!("Cleanup complete, elapsed: {:?}", start.elapsed());
                            },
                            |_| Message::QuitAfterCleanup,
                        );
                    }
                }
                // No active connection, exit immediately
                iced::exit()
            }
            Message::QuitAfterCleanup => {
                log::info!("Cleanup complete, exiting");
                iced::exit()
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![];

        let need_tick = self.is_transitioning()
            || cfg!(unix)
                && self.status == ConnectionStatus::Connected
                && self.active_interface.is_none();
        if need_tick {
            subs.push(
                iced::time::every(std::time::Duration::from_millis(16)).map(|_| Message::Tick),
            );
        }

        if self.notif_fade > 0.0 {
            subs.push(
                iced::time::every(std::time::Duration::from_millis(50))
                    .map(|_| Message::ActionNotifTick),
            );
        }

        // GTK Event Loop pump (for Tray Icon on Linux)
        #[cfg(target_os = "linux")]
        subs.push(
            iced::time::every(std::time::Duration::from_millis(20)).map(|_| Message::GtkTick),
        );

        // Window events
        subs.push(
            iced::window::close_requests().map(|_| Message::ToggleVisibility {
                from_close_request: true,
            }),
        );
        subs.push(iced::window::close_events().map(Message::WindowClosed));
        subs.push(iced::event::listen_with(|event, _, _| {
            if let iced::Event::Window(iced::window::Event::Focused) = event {
                Some(Message::WindowFocused)
            } else {
                None
            }
        }));

        // Tray & Menu events
        subs.push(Subscription::run(|| {
            iced::stream::channel(
                10,
                |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
                    let tray_rx = TrayIconEvent::receiver();
                    let menu_rx = MenuEvent::receiver();
                    loop {
                        if let Ok(event) = tray_rx.try_recv() {
                            let _ = output.send(Message::TrayEvent(event)).await;
                        }
                        if let Ok(event) = menu_rx.try_recv() {
                            let _ = output.send(Message::MenuEvent(event)).await;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                    }
                },
            )
        }));

        Subscription::batch(subs)
    }
}

impl Default for KuVpnGui {
    fn default() -> Self {
        #[allow(unused_mut)]
        let mut settings = GuiSettings::load();

        crate::logger::GUI_LOGGER.set_level(log_level_from_slider(settings.log_level_val));

        // Detect which privilege escalation tools are installed on this system.
        // On Windows this is always empty (elevation is handled differently).
        let available_escalation_tools: Vec<&'static str> = {
            #[cfg(unix)]
            {
                kuvpn::list_available_escalation_tools()
            }
            #[cfg(not(unix))]
            {
                vec![]
            }
        };

        // If the saved escalation tool is no longer installed, auto-select the first
        // available one so the user doesn't start in a broken state.
        #[cfg(unix)]
        if !available_escalation_tools.is_empty()
            && !available_escalation_tools.contains(&settings.escalation_tool.as_str())
        {
            settings.escalation_tool = available_escalation_tools[0].to_string();
            let _ = settings.save();
        }

        Self {
            settings,
            current_tab: Tab::Connection,
            logs: vec!["Ready for secure campus access.".to_string()],
            status: ConnectionStatus::Disconnected,
            pending_request: None,
            pending_email: None,
            current_input: String::new(),
            show_password_held: false,
            mfa_info: None,
            status_message: "Ready to connect".to_string(),
            error_message: None,
            error_category: None,
            rotation: 0.0,
            oc_test_result: None,
            vpnc_script_test_result: None,
            automation_warning: None,
            oc_path_notification: None,
            oc_startup_tested: false,
            session_wipe_result: None,
            reset_notification: false,
            notif_fade: 0.0,
            history: Vec::new(),
            session: None,
            tray_icon: None,
            tray_menu: None,
            window_id: None,
            is_visible: false,
            is_minimized: false,
            window_close_pending: false,
            window_open_pending: false,
            last_tray_click: None,
            connection_start: None,
            active_interface: None,
            available_escalation_tools,
            was_shown_for_prompt: false,
            last_diagnostic_path: None,
            console_auto_scroll: true,
            log_file: kuvpn::get_user_data_dir()
                .ok()
                .and_then(|d| kuvpn::FileLogger::open(d.join("kuvpn-gui.log"))),
        }
    }
}
