mod args;
mod logger;

use args::Args;
use clap::Parser;
use headless_chrome::browser::default_executable;
use headless_chrome::{Browser, LaunchOptions};
use log::{error, info};
use logger::init_logger;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

// Function to get the user data directory based on the operating system
fn get_user_data_dir() -> Result<PathBuf, Box<dyn Error>> {
    let home_dir = env::var("HOME").or_else(|_| env::var("USERPROFILE"))?;

    #[cfg(target_os = "linux")]
    let base_path = ".local/share/kuvpn/profile";

    #[cfg(target_os = "macos")]
    let base_path = "Library/Application Support/kuvpn/profile";

    #[cfg(target_os = "windows")]
    let base_path = "AppData/Roaming/kuvpn/profile";

    let user_data_dir = PathBuf::from(format!("{}/{}", home_dir, base_path));

    if !user_data_dir.exists() {
        fs::create_dir_all(&user_data_dir)?;
        info!("User data directory created at: {:?}", user_data_dir);
    }

    Ok(user_data_dir)
}

fn main() -> ExitCode {
    let args = Args::parse();

    init_logger(&args.level);

    info!("Parsed arguments: {:?}", args);

    if args.clean {
        // Use the get_user_data_dir function to obtain the correct path
        let user_data_dir = get_user_data_dir().expect("Unable to obtain user data directory");

        info!("Cleaning user data directory: {:?}", user_data_dir);

        if user_data_dir.exists() {
            match std::fs::remove_dir_all(&user_data_dir) {
                Ok(_) => {
                    info!("Session information successfully removed.");
                    return ExitCode::SUCCESS;
                }
                Err(e) => {
                    error!("Failed to remove session information: {}", e);
                    return ExitCode::FAILURE;
                }
            }
        } else {
            info!("No session information found.");
            return ExitCode::FAILURE;
        }
    }

    if !args.dsid {
        let openconnect_installed = std::process::Command::new("which")
            .arg("openconnect")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
        if !(openconnect_installed) {
            log::error!("Please install openconnect.");
            return ExitCode::FAILURE;
        }
    }

    // Create the browser
    info!("Creating browser with agent: {}", args.agent);

    let browser = match create_browser(&args.agent) {
        Ok(browser) => browser,
        Err(e) => {
            error!("Failed to create browser: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Fetch the DSID using the browser
    info!("Fetching DSID from URL: {}", args.url);

    let dsid = match fetch_dsid(&args.url, &browser) {
        Ok(dsid) => dsid,
        Err(e) => {
            error!("Error: {}", e);
            return ExitCode::FAILURE;
        }
    };

    if args.dsid {
        info!("DSID retrieved: {}", dsid);
        println!("{dsid}");
        return ExitCode::SUCCESS;
    }

    if let Err(e) = execute_openconnect(dsid, args.url) {
        error!("Error executing openconnect: {}", e);
        return ExitCode::FAILURE;
    }
    return ExitCode::SUCCESS;
}

// New function to create the browser
fn create_browser(agent: &str) -> Result<Browser, Box<dyn Error>> {
    // Use the get_user_data_dir function to obtain the correct path
    let user_data_dir = get_user_data_dir()?;

    let user_agent = OsString::from(format!("--user-agent={agent}"));
    let body = OsString::from("--app=data:text/html,<html><body></body></html>");
    let window = OsString::from("--new-window");

    let mut options = LaunchOptions::default_builder();
    let mut launch_options = options
        .headless(false)
        .sandbox(false)
        .idle_browser_timeout(Duration::MAX)
        .args(vec![
            body.as_os_str(),
            window.as_os_str(),
            user_agent.as_os_str(),
        ])
        .user_data_dir(Some(user_data_dir));

    if let Ok(executable_path) = default_executable() {
        launch_options = launch_options.path(Some(executable_path));
    }

    Ok(Browser::new(launch_options.build()?)?)
}

fn fetch_dsid(url: &str, browser: &Browser) -> Result<String, Box<dyn Error>> {
    #[allow(deprecated)]
    let tab = browser.wait_for_initial_tab()?;

    tab.navigate_to(url)?;
    tab.wait_until_navigated()?;

    info!("Navigating to URL: {}", url);

    loop {
        let script =
            "document.cookie.split('; ').find(row => row.startsWith('DSID='))?.split('=')[1];";
        let remote_object = tab.evaluate(script, true)?;

        if let Some(dsid_value) = remote_object.value {
            if let Some(dsid_string) = dsid_value.as_str() {
                tab.close_with_unload().expect("failed to close");
                info!("DSID value found: {}", dsid_string);
                return Ok(dsid_string.to_string());
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}

pub fn execute_openconnect(cookie_value: String, url: String) -> Result<(), Box<dyn Error>> {
    let openconnect_command = format!(
        "sudo openconnect --protocol nc -C 'DSID={}' {}",
        cookie_value, url
    );

    println!("Running openconnect with sudo");

    use std::process::Command as StdCommand;
    StdCommand::new("sh")
        .arg("-c")
        .arg(&openconnect_command)
        .status()?;

    Ok(())
}
