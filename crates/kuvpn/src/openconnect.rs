use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::process::Command;
use std::process::{Child, Stdio};
use which::which;

#[cfg(windows)]
use sysinfo::System;

pub enum VpnProcess {
    Unix(Child),
    Windows {
        interface_name: String,
        #[cfg(windows)]
        helper: crate::win_elevated::WinElevatedClient,
    },
}

impl VpnProcess {
    pub fn kill(&mut self) -> anyhow::Result<()> {
        match self {
            VpnProcess::Unix(_child) => {
                #[cfg(unix)]
                {
                    use nix::sys::signal::{self, Signal};
                    use nix::unistd::Pid;

                    let pid = Pid::from_raw(_child.id() as i32);

                    // Step 1: Send SIGTERM to the escalation process (sudo/pkexec).
                    // sudo forwards this to openconnect; pkexec does NOT.
                    let _ = signal::kill(pid, Signal::SIGTERM);

                    // Step 2: Kill openconnect directly using the available privilege tools.
                    // Since we just ran `sudo openconnect`, credentials are cached so
                    // `sudo -n` works without another password prompt. For pkexec users
                    // (pkexec doesn't forward signals), this is the only reliable path.
                    if let Some(oc_pid) = get_openconnect_pid() {
                        let oc_pid_str = oc_pid.to_string();
                        // sudo / sudo-rs -n: cached credentials, no dialog
                        for tool in &["sudo", "sudo-rs"] {
                            if let Ok(tool_path) = which(tool) {
                                let _ = Command::new(tool_path)
                                    .args(["-n", "kill", "-15", &oc_pid_str])
                                    .stdout(Stdio::null())
                                    .stderr(Stdio::null())
                                    .status();
                            }
                        }
                        // pkexec: last resort for pure-pkexec systems (may show auth dialog)
                        if let Ok(pkexec_path) = which("pkexec") {
                            let _ = Command::new(pkexec_path)
                                .args(["kill", &oc_pid_str])
                                .stdout(Stdio::null())
                                .stderr(Stdio::null())
                                .status();
                        }
                    }

                    // Step 3: Wait with a timeout instead of blocking indefinitely.
                    // Without a timeout, if openconnect ignores the signal (or pkexec
                    // already exited without cleaning up), _child.wait() blocks forever.
                    let start = std::time::Instant::now();
                    loop {
                        match _child.try_wait() {
                            Ok(Some(_)) => break,
                            Ok(None) => {
                                if start.elapsed() > std::time::Duration::from_secs(5) {
                                    let _ = signal::kill(pid, Signal::SIGKILL);
                                    break;
                                }
                                std::thread::sleep(std::time::Duration::from_millis(200));
                            }
                            Err(_) => break,
                        }
                    }
                    let _ = _child.wait();
                }
                Ok(())
            }
            VpnProcess::Windows {
                #[cfg(windows)]
                ref mut helper,
                ..
            } => {
                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    use std::process::Command as StdCommand;
                    const CREATE_NO_WINDOW: u32 = 0x08000000;

                    // Route through the already-elevated helper — no UAC prompt.
                    log::info!("Stopping OpenConnect via elevated helper...");
                    if let Err(e) = helper.stop_openconnect() {
                        log::warn!("Helper stop failed ({}), trying non-elevated fallback...", e);
                        // Last resort: non-elevated taskkill (may fail for elevated processes)
                        let _ = StdCommand::new("taskkill")
                            .creation_flags(CREATE_NO_WINDOW)
                            .args(["/F", "/IM", "openconnect.exe"])
                            .status();
                    }
                }
                Ok(())
            }
        }
    }

    /// Checks if the spawned process (sudo/pkexec) is still alive.
    /// This is different from is_openconnect_running() - it checks the actual
    /// child process we spawned, not searching by process name.
    pub fn is_process_alive(&mut self) -> bool {
        match self {
            VpnProcess::Unix(_child) => {
                #[cfg(unix)]
                {
                    // try_wait returns Ok(None) if the process is still running
                    match _child.try_wait() {
                        Ok(None) => true, // still running
                        _ => false,       // exited or error
                    }
                }
                #[cfg(not(unix))]
                false
            }
            VpnProcess::Windows { .. } => {
                #[cfg(windows)]
                {
                    return is_openconnect_running();
                }
                #[cfg(not(windows))]
                false
            }
        }
    }

    pub fn wait(&mut self) -> anyhow::Result<()> {
        match self {
            VpnProcess::Unix(child) => {
                child.wait()?;
                Ok(())
            }
            VpnProcess::Windows { .. } => {
                // The helper manages the openconnect lifetime.
                // Poll until openconnect exits (or give up after a timeout).
                #[cfg(windows)]
                {
                    let start = std::time::Instant::now();
                    while is_openconnect_running() {
                        if start.elapsed() > std::time::Duration::from_secs(5) {
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(200));
                    }
                }
                Ok(())
            }
        }
    }
}

/// Attempts to locate the `openconnect` executable.
pub fn locate_openconnect(user_path: &str) -> Option<PathBuf> {
    // Step 1: Check if `user_path` directly points to an existing file.
    let candidate = Path::new(user_path);
    if candidate.exists() && candidate.is_file() {
        return Some(candidate.to_path_buf());
    }

    // Step 2: Attempt to locate `openconnect` using the system's PATH.
    if let Ok(found) = which(user_path) {
        return Some(found);
    }

    // Step 3: Fallback to searching in common sbin directories.
    #[cfg(unix)]
    {
        let fallback_dirs = [
            "/sbin",
            "/usr/sbin",
            "/usr/local/sbin",
            "/usr/local/bin",
            "/opt/homebrew/bin",
        ];
        for dir in &fallback_dirs {
            let path_in_dir = Path::new(dir).join("openconnect");
            if path_in_dir.exists() && path_in_dir.is_file() {
                return Some(path_in_dir);
            }
        }
    }

    #[cfg(windows)]
    {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(parent) = exe_path.parent() {
                let bundled_path = parent.join("openconnect").join("openconnect.exe");
                if bundled_path.exists() && bundled_path.is_file() {
                    return Some(bundled_path);
                }
            }
        }

        let common_paths = [
            "C:\\Program Files\\OpenConnect\\openconnect.exe",
            "C:\\Program Files (x86)\\OpenConnect\\openconnect.exe",
        ];
        for path in &common_paths {
            let p = Path::new(path);
            if p.exists() && p.is_file() {
                return Some(p.to_path_buf());
            }
        }
    }

    None
}

/// Searches for an available askpass program on the system.
/// Returns the path to the askpass program if found.
#[cfg(unix)]
pub fn find_askpass() -> Option<PathBuf> {
    // Check if SUDO_ASKPASS is already set
    if let Ok(askpass) = std::env::var("SUDO_ASKPASS") {
        let p = Path::new(&askpass);
        if p.exists() && p.is_file() {
            return Some(p.to_path_buf());
        }
    }

    // Search for common askpass programs
    let askpass_programs = [
        "ssh-askpass",
        "ksshaskpass",
        "lxqt-openssh-askpass",
        "x11-ssh-askpass",
        "gnome-ssh-askpass",
    ];
    for prog in &askpass_programs {
        if let Ok(path) = which(prog) {
            return Some(path);
        }
    }

    None
}

/// Determines which escalation tool will be used (for checking if password prompt is needed).
#[cfg(unix)]
pub fn resolve_escalation_tool(run_command: &Option<String>) -> Option<String> {
    let mut default_tools = vec!["sudo", "sudo-rs", "pkexec"];

    if let Some(custom_command) = run_command {
        if which(custom_command).is_ok() {
            default_tools.insert(0, custom_command.as_str());
        }
    }

    default_tools
        .iter()
        .find_map(|&tool| which(tool).ok().map(|_| tool.to_string()))
}

/// Returns the ordered list of escalation tools that are actually installed on this system.
///
/// - On macOS: pkexec is never included (it is not available on macOS).
/// - On other Unix: sudo, sudo-rs, and pkexec are checked in preference order.
///
/// Only tools found via `which` are returned, so callers can use this list to populate
/// a UI selector showing only the options the user can actually use.
#[cfg(unix)]
pub fn list_available_escalation_tools() -> Vec<&'static str> {
    let candidates: &[&'static str] = if cfg!(target_os = "macos") {
        &["sudo", "sudo-rs"]
    } else {
        &["sudo", "sudo-rs", "pkexec"]
    };
    candidates
        .iter()
        .copied()
        .filter(|&tool| which(tool).is_ok())
        .collect()
}

/// Returns true if the given escalation tool needs a password piped via stdin
/// (i.e., it's sudo or sudo-rs, not pkexec which has its own GUI prompt).
#[cfg(unix)]
pub fn needs_password_prompt(tool: &str) -> bool {
    let base = Path::new(tool)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(tool);
    matches!(base, "sudo" | "sudo-rs")
}

/// Checks whether the tool actually requires a password right now by doing a
/// non-interactive dry-run (`sudo -n true` / `sudo-rs -n true`).
///
/// Returns `false` (no prompt needed) when:
/// - The user has NOPASSWD configured in sudoers
/// - Credentials are already cached from a recent successful run
///
/// Returns `true` (prompt needed) when the tool exits non-zero, indicating it
/// would block waiting for a password if run interactively.
/// pkexec always returns `false` — it uses its own GUI prompt.
#[cfg(unix)]
pub fn tool_requires_password(tool: &str) -> bool {
    let base = Path::new(tool)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(tool);

    match base {
        "sudo" | "sudo-rs" => which(base)
            .ok()
            .map(|p| {
                Command::new(p)
                    .args(["-n", "true"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .map(|s| !s.success()) // non-zero → password required
                    .unwrap_or(true)
            })
            .unwrap_or(false),
        _ => false, // pkexec uses its own GUI prompt, never needs stdin
    }
}

/// Verifies a sudo/sudo-rs password with a quick non-interactive test (`sudo -S true`).
///
/// Both sudo and sudo-rs support `-S` (read password from stdin once, then exit).
/// Even when PAM enforces a failure delay (typically 2-5 s) the process will always
/// exit, so a plain blocking wait is safe and avoids falsely treating a slow
/// rejection as "unverifiable".
///
/// Returns:
/// - `Some(true)`  — password accepted
/// - `Some(false)` — password rejected
/// - `None`        — tool not found, not applicable (pkexec), or could not be spawned
#[cfg(unix)]
pub fn verify_escalation_password(tool: &str, password: &str) -> Option<bool> {
    use std::io::Write;

    let base = Path::new(tool)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(tool);

    if !matches!(base, "sudo" | "sudo-rs") {
        return None; // pkexec / unknown — not verifiable via stdin
    }

    let tool_path = which(base).ok()?;

    let mut child = Command::new(&tool_path)
        .args(["-S", "true"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    // Write the password and close stdin so the tool sees EOF immediately.
    if let Some(mut stdin) = child.stdin.take() {
        let _ = writeln!(stdin, "{}", password);
    }

    // Blocking wait — sudo/sudo-rs -S always exits once it has read from stdin.
    Some(child.wait().map(|s| s.success()).unwrap_or(false))
}

/// Detects an active VPN utun interface on macOS by parsing `ifconfig` output.
/// Returns the name of a utun interface that has an IPv4 `inet` address, which indicates
/// an active point-to-point VPN tunnel (as opposed to system-managed utun interfaces that
/// typically only carry IPv6 link-local addresses).
#[cfg(target_os = "macos")]
fn detect_active_utun() -> Option<String> {
    let output = Command::new("/sbin/ifconfig").output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut current_utun: Option<String> = None;
    let mut found: Option<String> = None;

    for line in stdout.lines() {
        // Interface header lines have no leading whitespace: "utunN: flags=..."
        if !line.starts_with('\t') && !line.starts_with(' ') {
            if let Some(colon_pos) = line.find(':') {
                let iface = &line[..colon_pos];
                current_utun = if iface.starts_with("utun") {
                    Some(iface.to_string())
                } else {
                    None
                };
            }
        } else if current_utun.is_some() {
            // Indented lines hold the interface configuration
            let trimmed = line.trim_start();
            // `inet ` (IPv4) — not `inet6 ` — signals an active VPN tunnel
            if trimmed.starts_with("inet ") {
                found = current_utun.clone();
            }
        }
    }

    found
}

/// Returns the name of the currently active VPN interface, or `None` if it cannot be
/// determined.
///
/// - **Linux**: returns `configured_name` when the interface exists in sysfs.
/// - **macOS**: detects the active `utun%d` interface via `ifconfig` (ignores `configured_name`
///   because macOS does not support custom TUN interface names; OpenConnect auto-assigns one).
/// - **Windows**: always returns `None`.
pub fn get_vpn_interface_name(configured_name: &str) -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        let _ = configured_name;
        return detect_active_utun();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let sys_path = format!("/sys/class/net/{}", configured_name);
        if std::path::Path::new(&sys_path).exists() {
            return Some(configured_name.to_string());
        }
        None
    }
    #[cfg(windows)]
    {
        let _ = configured_name;
        None
    }
}

/// Executes the `openconnect` command.
#[allow(clippy::too_many_arguments)]
pub fn execute_openconnect(
    cookie_value: String,
    url: String,
    _run_command: &Option<String>,
    _openconnect_path: &Path,
    _stdout: Stdio,
    _stderr: Stdio,
    interface_name: &str,
    _sudo_password: Option<String>,
) -> anyhow::Result<VpnProcess> {
    #[cfg(unix)]
    {
        let mut default_tools = vec!["sudo", "sudo-rs", "pkexec"];

        if let Some(custom_command) = _run_command {
            if which(custom_command).is_ok() {
                default_tools.insert(0, custom_command.as_str());
            }
        }

        let command_to_run = default_tools
            .iter()
            .find_map(|&tool| which(tool).ok().map(|_| tool))
            .ok_or(anyhow::anyhow!(
                "No available tool for running openconnect (sudo, sudo-rs, or pkexec not found)"
            ))?;

        let tool_base = Path::new(command_to_run)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(command_to_run);

        let askpass = find_askpass();
        let use_askpass = askpass.is_some() && needs_password_prompt(tool_base);
        let use_stdin_password =
            !use_askpass && _sudo_password.is_some() && needs_password_prompt(tool_base);

        let mut cmd = Command::new(command_to_run);

        // Configure password feeding method
        if use_askpass {
            let askpass_path = askpass.unwrap();
            log::info!("Using askpass program: {:?}", askpass_path);
            cmd.env("SUDO_ASKPASS", &askpass_path);
            if matches!(tool_base, "sudo" | "sudo-rs") {
                cmd.arg("-A");
            }
        } else if use_stdin_password {
            log::info!("Piping password via stdin to {}", tool_base);
            if matches!(tool_base, "sudo" | "sudo-rs") {
                cmd.arg("-S");
            }
            cmd.stdin(Stdio::piped());
        }

        cmd.arg(_openconnect_path).arg("--protocol").arg("nc");

        // macOS does not support custom TUN interface names; OpenConnect auto-assigns a
        // utun%d interface. Only pass --interface on Linux and other non-macOS Unix.
        #[cfg(not(target_os = "macos"))]
        cmd.arg("--interface").arg(interface_name);

        cmd.arg("-C")
            .arg(format!("DSID={}", cookie_value))
            .arg(url)
            .stdout(_stdout)
            .stderr(_stderr);

        let mut child = cmd.spawn()?;

        // Pipe password to stdin if needed
        if use_stdin_password {
            if let Some(ref password) = _sudo_password {
                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    let _ = writeln!(stdin, "{}", password);
                    // Drop stdin to close it so the process doesn't hang waiting for more input
                    drop(stdin);
                }
            }
        }

        Ok(VpnProcess::Unix(child))
    }

    #[cfg(windows)]
    {
        // Find the helper binary next to the current executable.
        let helper_exe = std::env::current_exe()?
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine executable directory"))?
            .join("kuvpn-win-helper.exe");

        if !helper_exe.exists() {
            anyhow::bail!(
                "kuvpn-win-helper.exe not found next to the main executable. \
                 Please reinstall KUVPN."
            );
        }

        // Launch helper elevated (one UAC prompt) and start openconnect through it.
        // No exe path is sent over the wire; the helper resolves openconnect locally.
        let mut helper = crate::win_elevated::WinElevatedClient::launch(&helper_exe)?;
        helper.start_openconnect(&format!("DSID={}", cookie_value), &url)?;

        Ok(VpnProcess::Windows {
            interface_name: interface_name.to_string(),
            helper,
        })
    }
}

/// Checks if an openconnect process is currently running
pub fn is_openconnect_running() -> bool {
    #[cfg(windows)]
    {
        let mut sys = System::new_all();
        sys.refresh_all();
        for process in sys.processes().values() {
            let name = process.name().to_string_lossy().to_lowercase();
            if name.contains("openconnect") {
                return true;
            }
        }
        false
    }
    #[cfg(unix)]
    {
        let candidate = which("pgrep");
        if let Ok(pgrep) = candidate {
            let status = Command::new(pgrep)
                .arg("openconnect")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            return status.map(|s| s.success()).unwrap_or(false);
        }
        false
    }
}

/// Checks if the named VPN TUN/TAP interface is up and active.
/// More reliable than process-name detection because it verifies
/// the tunnel itself exists, not just that a process is running.
pub fn is_vpn_interface_up(interface_name: &str) -> bool {
    #[cfg(unix)]
    {
        #[cfg(target_os = "macos")]
        {
            // On macOS, OpenConnect auto-assigns a utun%d interface so the configured
            // name is not meaningful; detect any active utun with an IPv4 address instead.
            let _ = interface_name;
            return detect_active_utun().is_some();
        }

        #[cfg(not(target_os = "macos"))]
        {
            // Check /sys/class/net/<interface_name> existence
            let sys_path = format!("/sys/class/net/{}", interface_name);
            let path = std::path::Path::new(&sys_path);
            if !path.exists() {
                return false;
            }
            // TUN devices report "unknown" when active, "down" when inactive
            let operstate_path = format!("{}/operstate", sys_path);
            if let Ok(state) = std::fs::read_to_string(&operstate_path) {
                return state.trim() != "down";
            }
            true
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use std::process::{Command as StdCommand, Stdio as StdStdio};
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let output = StdCommand::new("netsh")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["interface", "show", "interface"])
            .stdout(StdStdio::piped())
            .stderr(StdStdio::null())
            .output();
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                for line in stdout.lines() {
                    if line.contains(interface_name) && line.contains("Connected") {
                        return true;
                    }
                }
                false
            }
            Err(_) => false,
        }
    }
}

/// Gets the PID of the running openconnect process
pub fn get_openconnect_pid() -> Option<u32> {
    #[cfg(windows)]
    {
        let mut sys = System::new_all();
        sys.refresh_all();
        for (pid, process) in sys.processes() {
            let name = process.name().to_string_lossy().to_lowercase();
            if name.contains("openconnect") {
                return Some(pid.as_u32());
            }
        }
        None
    }
    #[cfg(unix)]
    {
        let candidate = which("pgrep");
        if let Ok(pgrep) = candidate {
            let output = Command::new(pgrep).arg("openconnect").output().ok()?;
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = s.lines().next() {
                    return line.parse().ok();
                }
            }
        }
        None
    }
}

/// Gracefully terminates a process by its PID.
pub fn kill_process(pid: u32) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use nix::sys::signal::{self, Signal};
        use nix::unistd::Pid;

        let pid_str = pid.to_string();

        // Prefer killing via privilege tools with cached credentials (no password prompt).
        // openconnect runs as root, so direct signals from a non-root process fail with EPERM.
        // Try each tool in order; stop as soon as one succeeds.
        let elevated_ok =
            // sudo / sudo-rs -n: uses cached credentials, never prompts
            ["sudo", "sudo-rs"].iter().any(|&tool| {
                which(tool).ok().map(|p| {
                    Command::new(p)
                        .args(["-n", "kill", "-15", &pid_str])
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false)
                }).unwrap_or(false)
            })
            // pkexec: last resort for pure-pkexec systems; may show a brief auth dialog
            || if let Ok(pkexec_path) = which("pkexec") {
                Command::new(pkexec_path)
                    .args(["kill", &pid_str])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
            } else { false };

        if !elevated_ok {
            // Fallback: direct signal. May fail with EPERM for root processes,
            // but worth trying (e.g. if openconnect wasn't started as root).
            let nix_pid = Pid::from_raw(pid as i32);
            let _ = signal::kill(nix_pid, Signal::SIGTERM);
            std::thread::sleep(std::time::Duration::from_millis(1000));
            let _ = signal::kill(nix_pid, Signal::SIGKILL);
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use std::process::Command as StdCommand;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        // Non-elevated fallback — the helper-based path is the primary stop mechanism.
        let _ = StdCommand::new("taskkill")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .status();
    }
    Ok(())
}
