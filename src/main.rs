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
use std::process::{Command, ExitCode};
use std::thread;
use std::time::Duration;
use which::which;

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

    if !args.dsid && which("openconnect").is_err() {
        log::error!("Please install openconnect.");
        return ExitCode::FAILURE;
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

    if let Err(e) = execute_openconnect(dsid, args.url, &args.run_command) {
        error!("Error executing openconnect: {}", e);
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
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
        .window_size(Some((800, 800)))
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

pub fn execute_openconnect(
    cookie_value: String,
    url: String,
    run_command: &Option<String>,
) -> Result<(), Box<dyn Error>> {
    // Default list of privilege escalation tools
    let mut default_tools = vec!["doas", "sudo", "pkexec"];

    // If a custom run command is provided, check if it's available and prioritize it
    if let Some(custom_command) = run_command {
        info!("Custom run command provided: {}", custom_command);

        if which(custom_command).is_ok() {
            info!("Custom command found: {}", custom_command);
            default_tools.insert(0, custom_command.as_str());
        } else {
            // Print message and fallback to default tools
            println!(
                "Custom command '{}' not found, falling back to default tools.",
                custom_command
            );
            info!(
                "Custom command '{}' could not be found, using default tools.",
                custom_command
            );
        }
    } else {
        info!("No custom run command provided, defaulting to built-in tools.");
    }

    // Log the list of commands/tools being checked
    info!("Checking for available tools/commands: {:?}", default_tools);

    // Check for the first available tool/command
    let command_to_run = default_tools
        .iter()
        .find_map(|&tool| {
            if which(tool).is_ok() {
                Some(tool)
            } else {
                None
            }
        })
        .ok_or("No available command for running openconnect")?;

    // Print user-facing message
    println!(
        "Running openconnect using {} for elevated privileges or execution",
        command_to_run
    );

    // Log detailed arguments for the command
    info!(
        "Command ({}) arguments: openconnect --protocol nc -C DSID={} {}",
        command_to_run, cookie_value, url
    );

    Command::new(command_to_run)
        .arg("openconnect")
        .arg("--protocol")
        .arg("nc")
        .arg("-C")
        .arg(format!("DSID={}", cookie_value))
        .arg(url)
        .status()?;

    Ok(())
}
