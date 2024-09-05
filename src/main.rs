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

    match browser_website(&args.url) {
        Ok(dsid) => println!("{dsid}"),
        Err(_) => todo!(),
    }
}

fn browser_website(url: &str) -> Result<String, Box<dyn Error>> {
    // Define the user data directory within the user's .config directory.
    let home_dir = env::var("HOME")?;
    let user_data_dir = PathBuf::from(format!("{}/.config/kuvpn", home_dir));

    // Ensure the directory exists
    if !user_data_dir.exists() {
        fs::create_dir_all(&user_data_dir)?;
    }

    let body = OsString::from("--app=data:text/html,<html><body></body></html>");
    let window = OsString::from("--new-window");
    let mut options = LaunchOptions::default_builder();
    let mut launch_options = options
        .headless(false)
        .sandbox(false)
        .args(vec![body.as_os_str(), window.as_os_str()]) // Converts &str to OsStr
        .user_data_dir(Some(user_data_dir)); // Set the .config/kuvpn directory

    // Check if default_executable exists and set path if found
    if let Ok(executable_path) = default_executable() {
        launch_options = launch_options.path(Some(executable_path));
    }

    // Build the browser
    let browser = Browser::new(
        launch_options
            .build()
            .expect("Could not find chrome-executable"), // Should be able to install chrome
    )?;

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
