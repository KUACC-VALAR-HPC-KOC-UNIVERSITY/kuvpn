mod args;

use args::Args;
use clap::Parser;
use headless_chrome::browser::default_executable;
use headless_chrome::{Browser, LaunchOptions};
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

fn main() {
    let args = Args::parse();

    let dsid = match fetch_dsid(&args.url) {
        Ok(dsid) => dsid,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };

    if args.dsid {
        println!("{dsid}");
        return;
    }

    if args.clean {
        let home_dir = env::var("HOME").expect("Unable to obtain home-folder");
        let user_data_dir = PathBuf::from(format!("{}/.config/kuvpn", home_dir));

        // Check if the directory exists
        if user_data_dir.exists() {
            // Remove the directory and its contents
            match std::fs::remove_dir_all(&user_data_dir) {
                Ok(_) => {
                    println!("Session information successfully removed.");
                }
                Err(e) => {
                    eprintln!("Failed to remove session information: {}", e);
                }
            }
        } else {
            println!("No session information found.");
        }

        return;
    }

    // Attempt to execute openconnect and handle any errors
    if let Err(e) = execute_openconnect(dsid, args.url) {
        eprintln!("Error executing openconnect: {}", e);
    }
}

pub fn execute_openconnect(cookie_value: String, url: String) -> Result<(), Box<dyn Error>> {
    let openconnect_command = format!(
        "sudo openconnect --protocol nc -C 'DSID={}' {}",
        cookie_value, url
    );

    println!("Running openconnect with sudo");

    // Use std::process::Command to execute openconnect
    use std::process::Command as StdCommand;
    // Spawn the openconnect process in the background
    StdCommand::new("sh")
        .arg("-c")
        .arg(&openconnect_command)
        .status()?;

    Ok(())
}

fn fetch_dsid(url: &str) -> Result<String, Box<dyn Error>> {
    // Define the user data directory within the user's .config directory.
    let home_dir = env::var("HOME")?;
    let user_data_dir = PathBuf::from(format!("{}/.config/kuvpn", home_dir));

    // Ensure the directory exists
    if !user_data_dir.exists() {
        fs::create_dir_all(&user_data_dir)?;
    }

    let user_agent = OsString::from("--user-agent=Mozilla/5.0 (iPhone; CPU iPhone OS 13_2_3 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/13.0.3 Mobile/15E148 Safari/604.1");
    let body = OsString::from("--app=data:text/html,<html><body></body></html>"); // Empty body for --app to work.
    let window = OsString::from("--new-window");

    let mut options = LaunchOptions::default_builder();
    let mut launch_options = options
        .headless(false)
        .sandbox(false)
        .args(vec![
            body.as_os_str(),
            window.as_os_str(),
            user_agent.as_os_str(), // used to skip hostchecker
        ])
        .user_data_dir(Some(user_data_dir)); // Set the .config/kuvpn directory

    // Check if default_executable exists and set path if found
    if let Ok(executable_path) = default_executable() {
        launch_options = launch_options.path(Some(executable_path));
    }

    // Build the browser
    let browser = Browser::new(launch_options.build()?)?;

    // Wait for the browser to launch and the tab to load
    #[allow(deprecated)]
    let tab = browser.wait_for_initial_tab()?;

    // Navigate to the given URL and wait for the page to load completely
    tab.navigate_to(url)?;
    tab.wait_until_navigated()?;

    // Run in a loop to continuously check for the DSID or storage item.
    loop {
        // Attempt to get the 'DSID' cookie directly.
        let script =
            "document.cookie.split('; ').find(row => row.startsWith('DSID='))?.split('=')[1];";
        let remote_object = tab.evaluate(script, true)?;

        if let Some(dsid_value) = remote_object.value {
            if let Some(dsid_string) = dsid_value.as_str() {
                return Ok(dsid_string.to_string());
            }
        }

        // Sleep for a specified duration before running the script again.
        thread::sleep(Duration::from_millis(100));
    }
}
