//! VPN helper mode for Windows.
//!
//! When KUVPN elevates itself to start a VPN tunnel it passes `--vpn-helper`
//! as the first argument so the elevated copy acts as a thin lifecycle manager
//! rather than a full GUI/CLI session.  This keeps the UAC prompt count to one
//! per connection: the helper starts OpenConnect as its own child (inheriting
//! the elevated token) and then waits for a named Windows kernel event to be
//! signalled by the non-elevated parent.  When the event fires the helper kills
//! the child and exits cleanly.
//!
//! Argument layout (positional, after `--vpn-helper`):
//!   1. oc-path    — path to the openconnect executable
//!   2. url        — VPN gateway URL
//!   3. dsid       — DSID cookie value (without the "DSID=" prefix)
//!   4. parent-pid — PID of the non-elevated parent; helper exits when it dies.
//!      Also used to construct the stop-event name `Local\kuvpn-stop-<pid>`.
//!   5. (optional) `--full-tunnel` — inject 0/1 + 128/1 routes after connect
//!
//! The stop signal is delivered via a named kernel event
//! `Local\kuvpn-stop-<parent-pid>` created by the parent before spawning the
//! helper.  Named events are visible across UAC elevation levels within the
//! same login session, making them far more reliable than temp files whose
//! paths can diverge between non-elevated and elevated processes on Windows 11.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// If the process was invoked with `--vpn-helper` as its first argument, runs
/// the helper and returns the exit code.  Returns `None` for normal startup.
///
/// Call this **before** any argument parser or GUI initialisation so the helper
/// can run silently without touching GUI or CLI machinery.
///
#[cfg(windows)]
pub fn run_vpn_helper_if_requested() -> Option<i32> {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(|s| s.as_str()) != Some("--vpn-helper") {
        return None;
    }

    let oc_path = args.get(2)?;
    let url = args.get(3)?;
    let dsid = args.get(4)?;
    let parent_pid: u32 = args.get(5)?.parse().ok()?;
    let full_tunnel = args.get(6).map(|s| s == "--full-tunnel").unwrap_or(false);

    Some(run_helper(oc_path, url, dsid, parent_pid, full_tunnel))
}

#[cfg(windows)]
fn run_helper(oc_path: &str, url: &str, dsid: &str, parent_pid: u32, full_tunnel: bool) -> i32 {
    use std::io::Write as _;
    use std::os::windows::process::CommandExt;
    use windows::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
    use windows::Win32::System::Threading::{
        OpenEventW, WaitForSingleObject, SYNCHRONIZATION_SYNCHRONIZE,
    };

    // Diagnostic log — written by the elevated helper so we can see exactly what
    // happens even when stdout/stderr are not visible.
    let log_path = std::path::Path::new("C:\\ProgramData\\kuvpn-helper.log");
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .ok();

    macro_rules! hlog {
        ($($arg:tt)*) => {
            if let Some(ref mut f) = log {
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let _ = writeln!(f, "[{}] {}", ts, format!($($arg)*));
                let _ = f.flush();
            }
        };
    }

    hlog!(
        "helper started: oc_path={:?} url={:?} parent_pid={} full_tunnel={}",
        oc_path,
        url,
        parent_pid,
        full_tunnel
    );

    // Open the named event the non-elevated parent created before launching us.
    // "Local\" scopes the event to this login session; both elevation levels
    // share the same session namespace.
    let event_name = format!("Local\\kuvpn-stop-{}", parent_pid);
    hlog!("opening event: {}", event_name);
    let event_name_wide: Vec<u16> = event_name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let stop_event = unsafe {
        match OpenEventW(
            SYNCHRONIZATION_SYNCHRONIZE,
            false,
            windows::core::PCWSTR(event_name_wide.as_ptr()),
        ) {
            Ok(h) => {
                hlog!("OpenEventW OK, handle={:?}", h.0);
                h
            }
            Err(e) => {
                hlog!("OpenEventW FAILED: {}", e);
                eprintln!("vpn-helper: failed to open stop event: {}", e);
                return 1;
            }
        }
    };

    // Start openconnect directly — we're already elevated, so it inherits our
    // token without a second UAC prompt.  CREATE_NO_WINDOW suppresses the console.
    hlog!("spawning openconnect");
    let mut child = match std::process::Command::new(oc_path)
        .creation_flags(CREATE_NO_WINDOW)
        .arg("--protocol")
        .arg("nc")
        .arg("-C")
        .arg(format!("DSID={}", dsid))
        .arg(url)
        .spawn()
    {
        Ok(c) => {
            hlog!("openconnect spawned, child_pid={:?}", c.id());
            c
        }
        Err(e) => {
            hlog!("openconnect spawn FAILED: {}", e);
            eprintln!("vpn-helper: failed to start openconnect: {}", e);
            unsafe {
                let _ = CloseHandle(stop_event);
            }
            return 1;
        }
    };

    // When full tunnel is requested, wait for the TAP adapter on a background
    // thread so the stop-event monitoring loop below starts immediately.
    // The routes are injected as soon as the TAP interface comes up; if the
    // helper is asked to stop first the thread is abandoned (it dies with the
    // process when the helper exits).
    if full_tunnel {
        hlog!("spawning TAP-route background thread (full tunnel)");
        std::thread::spawn(move || {
            if wait_for_tap_interface(30) {
                inject_full_tunnel_routes();
            }
        });
    }

    // Spawn a thread that blocks on the parent process handle.  When the parent
    // exits for any reason (clean exit, crash, kill), WaitForSingleObject returns
    // and we set the flag so the loop below cleans up without a second UAC prompt.
    let parent_died = Arc::new(AtomicBool::new(false));
    let parent_died_clone = Arc::clone(&parent_died);
    std::thread::spawn(move || {
        wait_for_process_exit(parent_pid);
        parent_died_clone.store(true, Ordering::SeqCst);
    });

    hlog!("entering monitor loop");

    // Monitor: stop when signalled by parent (named event, 200 ms blocking wait),
    // when the parent process dies unexpectedly, or when openconnect exits on its own.
    let mut iteration: u64 = 0;
    loop {
        iteration += 1;

        // Block up to 200 ms waiting for the stop event.  Returns WAIT_OBJECT_0
        // immediately when the parent calls SetEvent; WAIT_TIMEOUT after 200 ms
        // if not signalled.  This replaces the previous non-blocking poll +
        // sleep(200 ms) combination and wakes faster on the stop signal.
        let wait_result = unsafe { WaitForSingleObject(stop_event, 200) };
        let signalled = wait_result == WAIT_OBJECT_0;

        if iteration <= 3 || iteration.is_multiple_of(300) {
            hlog!(
                "loop iter={} wait_result={:?} signalled={} parent_died={}",
                iteration,
                wait_result.0,
                signalled,
                parent_died.load(Ordering::SeqCst)
            );
        }

        if signalled {
            hlog!("STOP: named event signalled by parent — killing openconnect");
            let _ = child.kill();
            let _ = child.wait();
            unsafe {
                let _ = CloseHandle(stop_event);
            }
            return 0;
        }

        if parent_died.load(Ordering::SeqCst) {
            hlog!("STOP: parent process died — killing openconnect");
            let _ = child.kill();
            let _ = child.wait();
            unsafe {
                let _ = CloseHandle(stop_event);
            }
            return 0;
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                hlog!(
                    "openconnect exited on its own: success={}",
                    status.success()
                );
                unsafe {
                    let _ = CloseHandle(stop_event);
                }
                return if status.success() { 0 } else { 1 };
            }
            Ok(None) => {}
            Err(e) => {
                hlog!("try_wait error: {}", e);
                eprintln!("vpn-helper: error waiting for child: {}", e);
                unsafe {
                    let _ = CloseHandle(stop_event);
                }
                return 1;
            }
        }
    }
}

/// Blocks until the process with the given PID exits.
/// Uses a kernel wait (zero CPU) so there is no polling overhead.
#[cfg(windows)]
fn wait_for_process_exit(pid: u32) {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE,
    };
    unsafe {
        match OpenProcess(PROCESS_SYNCHRONIZE, false, pid) {
            Ok(handle) if !handle.is_invalid() => {
                WaitForSingleObject(handle, u32::MAX); // INFINITE = 0xFFFFFFFF
                let _ = CloseHandle(handle);
            }
            _ => {} // process already gone — treat as exited
        }
    }
}

/// Polls until a TAP-Windows adapter appears with Status "Up", or until `timeout_secs` elapses.
#[cfg(windows)]
fn wait_for_tap_interface(timeout_secs: u64) -> bool {
    use std::os::windows::process::CommandExt;

    let deadline = std::time::Instant::now() + Duration::from_secs(timeout_secs);
    loop {
        if std::time::Instant::now() >= deadline {
            return false;
        }
        let ready = std::process::Command::new("powershell")
            .creation_flags(CREATE_NO_WINDOW)
            .args([
                "-NoProfile",
                "-Command",
                "(Get-NetAdapter | Where-Object { $_.Status -eq 'Up' -and $_.InterfaceDescription -match 'TAP' }).Count",
            ])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0)
            > 0;

        if ready {
            return true;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

/// Adds 0.0.0.0/1 and 128.0.0.0/1 routes through the first connected TAP adapter.
#[cfg(windows)]
fn inject_full_tunnel_routes() {
    use std::os::windows::process::CommandExt;

    let script = "\
        $a = Get-NetAdapter | Where-Object { $_.Status -eq 'Up' -and $_.InterfaceDescription -match 'TAP' } | Select-Object -First 1; \
        if ($a) { \
            New-NetRoute -DestinationPrefix '0.0.0.0/1'   -InterfaceIndex $a.ifIndex -RouteMetric 1 -ErrorAction SilentlyContinue; \
            New-NetRoute -DestinationPrefix '128.0.0.0/1' -InterfaceIndex $a.ifIndex -RouteMetric 1 -ErrorAction SilentlyContinue \
        }";

    let _ = std::process::Command::new("powershell")
        .creation_flags(CREATE_NO_WINDOW)
        .args(["-NoProfile", "-Command", script])
        .status();
}
