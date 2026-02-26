use futures::SinkExt;
use iced::{Subscription, Task};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tray_icon::{
    menu::{MenuEvent, MenuItem},
    TrayIcon, TrayIconEvent,
};

use crate::config::GuiSettings;
use crate::provider::{GuiInteraction, GuiProvider};
use crate::types::{
    log_level_from_slider, login_mode_flags, ConnectionStatus, InputRequest, InputRequestWrapper,
    Message, Tab,
};
use kuvpn::{SessionConfig, VpnSession};

fn load_window_icon() -> Option<iced::window::Icon> {
    iced::window::icon::from_file_data(crate::types::WINDOW_ICON, Some(image::ImageFormat::Png))
        .ok()
}

pub struct KuVpnGui {
    // Settings
    pub settings: GuiSettings,

    // UI State
    pub current_tab: Tab,
    pub logs: Vec<String>,
    pub status: ConnectionStatus,
    pub pending_request: Option<InputRequest>,
    pub current_input: String,
    pub show_password_held: bool,
    pub mfa_info: Option<String>,
    pub status_message: String,
    pub error_message: Option<String>,
    pub error_category: Option<kuvpn::ErrorCategory>,
    pub rotation: f32,
    #[cfg(not(windows))]
    pub oc_test_result: Option<bool>,
    pub automation_warning: Option<String>,
    /// Set when the Test button resolves a different path than what was entered.
    #[cfg(not(windows))]
    pub oc_path_notification: Option<String>,
    /// True after the first (startup) auto-test completes; used to suppress
    /// the replacement notification for the initial auto-detection pass.
    #[cfg(not(windows))]
    pub oc_startup_tested: bool,

    // VPN Session
    pub session: Option<Arc<VpnSession>>,

    // Tray & Window state
    pub tray_icon: Option<TrayIcon>,
    pub show_item: Option<MenuItem>,
    pub connect_item: Option<MenuItem>,
    pub disconnect_item: Option<MenuItem>,
    pub window_id: Option<iced::window::Id>,
    pub is_visible: bool,
    pub window_close_pending: bool,
    pub window_open_pending: bool,
    pub last_tray_click: Option<std::time::Instant>,
    pub connection_start: Option<Instant>,
    /// Detected name of the active VPN interface (e.g. "kuvpn0" on Linux, "utun3" on macOS).
    /// `None` when not connected or on platforms where it cannot be determined (Windows).
    pub active_interface: Option<String>,
    /// Privilege escalation tools found on this system (e.g. ["pkexec", "sudo"]).
    /// Empty on Windows. Empty on Unix means no tool is installed — VPN cannot start.
    pub available_escalation_tools: Vec<&'static str>,
    /// True when the window was auto-shown from a hidden state for a prompt.
    /// Used to auto-hide the window again after the prompt resolves.
    pub was_shown_for_prompt: bool,
}

impl KuVpnGui {
    pub fn theme(&self, _id: iced::window::Id) -> iced::Theme {
        iced::Theme::Dark
    }

    fn save_settings(&self) {
        if let Err(e) = self.settings.save() {
            log::error!("Failed to save settings: {}", e);
        }
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
                return iced::window::gain_focus(id);
            }
            Task::none()
        } else {
            Task::none()
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowOpened(id) => {
                log::info!("Window opened with ID: {:?}", id);
                self.window_id = Some(id);
                self.is_visible = true;
                self.window_close_pending = false;
                self.window_open_pending = false;
                if let Some(item) = &self.show_item {
                    item.set_text("Toggle Visibility");
                }
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
                    self.window_close_pending = false;
                }
                Task::none()
            }
            Message::ResetClosePending => {
                log::info!("Resetting window_close_pending (safety timeout)");
                self.window_close_pending = false;
                Task::none()
            }
            Message::GtkTick => {
                #[cfg(target_os = "linux")]
                {
                    while gtk::events_pending() {
                        gtk::main_iteration();
                    }
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
                if let Some(id) = self.window_id {
                    self.is_visible = false;
                    self.window_close_pending = true;
                    return Task::batch(vec![
                        iced::window::close(id),
                        Task::perform(
                            async {
                                tokio::time::sleep(std::time::Duration::from_millis(100)).await
                            },
                            |_| Message::ToggleVisibility {
                                from_close_request: false,
                            },
                        ),
                    ]);
                }
                Task::none()
            }
            Message::ToggleVisibility { from_close_request } => {
                log::info!(
                    "ToggleVisibility called. visible={}, close_to_tray={}, from_close_request={}, close_pending={}, open_pending={}",
                    self.is_visible,
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
                    let (id, task) = iced::window::open(iced::window::Settings {
                        exit_on_close_request: false,
                        size: iced::Size::new(580.0, 650.0),
                        min_size: Some(iced::Size::new(560.0, 580.0)),
                        max_size: Some(iced::Size::new(580.0, 650.0)),
                        resizable: false,
                        decorations: !use_csd,
                        transparent: use_csd,
                        icon: load_window_icon(),
                        ..Default::default()
                    });
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
            Message::EscalationToolChanged(tool) => {
                self.settings.escalation_tool = tool;
                self.save_settings();
                Task::none()
            }
            Message::LogLevelSliderChanged(val) => {
                self.settings.log_level_val = val;
                if let Ok(mut guard) = crate::logger::GUI_LOGGER.user_level.lock() {
                    *guard = log_level_from_slider(val);
                }
                self.save_settings();
                Task::none()
            }
            #[cfg(not(windows))]
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
                if self.status == ConnectionStatus::Connecting
                    || self.status == ConnectionStatus::Disconnecting
                {
                    self.rotation += 0.1;
                }
                Task::none()
            }
            // ConnectPressed handled above
            Message::ConnectPressed => {
                if self.status == ConnectionStatus::Disconnected
                    || self.status == ConnectionStatus::Error
                {
                    self.automation_warning = None; // Clear previous warnings
                    self.error_message = None;
                    self.error_category = None;
                    self.status_message = "Initializing...".to_string();
                    self.connection_start = Some(Instant::now());
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
                        #[cfg(not(windows))]
                        openconnect_path: if self.settings.openconnect_path.is_empty() {
                            "openconnect".to_string()
                        } else {
                            self.settings.openconnect_path.clone()
                        },
                        escalation_tool: Some(self.settings.escalation_tool.clone()),
                        interface_name: "kuvpn0".to_string(),
                    };

                    let session = Arc::new(VpnSession::new(config));
                    self.session = Some(Arc::clone(&session));
                    self.status = ConnectionStatus::Connecting;
                    self.logs.clear();

                    let (log_tx, log_rx) = crossbeam_channel::unbounded();
                    session.set_logs_tx(log_tx.clone());

                    // Initialize global logger once
                    crate::logger::LOGGER_INIT.call_once(|| {
                        let _ = log::set_logger(&crate::logger::GUI_LOGGER);
                        log::set_max_level(log::LevelFilter::Trace);
                    });

                    // Bridge global logger to our log stream
                    let (gui_tx, mut gui_rx) = tokio::sync::mpsc::channel(100);
                    if let Ok(mut guard) = crate::logger::GUI_LOGGER.tx.lock() {
                        *guard = Some(gui_tx);
                    }

                    return Task::stream(iced::stream::channel(
                        100,
                        move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
                            use futures::SinkExt;
                            let (interaction_tx, mut interaction_rx) =
                                tokio::sync::mpsc::channel(10);

                            let provider = Arc::new(GuiProvider {
                                interaction_tx,
                                cancel_token: kuvpn::utils::CancellationToken::new(),
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
                                            .send(Message::RequestInput(Arc::new(
                                                InputRequestWrapper(Mutex::new(Some(req))),
                                            )))
                                            .await;
                                    }
                                    Ok(GuiInteraction::MfaPush(code)) => {
                                        let _ = output.send(Message::MfaPushReceived(code)).await;
                                    }
                                    Ok(GuiInteraction::MfaComplete) => {
                                        let _ = output.send(Message::MfaCompleteReceived).await;
                                    }
                                    Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                                        break
                                    }
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
                    ));
                }
                Task::none()
            }
            Message::DisconnectPressed => {
                if let Some(session) = &self.session {
                    session.cancel();
                }
                Task::none()
            }
            Message::LogAppended(raw_log) => {
                if let Some(parsed) = kuvpn::ParsedLog::parse(&raw_log) {
                    // Update UI status/error messages
                    // Note: Don't overwrite error_message from logs since ConnectionFinished
                    // provides properly formatted, categorized error messages
                    match parsed.level {
                        log::Level::Warn | log::Level::Info => {
                            self.status_message = parsed.message.clone()
                        }
                        _ => {}
                    }

                    let user_filter = if let Ok(guard) = crate::logger::GUI_LOGGER.user_level.lock()
                    {
                        *guard
                    } else {
                        log::LevelFilter::Info
                    };

                    if parsed.level <= user_filter {
                        self.logs
                            .push(format!("[{}] {}", parsed.prefix(), parsed.message));
                        if self.logs.len() > 500 {
                            self.logs.remove(0);
                        }
                    }
                }
                Task::none()
            }

            Message::MfaPushReceived(code) => {
                self.mfa_info = Some(code.clone());
                log::info!("MFA received - bringing window to front");
                if !self.is_visible && !self.window_close_pending && !self.window_open_pending {
                    self.was_shown_for_prompt = true;
                }
                self.show_or_focus_window()
            }
            Message::MfaCompleteReceived => {
                self.mfa_info = None;
                if self.was_shown_for_prompt && self.settings.auto_hide_after_prompt {
                    self.was_shown_for_prompt = false;
                    return Task::perform(
                        async {
                            tokio::time::sleep(std::time::Duration::from_millis(400)).await
                        },
                        |_| Message::AutoHideWindow,
                    );
                }
                Task::none()
            }
            Message::RequestInput(wrapper) => {
                if let Ok(mut guard) = wrapper.0.lock() {
                    if let Some(req) = guard.take() {
                        self.pending_request = Some(req);
                        self.current_input = String::new();
                        log::info!("Input requested - bringing window to front");
                        if !self.is_visible && !self.window_close_pending && !self.window_open_pending {
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
                    let _ = req.response_tx.send(self.current_input.clone());
                    self.current_input = String::new();
                    self.show_password_held = false;
                }
                if self.was_shown_for_prompt && self.settings.auto_hide_after_prompt {
                    self.was_shown_for_prompt = false;
                    return Task::perform(
                        async {
                            tokio::time::sleep(std::time::Duration::from_millis(300)).await
                        },
                        |_| Message::AutoHideWindow,
                    );
                }
                Task::none()
            }
            Message::ShowPasswordHeld(held) => {
                self.show_password_held = held;
                Task::none()
            }
            Message::ClearSessionPressed => {
                match kuvpn::get_user_data_dir() {
                    Ok(dir) => {
                        if dir.exists() {
                            if let Err(e) = std::fs::remove_dir_all(&dir) {
                                self.logs
                                    .push(format!("[!] Failed to clear session: {}", e));
                            } else {
                                self.logs.push("[✓] Saved session data wiped.".to_string());
                            }
                        } else {
                            self.logs.push("[*] No active session found.".to_string());
                        }
                    }
                    Err(e) => {
                        self.logs.push(format!("[!] System error: {}", e));
                    }
                }
                Task::none()
            }
            Message::ConnectionFinished(err, category) => {
                self.status = if err.is_some() {
                    ConnectionStatus::Error
                } else {
                    ConnectionStatus::Disconnected
                };
                self.mfa_info = None;
                self.connection_start = None;
                self.active_interface = None;

                // Update tray icon based on final status
                if let Some(tray) = &self.tray_icon {
                    crate::tray::update_tray_icon(tray, self.status);
                }

                if let Some(e) = err {
                    // Store the error category for display
                    self.error_category = category;

                    // Check if this is an automation failure that needs the special banner
                    if matches!(category, Some(kuvpn::ErrorCategory::Authentication))
                        && (e.contains("Full Auto mode unable to complete login")
                            || e.contains("Could not find a handler"))
                    {
                        // For automation failures, show simple actionable message
                        self.automation_warning = Some(
                            "Full Auto mode was unable to complete the login flow.\n\n\
                             What to do:\n\
                             • Switch to Manual mode and complete login yourself\n\
                             • Try clearing session data (Wipe Session button)\n\
                             • Use Visual Auto mode to record a video for bug reporting\n\n\
                             Check console/logs for technical details."
                                .to_string(),
                        );
                        // Clear error_message so we don't show it twice
                        self.error_message = None;
                    } else {
                        // For other errors, show in the normal error display
                        self.error_message = Some(e.clone());
                    }
                }
                Task::none()
            }
            Message::StatusChanged(status) => {
                if self.status != status {
                    self.status = status;
                    if let Some(item) = &self.connect_item {
                        item.set_enabled(status == ConnectionStatus::Disconnected);
                    }
                    if let Some(item) = &self.disconnect_item {
                        item.set_enabled(status == ConnectionStatus::Connected);
                    }
                    // Detect and store the active VPN interface name when connected.
                    // On Linux this checks sysfs; on macOS it reads ifconfig for utun%d.
                    if status == ConnectionStatus::Connected {
                        #[cfg(unix)]
                        {
                            self.active_interface = kuvpn::get_vpn_interface_name("kuvpn0");
                        }
                    } else if status == ConnectionStatus::Disconnected
                        || status == ConnectionStatus::Error
                    {
                        self.active_interface = None;
                    }
                    // Update tray icon based on new status
                    if let Some(tray) = &self.tray_icon {
                        crate::tray::update_tray_icon(tray, status);
                    }
                }
                Task::none()
            }
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
                #[cfg(not(windows))]
                {
                    self.oc_test_result = None;
                    self.oc_path_notification = None;
                    self.oc_startup_tested = false; // treat the next test as a fresh startup
                }

                // If window decoration setting changed, refresh window
                if old_use_csd != self.settings.use_client_decorations {
                    if let Some(id) = self.window_id {
                        self.is_visible = false;
                        self.window_close_pending = true;
                        return Task::batch(vec![
                            iced::window::close(id),
                            Task::perform(
                                async {
                                    tokio::time::sleep(std::time::Duration::from_millis(100)).await
                                },
                                |_| Message::ToggleVisibility {
                                    from_close_request: false,
                                },
                            ),
                        ]);
                    }
                }
                Task::none()
            }
            #[cfg(not(windows))]
            Message::TestOpenConnect => {
                let path = self.settings.openconnect_path.clone();
                Task::perform(
                    async move {
                        kuvpn::locate_openconnect(&path).map(|p| p.to_string_lossy().into_owned())
                    },
                    Message::OpenConnectTestResult,
                )
            }
            #[cfg(not(windows))]
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

        if self.status == ConnectionStatus::Connecting
            || self.status == ConnectionStatus::Disconnecting
        {
            subs.push(
                iced::time::every(std::time::Duration::from_millis(16)).map(|_| Message::Tick),
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
        #[cfg(unix)]
        let mut settings = GuiSettings::load();
        #[cfg(not(unix))]
        let settings = GuiSettings::load();

        if let Ok(mut guard) = crate::logger::GUI_LOGGER.user_level.lock() {
            *guard = log_level_from_slider(settings.log_level_val);
        }

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
            current_input: String::new(),
            show_password_held: false,
            mfa_info: None,
            status_message: "Ready to connect".to_string(),
            error_message: None,
            error_category: None,
            rotation: 0.0,
            #[cfg(not(windows))]
            oc_test_result: None,
            automation_warning: None,
            #[cfg(not(windows))]
            oc_path_notification: None,
            #[cfg(not(windows))]
            oc_startup_tested: false,
            session: None,
            tray_icon: None,
            show_item: None,
            connect_item: None,
            disconnect_item: None,
            window_id: None,
            is_visible: false,
            window_close_pending: false,
            window_open_pending: false,
            last_tray_click: None,
            connection_start: None,
            active_interface: None,
            available_escalation_tools,
            was_shown_for_prompt: false,
        }
    }
}
