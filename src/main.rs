mod args;
mod client;
mod driver;
mod login;
mod utils;
mod wait;

use args::Args;
use clap::Parser;
use driver::{Driver, DriverError};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), DriverError> {
    // Check if chromedriver & openconnect are installed
    let chromedriver_installed = std::process::Command::new("which")
        .arg("chromedriver")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    let openconnect_installed = std::process::Command::new("which")
        .arg("openconnect")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if !chromedriver_installed || !openconnect_installed {
        eprintln!("Error: Required dependencies are not installed.");
        if !chromedriver_installed {
            eprintln!("Please install chromedriver.");
        }
        if !openconnect_installed {
            eprintln!("Please install openconnect.");
        }
        std::process::exit(1);
    }

    let mut args = Args::parse();

    let mut driver = Driver::start(&mut args.port).await?;

    // Wait for the WebDriver to be reachable
    driver
        .wait_for_webdriver(args.port, Duration::from_secs(30))
        .await?;

    let mut attempt_count = 0;
    loop {
        match client::run_client(args.url.clone(), args.port).await {
            Ok(_) => break,
            Err(err) => match err {
                DriverError::WebDriverConnectionError(e) => {
                    println!("WebDriverConnectionError encountered: {}. Retrying...", e);
                    attempt_count += 1;
                    if attempt_count > 3 {
                        println!("Exceeded maximum retry attempts for WebDriver connection.");
                        return Err(DriverError::WebDriverConnectionError(e));
                    }
                    sleep(Duration::from_secs(2)).await; // Delay before retrying
                }
                DriverError::WebDriverClientError(e) => {
                    println!(
                        "WebDriverClientError encountered: {}. Restarting driver...",
                        e
                    );
                    drop(driver); // Drop the current driver to kill the process
                    driver = Driver::start(&mut args.port).await?; // Restart driver
                    attempt_count = 0; // Reset attempt count after restarting
                }
                DriverError::ProcessStartError(e) => {
                    println!(
                        "ProcessStartError encountered: {}. Stopping application.",
                        e
                    );
                    return Err(DriverError::ProcessStartError(e)); // Exit on process start error
                }
                DriverError::WebDriverStartTimeout => {
                    return Err(DriverError::WebDriverStartTimeout);
                }
            },
        }
    }

    Ok(())
}
