use std::net::TcpListener;
use std::process::Stdio;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio::process::{Child, Command};
use tokio::time::{sleep, timeout, Duration};

pub struct Driver {
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
pub enum DriverError {
    #[error("Failed to start WebDriver process: {0}")]
    ProcessStartError(#[from] std::io::Error),
    #[error("Failed to connect to WebDriver: {0}")]
    WebDriverConnectionError(#[from] fantoccini::error::CmdError),
    #[error("Failed to build client {0}")]
    WebDriverClientError(#[from] fantoccini::error::NewSessionError),
    #[error("WebDriver failed to start within the timeout period")]
    WebDriverStartTimeout,
}

impl Driver {
    pub async fn start(port: &mut u16) -> Result<Driver, DriverError> {
        Driver::find_available_port(*port).await;
        log::info!("Using port: {}", *port);

        let process = Command::new("chromedriver")
            .arg(format!("--port={}", port))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(DriverError::ProcessStartError)?;

        Ok(Driver {
            process: Some(process),
        })
    }

    async fn find_available_port(mut port: u16) -> u16 {
        loop {
            match TcpListener::bind(("127.0.0.1", port)) {
                Ok(listener) => {
                    drop(listener);
                    break port;
                }
                Err(_) => {
                    log::info!(
                        "Webdriver port {} is in use, trying next port: {}",
                        port,
                        port.wrapping_add(1)
                    );
                    port = port.wrapping_add(1);
                }
            }
        }
    }

    pub async fn wait_for_webdriver(
        &self,
        port: u16,
        timeout_duration: Duration,
    ) -> Result<(), DriverError> {
        let address = format!("127.0.0.1:{}", port);
        match timeout(timeout_duration, async {
            loop {
                if TcpStream::connect(&address).await.is_ok() {
                    log::info!("WebDriver is up on port: {}.", port);
                    return Ok(());
                }
                sleep(Duration::from_millis(100)).await;
            }
        })
        .await
        {
            Ok(result) => result,
            Err(_) => Err(DriverError::WebDriverStartTimeout),
        }
    }
}
