use clap::{Parser, ValueEnum};
use fantoccini::ClientBuilder;
use std::net::TcpListener;
use std::process::Stdio;
use thiserror::Error;
use tokio::process::Command;
use tokio::time::{sleep, Duration};

#[derive(ValueEnum, Clone, Debug)]
enum Browser {
    Chrome,
    Gecko,
    None,
}

struct Driver {
    process: Option<tokio::process::Child>,
}

impl Drop for Driver {
    fn drop(&mut self) {
        if let Some(ref mut process) = self.process {
            let _ = process.kill();
        }
    }
}

#[derive(Error, Debug)]
enum DriverError {
    #[error("Failed to start WebDriver process: {0}")]
    ProcessStartError(#[from] std::io::Error),
    #[error("Port {0} is already in use")]
    PortInUse(u16),
    #[error("Failed to connect to WebDriver: {0}")]
    WebDriverConnectionError(#[from] fantoccini::error::CmdError),
    #[error("Failed to build client {0}")]
    WebDriverClientError(#[from] fantoccini::error::NewSessionError),
}

async fn start_driver(browser: Browser, port: u16) -> Result<Driver, DriverError> {
    // Check if the port is available
    match TcpListener::bind(("127.0.0.1", port)) {
        Ok(listener) => {
            drop(listener); // Close the listener immediately as we only need to check the port availability
        }
        Err(_) => {
            return Err(DriverError::PortInUse(port));
        }
    }

    match browser {
        Browser::Chrome => {
            let process = Command::new("chromedriver")
                .arg(format!("--port={}", port))
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            sleep(Duration::from_secs(2)).await;
            Ok(Driver { process: Some(process) })
        },
        Browser::Gecko => {
            let process = Command::new("geckodriver")
                .arg(format!("--port={}", port))
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            sleep(Duration::from_secs(2)).await;
            Ok(Driver { process: Some(process) })
        },
        Browser::None => Ok(Driver { process: None }),
    }
}

/// Simple program to retrieve DSID cookie and execute OpenConnect command
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// URL to visit
    #[arg(short, long, default_value = "https://vpn.ku.edu.tr")]
    url: String,

    /// Browser to use
    #[arg(short, long, value_enum, default_value_t = Browser::Chrome)]
    browser: Browser,

    /// Port to use for WebDriver
    #[arg(short, long, default_value_t = 9515)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), DriverError> {
    let args = Args::parse();

    let _driver = start_driver(args.browser.clone(), args.port).await?;

    let c = ClientBuilder::native()
        .connect(&format!("http://localhost:{}", args.port))
        .await?;

    // Go to the specified URL
    c.goto(&args.url).await?;

    loop {
        // Execute JavaScript to get the DSID cookie
        let script = "return document.cookie.split('; ').find(row => row.startsWith('DSID='))?.split('=')[1];";
        let result = c.execute(script, vec![]).await?;

        // Extract the DSID cookie from the result
        let dsid_cookie = result.as_str().map(|s| s.to_string());

        if let Some(cookie_value) = dsid_cookie {
            c.close().await?;

            println!("DSID cookie found: {}", cookie_value);

            // Construct the OpenConnect command
            let openconnect_command = format!(
                "sudo openconnect --protocol nc -C 'DSID={}' vpn.ku.edu.tr",
                cookie_value
            );
            println!("Command to execute: {}", openconnect_command);

            // Optionally execute the command using std::process::Command
            use std::process::Command as StdCommand;
            StdCommand::new("sh")
                .arg("-c")
                .arg(openconnect_command)
                .status()?;
            break;
        }
    }

    Ok(())
}
