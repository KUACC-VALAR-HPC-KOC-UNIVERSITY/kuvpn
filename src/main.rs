mod args;
mod driver;

use args::Args;
use clap::Parser;
use driver::{Driver, DriverError};
use fantoccini::ClientBuilder;
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), DriverError> {
    let mut args = Args::parse();

    let mut driver = Driver::start(args.browser.clone(), &mut args.port).await?;

    // Wait until the WebDriver is reachable
    let address = format!("127.0.0.1:{}", args.port);
    loop {
        match TcpStream::connect(&address).await {
            Ok(_) => {
                println!("Webdriver is up on: {}.", args.port);
                break;
            }
            Err(_) => {
                sleep(Duration::from_millis(100)).await;
            }
        }
    }

    let mut attempt_count = 0;
    loop {
        match run_client(args.url.clone(), args.port).await {
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
                    driver = Driver::start(args.browser.clone(), &mut args.port).await?; // Restart driver
                    attempt_count = 0; // Reset attempt count after restarting
                }
                DriverError::ProcessStartError(e) => {
                    println!(
                        "ProcessStartError encountered: {}. Stopping application.",
                        e
                    );
                    return Err(DriverError::ProcessStartError(e)); // Exit on process start error
                }
            },
        }
    }

    Ok(())
}

async fn run_client(url: String, port: u16) -> Result<(), DriverError> {
    let c = ClientBuilder::rustls()?
        .connect(&format!("http://localhost:{}", port))
        .await
        .map_err(DriverError::WebDriverClientError)?;

    c.goto(&url).await?;

    loop {
        let script = "return document.cookie.split('; ').find(row => row.startsWith('DSID='))?.split('=')[1];";
        let result = c.execute(script, vec![]).await?;

        let dsid_cookie = result.as_str().map(|s| s.to_string());

        if let Some(cookie_value) = dsid_cookie {
            c.close().await?;

            println!("DSID cookie found: {}", cookie_value);

            let openconnect_command = format!(
                "sudo openconnect --protocol nc -C 'DSID={}' vpn.ku.edu.tr",
                cookie_value
            );
            println!("Executing: {}", openconnect_command);

            use std::process::Command as StdCommand;
            StdCommand::new("sh")
                .arg("-c")
                .arg(openconnect_command)
                .status()
                .map_err(DriverError::ProcessStartError)?;
            break;
        }
    }

    Ok(())
}
