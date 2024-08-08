use clap::{Parser, ValueEnum};
use fantoccini::ClientBuilder;
use std::net::TcpListener;
use std::process::Stdio;
use thiserror::Error;
use tokio::process::{Child, Command};
use tokio::time::{sleep, Duration};

#[derive(ValueEnum, Clone, Debug)]
enum Browser {
    Chrome,
    Gecko,
    None,
}

struct Driver {
    process: Option<Child>,
}

impl Drop for Driver {
    fn drop(&mut self) {
        if let Some(ref mut process) = self.process {
            let _ = process.start_kill();
            let _ = process.try_wait();
        }   
    }
}

#[derive(Error, Debug)]
enum DriverError {
    #[error("Failed to start WebDriver process: {0}")]
    ProcessStartError(#[from] std::io::Error),
    #[error("Failed to connect to WebDriver: {0}")]
    WebDriverConnectionError(#[from] fantoccini::error::CmdError),
    #[error("Failed to build client {0}")]
    WebDriverClientError(#[from] fantoccini::error::NewSessionError),
}

impl Driver {
    async fn start(browser: Browser, port: u16) -> Result<Driver, DriverError> {
        let port = Driver::find_available_port(port).await;
        println!("Using port: {}", port);

        match browser {
            Browser::Chrome => {
                let process = Command::new("chromedriver")
                    .arg(format!("--port={}", port))
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .map_err(DriverError::ProcessStartError)?;
                sleep(Duration::from_secs(2)).await;
                Ok(Driver {
                    process: Some(process),
                })
            }
            Browser::Gecko => {
                let process = Command::new("geckodriver")
                    .arg(format!("--port={}", port))
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .map_err(DriverError::ProcessStartError)?;
                sleep(Duration::from_secs(2)).await;
                Ok(Driver {
                    process: Some(process),
                })
            }
            Browser::None => Ok(Driver { process: None }),
        }
    }

    async fn find_available_port(mut port: u16) -> u16 {
        loop {
            match TcpListener::bind(("127.0.0.1", port)) {
                Ok(listener) => {
                    drop(listener);
                    break port;
                }
                Err(_) => {
                    println!(
                        "Webdriver port {} is in use, trying next port: {}",
                        port,
                        port.wrapping_add(1)
                    );
                    port = port.wrapping_add(1);
                }
            }
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "https://vpn.ku.edu.tr")]
    url: String,

    #[arg(short, long, value_enum, default_value_t = Browser::Chrome)]
    browser: Browser,

    #[arg(short, long, default_value_t = 9515)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), DriverError> {
    let args = Args::parse();

    let driver = Driver::start(args.browser.clone(), args.port).await?;

    let result = run_client(args.url, args.port).await;

    drop(driver);

    result
}

async fn run_client(url: String, port: u16) -> Result<(), DriverError> {
    let c = ClientBuilder::native()
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
