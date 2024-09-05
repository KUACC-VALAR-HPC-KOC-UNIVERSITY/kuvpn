mod args;
mod logger;

use args::Args;
use clap::Parser;
use headless_chrome::{Browser, LaunchOptions};
use logger::init_logger;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

fn main() {
    let args = Args::parse();

    init_logger(&args.level);

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

    // Launch the browser with the specified user data directory.
    let browser = Browser::new(
        LaunchOptions::default_builder()
            .headless(false)
            .sandbox(false)
            .args(vec![
                OsString::from("--app=data:text/html,<html><body></body></html>").as_os_str(),
                OsString::from("----new-window").as_os_str(),
            ]) // Converts &str to OsStr
            .user_data_dir(Some(user_data_dir)) // Set the .config/kuvpn directory
            .build()
            .expect("Could not find chrome-executable"),
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
