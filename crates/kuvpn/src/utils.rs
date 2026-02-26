use console::{Style, Term};
use dialoguer::{Input, Password};
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

/// Trait for providing credentials and user input.
pub trait CredentialsProvider: Send + Sync {
    fn request_text(&self, msg: &str) -> String;
    fn request_password(&self, msg: &str) -> String;
    fn on_mfa_push(&self, _code: &str) {}
    fn on_mfa_complete(&self) {}
}

/// A implementation of `CredentialsProvider` for terminal input.
pub struct TerminalCredentialsProvider;

impl CredentialsProvider for TerminalCredentialsProvider {
    fn request_text(&self, msg: &str) -> String {
        // Clear the current line in case a spinner is active on another thread
        let term = Term::stderr();
        let _ = term.clear_line();
        Input::new()
            .with_prompt(msg.trim_end_matches(": ").trim_end_matches(':'))
            .interact_text()
            .unwrap_or_default()
    }

    fn request_password(&self, msg: &str) -> String {
        // Clear the current line in case a spinner is active on another thread
        let term = Term::stderr();
        let _ = term.clear_line();
        Password::new()
            .with_prompt(msg.trim_end_matches(": ").trim_end_matches(':'))
            .interact()
            .unwrap_or_default()
    }

    fn on_mfa_push(&self, code: &str) {
        let bold = Style::new().bold();
        let cyan = Style::new().cyan().bold();
        eprintln!();
        eprintln!(
            "{} Enter {} in Authenticator",
            bold.apply_to(">>"),
            cyan.apply_to(code),
        );
        eprintln!();
    }

    fn on_mfa_complete(&self) {
        let green = Style::new().green();
        eprintln!("  {} MFA approved", green.apply_to("✓"));
    }
}

use once_cell::sync::Lazy;
use std::fs::File;
use std::sync::Mutex;

static INSTANCE_LOCK: Lazy<Mutex<Option<File>>> = Lazy::new(|| Mutex::new(None));

/// Ensures that only one instance of the application is running.
/// Returns an error if another instance is already active.
pub fn ensure_single_instance() -> Result<(), Box<dyn Error>> {
    let mut lock_path = get_user_data_dir()?;
    lock_path.pop(); // Go to parent of 'profile' (~/.local/share/kuvpn/)
    std::fs::create_dir_all(&lock_path)?;
    lock_path.push("kuvpn.lock");

    let file = File::create(&lock_path)?;

    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        let fd = file.as_raw_fd();
        let res = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
        if res != 0 {
            return Err("Another instance of KUVPN is already running.".into());
        }
    }

    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        use windows::Win32::Foundation::HANDLE;
        use windows::Win32::Storage::FileSystem::{
            LockFileEx, LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY,
        };
        use windows::Win32::System::IO::OVERLAPPED;

        let handle = file.as_raw_handle();
        let mut overlapped = OVERLAPPED::default();
        let res = unsafe {
            LockFileEx(
                HANDLE(handle as _),
                LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
                None,
                1,
                0,
                &mut overlapped,
            )
        };
        if res.is_err() {
            return Err("Another instance of KUVPN is already running.".into());
        }
    }

    let mut guard = INSTANCE_LOCK.lock().unwrap();
    *guard = Some(file);

    Ok(())
}

/// Returns a platform-appropriate user data directory for the Chrome instance.
///
/// The directory path is constructed based on the operating system:
/// - **Linux:** `~/.local/share/kuvpn/profile`
/// - **macOS:** `~/Library/Application Support/kuvpn/profile`
/// - **Windows:** `%USERPROFILE%\AppData\Roaming\kuvpn\profile`
///
/// If the directory does not exist, it is created.
///
/// # Errors
///
/// Returns an error if the home directory cannot be determined or if the directory cannot be created.
pub fn get_user_data_dir() -> Result<PathBuf, Box<dyn Error>> {
    // Determine the user's home directory from environment variables.
    let home_dir = env::var("HOME").or_else(|_| env::var("USERPROFILE"))?;

    // Select the appropriate base path for the current operating system.
    #[cfg(target_os = "linux")]
    let base_path = ".local/share/kuvpn/profile";

    #[cfg(target_os = "macos")]
    let base_path = "Library/Application Support/kuvpn/profile";

    #[cfg(target_os = "windows")]
    let base_path = "AppData/Roaming/kuvpn/profile";

    // Construct the full user data directory path.
    let user_data_dir = PathBuf::from(format!("{}/{}", home_dir, base_path));

    // Create the directory if it does not exist.
    if !user_data_dir.exists() {
        std::fs::create_dir_all(&user_data_dir)?;
        log::info!("User data directory created at: {:?}", user_data_dir);
    }

    Ok(user_data_dir)
}

/// Completely removes the user data directory
pub fn wipe_user_data_dir() -> Result<(), Box<dyn Error>> {
    let path = get_user_data_dir()?;
    if path.exists() {
        std::fs::remove_dir_all(&path)?;
        log::info!("Wiped profile directory: {:?}", path);
    }
    Ok(())
}

/// Escapes JavaScript strings to prevent injection.
pub fn js_escape(s: &str) -> String {
    s.replace("\\", "\\\\").replace("'", "\\'")
}
