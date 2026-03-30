use headless_chrome::browser::default_executable;
use headless_chrome::{Browser, LaunchOptions};
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::time::Duration;

/// Creates a browser instance configured with a blank page, a custom user agent,
/// and a dedicated user data directory.
///
/// The browser is launched with the following settings:
/// - **Headless or non-headless mode** based on the `headless` parameter.
/// - **A blank app window** containing a minimal HTML page.
/// - **Custom user agent** as specified by the `agent` parameter.
/// - **Custom user data directory** for isolated session data.
/// - **Custom window size** of 800x800 pixels.
/// - **Sandbox disabled** and an appropriate idle timeout based on mode.
///
/// # Arguments
///
/// * `agent` - A string slice representing the desired user agent.
/// * `headless` - Whether to run the browser in headless mode.
/// * `manual_mode` - Whether manual user interaction is expected (use longer timeout).
///
/// # Returns
///
/// A `Result` containing the configured `Browser` instance on success, or an error on failure.
pub fn create_browser(
    agent: &str,
    headless: bool,
    manual_mode: bool,
) -> Result<Browser, Box<dyn Error>> {
    let user_agent = OsString::from(format!("--user-agent={agent}"));

    let mut attempts = 0;
    loop {
        let user_data_dir = crate::utils::get_user_data_dir()?;

        // Timeout based on mode:
        // - Headless (auto): 2 minutes - the flow is fully automated, shouldn't take long
        // - Manual mode: 10 minutes - user might interact slowly or be away from screen
        let idle_timeout = if manual_mode {
            Duration::from_secs(600) // 10 minutes for manual modes
        } else {
            Duration::from_secs(120) // 2 minutes for headless mode
        };
        log::info!(
            "Browser idle timeout: {}s ({})",
            idle_timeout.as_secs(),
            if manual_mode { "manual mode" } else { "headless mode" }
        );

        // --new-window on Windows causes Chrome to open a window that is not
        // attached to the CDP session; headless_chrome then controls a blank
        // internal tab while the visible window is unreachable.
        let mut args: Vec<&OsStr> = vec![
            user_agent.as_os_str(),
            OsStr::new("--no-first-run"),
            OsStr::new("--no-default-browser-check"),
            OsStr::new("--disable-session-crashed-bubble"),
            OsStr::new("--lang=en-US"),
        ];
        #[cfg(not(target_os = "windows"))]
        args.push(OsStr::new("--new-window"));

        let mut options = LaunchOptions::default_builder();
        let mut launch_options = options
            .headless(headless)
            .sandbox(false)
            .idle_browser_timeout(idle_timeout)
            .window_size(Some((800, 800)))
            .enable_gpu(false)
            .args(args)
            .user_data_dir(Some(user_data_dir));

        if let Ok(path) = std::env::var("KUVPN_CHROME_PATH") {
            launch_options = launch_options.path(Some(path.into()));
        } else if let Ok(executable_path) = default_executable() {
            launch_options = launch_options.path(Some(executable_path));
        }

        match Browser::new(launch_options.build()?) {
            Ok(browser) => return Ok(browser),
            Err(e) => {
                attempts += 1;
                if attempts >= 2 {
                    return Err(format!("Browser failed even after wipe: {}", e).into());
                }
                log::warn!("Browser connection failed. Wiping profile and retrying...");
                crate::utils::wipe_user_data_dir()?;
            }
        }
    }
}
