use crate::dsid::run_login_and_get_dsid;
use crate::openconnect::{
    execute_openconnect, get_openconnect_pid, is_openconnect_running, is_vpn_interface_up,
    kill_process, VpnProcess,
};
#[cfg(not(windows))]
use crate::openconnect::locate_openconnect;
use crate::utils::{CancellationToken, CredentialsProvider};
use std::io::{BufRead, BufReader};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
    Error,
}

/// Parsed log message from the "Level|message" format used by the session log channel.
#[derive(Debug, Clone)]
pub struct ParsedLog {
    pub level: log::Level,
    pub message: String,
}

impl ParsedLog {
    /// Parses a "Level|message" string. Returns None if format is invalid.
    pub fn parse(raw: &str) -> Option<Self> {
        let (level_str, message) = raw.split_once('|')?;
        let level = match level_str {
            "Error" => log::Level::Error,
            "Warn" => log::Level::Warn,
            "Info" => log::Level::Info,
            "Debug" => log::Level::Debug,
            "Trace" => log::Level::Trace,
            _ => return None,
        };
        Some(Self {
            level,
            message: message.to_string(),
        })
    }

    /// Returns a short prefix for display (e.g. "ERR", "INF").
    pub fn prefix(&self) -> &'static str {
        match self.level {
            log::Level::Error => "ERR",
            log::Level::Warn => "WRN",
            log::Level::Info => "INF",
            log::Level::Debug => "DBG",
            log::Level::Trace => "TRC",
        }
    }
}

#[derive(Clone)]
pub struct SessionConfig {
    pub url: String,
    pub domain: String,
    pub user_agent: String,
    pub headless: bool,
    pub no_auto_login: bool,
    pub email: Option<String>,
    /// Path/name of the openconnect binary to locate and run.
    /// Not used on Windows — the elevated helper resolves it from its own directory.
    #[cfg(not(windows))]
    pub openconnect_path: String,
    pub escalation_tool: Option<String>,
    pub interface_name: String,
}

pub struct VpnSession {
    config: SessionConfig,
    status: Arc<Mutex<ConnectionStatus>>,
    cancel_token: CancellationToken,
    last_error: Arc<Mutex<Option<String>>>,
    error_category: Arc<Mutex<Option<crate::error::ErrorCategory>>>,
    logs_tx: Arc<Mutex<Option<crossbeam_channel::Sender<String>>>>,
    browser_pid: Arc<Mutex<Option<u32>>>,
}

impl VpnSession {
    pub fn new(config: SessionConfig) -> Self {
        Self {
            config,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            cancel_token: CancellationToken::new(),
            last_error: Arc::new(Mutex::new(None)),
            error_category: Arc::new(Mutex::new(None)),
            logs_tx: Arc::new(Mutex::new(None)),
            browser_pid: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_logs_tx(&self, tx: crossbeam_channel::Sender<String>) {
        *self.logs_tx.lock().unwrap() = Some(tx);
    }

    pub fn status(&self) -> ConnectionStatus {
        *self.status.lock().unwrap()
    }

    pub fn last_error(&self) -> Option<String> {
        self.last_error.lock().unwrap().clone()
    }

    pub fn error_category(&self) -> Option<crate::error::ErrorCategory> {
        *self.error_category.lock().unwrap()
    }

    /// Returns true if the session has reached a terminal state.
    pub fn is_finished(&self) -> bool {
        let s = self.status();
        s == ConnectionStatus::Disconnected || s == ConnectionStatus::Error
    }

    pub fn cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    pub fn cancel(&self) {
        self.cancel_token.cancel();
        let mut status = self.status.lock().unwrap();
        if *status == ConnectionStatus::Connected || *status == ConnectionStatus::Connecting {
            *status = ConnectionStatus::Disconnecting;
        }
        drop(status);

        // Force-kill the browser process to unblock any pending CDP calls
        if let Some(pid) = self.browser_pid.lock().unwrap().take() {
            log::info!("[*] Force-killing browser process (PID: {})", pid);
            #[cfg(unix)]
            {
                use nix::sys::signal::{self, Signal};
                use nix::unistd::Pid;
                let _ = signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
            }
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                let _ = std::process::Command::new("taskkill")
                    .creation_flags(CREATE_NO_WINDOW)
                    .args(["/F", "/PID", &pid.to_string()])
                    .status();
            }
        }
    }

    pub fn connect(&self, provider: Arc<dyn CredentialsProvider>) -> thread::JoinHandle<()> {
        {
            let mut s = self.status.lock().unwrap();
            if *s != ConnectionStatus::Disconnected && *s != ConnectionStatus::Error {
                return thread::spawn(|| {});
            }
            *s = ConnectionStatus::Connecting;
            *self.last_error.lock().unwrap() = None;
            *self.error_category.lock().unwrap() = None;
        }

        let status = Arc::clone(&self.status);
        let config = self.config.clone();
        let cancel_token = self.cancel_token.clone();
        let last_error = Arc::clone(&self.last_error);
        let error_category = Arc::clone(&self.error_category);
        let logs_tx = Arc::clone(&self.logs_tx);
        let browser_pid = Arc::clone(&self.browser_pid);

        thread::spawn(move || {
            let log = |msg: String| {
                if let Some(tx) = logs_tx.lock().unwrap().as_ref() {
                    let _ = tx.send(msg);
                }
            };

            let interface_name = &config.interface_name;

            // Check if VPN is already connected.
            // On Unix, we use the named TUN interface (precise).
            // On Windows, we fall back to process-name detection since
            // --interface is not passed on Windows.
            let is_vpn_connected =
                || -> bool { is_vpn_interface_up(interface_name) || is_openconnect_running() };

            // Check if already connected
            let already_connected = is_vpn_connected();
            let mut process: Option<VpnProcess> = None;

            if already_connected {
                log("Info|VPN interface already active, monitoring...".to_string());
                *status.lock().unwrap() = ConnectionStatus::Connected;
            } else {
                log("Info|Accessing campus gateway...".to_string());

                // 1. Get DSID
                let dsid = match run_login_and_get_dsid(
                    config.headless,
                    &config.url,
                    &config.domain,
                    &config.user_agent,
                    config.no_auto_login,
                    config.email.clone(),
                    provider.as_ref(),
                    Some(cancel_token.clone()),
                    Some(Arc::clone(&browser_pid)),
                ) {
                    Ok(d) => {
                        // Browser is done, clear PID so cancel() won't kill a dead process
                        *browser_pid.lock().unwrap() = None;
                        d
                    }
                    Err(e) => {
                        *browser_pid.lock().unwrap() = None;
                        let mut s = status.lock().unwrap();
                        if *s != ConnectionStatus::Disconnecting {
                            *s = ConnectionStatus::Error;

                            // Try to extract AuthError category
                            let category = if let Some(auth_err) =
                                e.downcast_ref::<crate::error::AuthError>()
                            {
                                Some(auth_err.category())
                            } else {
                                Some(crate::error::ErrorCategory::Authentication)
                            };

                            *last_error.lock().unwrap() = Some(e.to_string());
                            *error_category.lock().unwrap() = category;
                            log(format!("Error|{}", e));
                        } else {
                            *s = ConnectionStatus::Disconnected;
                        }
                        return;
                    }
                };

                if cancel_token.is_cancelled() {
                    *status.lock().unwrap() = ConnectionStatus::Disconnected;
                    return;
                }

                log("Info|Initializing tunnel...".to_string());
                thread::sleep(Duration::from_millis(100));

                // 2. Locate OpenConnect
                // On Windows the elevated helper resolves the binary itself — no lookup needed.
                #[cfg(not(windows))]
                let oc_path = match locate_openconnect(&config.openconnect_path) {
                    Some(p) => p,
                    None => {
                        *status.lock().unwrap() = ConnectionStatus::Error;
                        *last_error.lock().unwrap() = Some(format!(
                            "Could not locate openconnect at '{}'. Please install openconnect or set the correct path.",
                            config.openconnect_path
                        ));
                        *error_category.lock().unwrap() =
                            Some(crate::error::ErrorCategory::Connection);
                        log(format!(
                            "Error|Could not locate openconnect at '{}'",
                            config.openconnect_path
                        ));
                        return;
                    }
                };
                // On Windows: dummy value — execute_openconnect ignores it.
                #[cfg(windows)]
                let oc_path = std::path::PathBuf::new();

                // 2.5. Check if we need to prompt for sudo password.
                // Uses the CredentialsProvider so both CLI (dialoguer) and GUI (modal)
                // get a proper prompt instead of relying on sudo's native terminal prompt
                // (which breaks when stdout/stderr are piped).
                let sudo_password: Option<String> = {
                    #[cfg(unix)]
                    {
                        use crate::openconnect::{
                            find_askpass, resolve_escalation_tool, tool_requires_password,
                            verify_escalation_password,
                        };

                        let tool = resolve_escalation_tool(&config.escalation_tool);
                        if let Some(ref tool_name) = tool {
                            if tool_requires_password(tool_name) && find_askpass().is_none() {
                                // Prompt and verify the password directly against the tool.
                                // Re-prompt as many times as needed — no hardcoded limit.
                                // The loop exits when:
                                //   - verification succeeds (Some(true))
                                //   - verification is indeterminate/timeout (None) — proceed
                                //     and let the actual sudo/openconnect invocation decide
                                //   - the user cancels (empty input + cancel token)
                                let mut wrong_password = false;
                                let pw = loop {
                                    let prompt = if wrong_password {
                                        format!(
                                            "Wrong password — please try again. \
                                             Enter your {} password to start the VPN tunnel",
                                            tool_name
                                        )
                                    } else {
                                        format!(
                                            "Enter your {} password to start the VPN tunnel",
                                            tool_name
                                        )
                                    };
                                    log(format!(
                                        "Info|{} requires a password. Prompting...",
                                        tool_name
                                    ));
                                    let entered = provider.request_password(&prompt);
                                    if entered.is_empty() && cancel_token.is_cancelled() {
                                        *status.lock().unwrap() = ConnectionStatus::Disconnected;
                                        return;
                                    }
                                    match verify_escalation_password(tool_name, &entered) {
                                        Some(true) => break entered,  // verified correct
                                        Some(false) => wrong_password = true, // re-prompt
                                        None => break entered, // unverifiable — let sudo decide
                                    }
                                };
                                Some(pw)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    #[cfg(not(unix))]
                    {
                        None
                    }
                };

                // 3. Execute OpenConnect
                let mut proc = match execute_openconnect(
                    dsid,
                    config.url.clone(),
                    &config.escalation_tool,
                    &oc_path,
                    Stdio::piped(),
                    Stdio::piped(),
                    interface_name,
                    sudo_password,
                ) {
                    Ok(p) => p,
                    Err(e) => {
                        *status.lock().unwrap() = ConnectionStatus::Error;
                        *last_error.lock().unwrap() = Some(e.to_string());
                        *error_category.lock().unwrap() =
                            Some(crate::error::ErrorCategory::Connection);
                        log(format!("Error|{}", e));
                        return;
                    }
                };

                // Capture logs if possible (Unix)
                if let VpnProcess::Unix(ref mut child) = proc {
                    if let Some(stdout) = child.stdout.take() {
                        let logs_tx_stdout = Arc::clone(&logs_tx);
                        thread::spawn(move || {
                            let reader = BufReader::new(stdout);
                            for l in reader.lines().map_while(Result::ok) {
                                if let Some(tx) = logs_tx_stdout.lock().unwrap().as_ref() {
                                    let _ = tx.send(format!("Info|{}", l));
                                }
                            }
                        });
                    }

                    if let Some(stderr) = child.stderr.take() {
                        let logs_tx_stderr = Arc::clone(&logs_tx);
                        thread::spawn(move || {
                            let reader = BufReader::new(stderr);
                            for l in reader.lines().map_while(Result::ok) {
                                if let Some(tx) = logs_tx_stderr.lock().unwrap().as_ref() {
                                    let _ = tx.send(format!("Warn|{}", l));
                                }
                            }
                        });
                    }
                }

                process = Some(proc);
            }

            // 4. Watchdog loop - uses TUN interface detection as primary signal
            let start_time = Instant::now();
            let mut connected_detected = already_connected;
            let timeout = Duration::from_secs(30);

            loop {
                if cancel_token.is_cancelled() {
                    break;
                }

                let interface_up = is_vpn_connected();

                if interface_up {
                    if !connected_detected {
                        connected_detected = true;
                        *status.lock().unwrap() = ConnectionStatus::Connected;
                        log("Info|Connected.".to_string());
                    }
                } else if connected_detected {
                    // Interface went down - disconnected
                    break;
                } else if start_time.elapsed() > timeout {
                    *status.lock().unwrap() = ConnectionStatus::Error;
                    *last_error.lock().unwrap() =
                        Some("VPN tunnel failed to establish within timeout".to_string());
                    *error_category.lock().unwrap() = Some(crate::error::ErrorCategory::Connection);
                    log("Error|VPN tunnel failed to establish within timeout".to_string());
                    if let Some(ref mut p) = process {
                        let _ = p.kill();
                    }
                    return;
                } else if let Some(ref mut p) = process {
                    // Check if our spawned process (sudo/pkexec) is still alive.
                    // This is the key check: if sudo is waiting for a password, the process
                    // is still alive and we should keep waiting. Only fail if the process
                    // has actually exited (e.g. wrong password, permission denied).
                    if !p.is_process_alive() {
                        *status.lock().unwrap() = ConnectionStatus::Error;
                        *last_error.lock().unwrap() = Some(
                            "OpenConnect process exited before tunnel was established".to_string(),
                        );
                        *error_category.lock().unwrap() =
                            Some(crate::error::ErrorCategory::Connection);
                        log(
                            "Error|OpenConnect process exited before tunnel was established"
                                .to_string(),
                        );
                        return;
                    }
                }

                thread::sleep(Duration::from_millis(1000));
            }

            // Cleanup
            log("Info|Disconnecting...".to_string());
            if let Some(ref mut p) = process {
                let _ = p.kill();
                let _ = p.wait();
            }

            // Fallback/Verify: Check if openconnect is still running and try to kill it again
            if let Some(pid) = get_openconnect_pid() {
                let _ = kill_process(pid);
                thread::sleep(Duration::from_millis(500));
            }

            if is_openconnect_running() {
                let mut s = status.lock().unwrap();
                *s = ConnectionStatus::Error;
                *last_error.lock().unwrap() =
                    Some("Failed to stop OpenConnect. Please close it manually.".to_string());
                log("Error|Failed to stop OpenConnect.".to_string());
            } else {
                *status.lock().unwrap() = ConnectionStatus::Disconnected;
                log("Info|Disconnected.".to_string());
            }
        })
    }
}
