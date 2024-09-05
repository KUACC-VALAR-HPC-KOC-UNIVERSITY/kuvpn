mod args;
mod logger;

use args::Args;
use clap::Parser;
use headless_chrome::browser::default_executable;
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

    if args.clean {
        let home_dir = env::var("HOME").expect("Unable to obtain home-folder");
        let user_data_dir = PathBuf::from(format!("{}/.config/kuvpn", home_dir));

        if user_data_dir.exists() {
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

    // Create the browser
    let browser = match create_browser(&args.agent) {
        Ok(browser) => browser,
        Err(e) => {
            eprintln!("Failed to create browser: {}", e);
            return;
        }
    };

    // Fetch the DSID using the browser
    let dsid = match fetch_dsid(&args.url, &browser) {
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

    if let Err(e) = execute_openconnect(dsid, args.url) {
        eprintln!("Error executing openconnect: {}", e);
    }
}

// New function to create the browser
fn create_browser(agent: &str) -> Result<Browser, Box<dyn Error>> {
    let home_dir = env::var("HOME")?;
    let user_data_dir = PathBuf::from(format!("{}/.config/kuvpn/profile", home_dir));

    if !user_data_dir.exists() {
        fs::create_dir_all(&user_data_dir)?;
    }

    let user_agent = OsString::from(format!("--user-agent={agent}"));
    let body = OsString::from("--app=data:text/html,<html><body></body></html>");
    let window = OsString::from("--new-window");

    let mut options = LaunchOptions::default_builder();
    let mut launch_options = options
        .headless(false)
        .sandbox(false)
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

    loop {
        let script =
            "document.cookie.split('; ').find(row => row.startsWith('DSID='))?.split('=')[1];";
        let remote_object = tab.evaluate(script, true)?;

        if let Some(dsid_value) = remote_object.value {
            if let Some(dsid_string) = dsid_value.as_str() {
                tab.close_with_unload().expect("failed to close");
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
