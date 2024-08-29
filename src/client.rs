use crate::args::Args;
use crate::driver::{Driver, DriverError};
use crate::utils::{execute_openconnect, get_dsid_cookie, skip_host_checker};
use fantoccini::ClientBuilder;
use serde_json;
use tokio::time::{sleep, Duration};

pub async fn run_client(url: String, port: u16) -> Result<(), DriverError> {
    let client = setup_client(port).await?;
    client.goto(&url).await?;

    skip_host_checker(&client).await;
    log::debug!("Skipped host checker");

    if let Some(cookie_value) = get_dsid_cookie_with_retries(&client).await? {
        client.close().await?;
        log::info!("DSID cookie found: {}", cookie_value);
        execute_openconnect(cookie_value)?;
        return Ok(());
    }

    Ok(())
}

const CHROME_PROFILE_RELATIVE_PATH: &str = ".config/kuvpn/profile";
async fn setup_client(port: u16) -> Result<fantoccini::Client, DriverError> {
    let home_dir = std::env::var("HOME").expect("Could not find home directory");
    let chrome_profile_path = std::path::Path::new(&home_dir).join(CHROME_PROFILE_RELATIVE_PATH);
    let chrome_profile_path_str = chrome_profile_path.to_str().expect("Invalid path");

    let mut capabilities = serde_json::Map::new();

    let mut chrome_options = serde_json::Map::new();
    chrome_options.insert(
        "args".to_string(),
        serde_json::Value::Array(vec![
            serde_json::Value::String(
                "--app=data:text/html,<html><body></body></html>".to_string(),
            ),
            serde_json::Value::String(format!("--user-data-dir={}", chrome_profile_path_str)),
        ]),
    );
    capabilities.insert(
        "goog:chromeOptions".to_string(),
        serde_json::Value::Object(chrome_options),
    );

    // Set the page load strategy
    capabilities.insert(
        "pageLoadStrategy".to_string(),
        serde_json::Value::String("normal".to_string()),
    );

    let client = ClientBuilder::rustls()?
        .capabilities(capabilities)
        .connect(&format!("http://localhost:{}", port))
        .await
        .map_err(DriverError::WebDriverClientError)?;

    log::debug!("Started brwoser");

    Ok(client)
}

async fn get_dsid_cookie_with_retries(
    client: &fantoccini::Client,
) -> Result<Option<String>, DriverError> {
    let start_time = std::time::Instant::now();
    log::debug!("Starting to look for dsid...");
    while start_time.elapsed() < Duration::MAX {
        if let Some(value) = get_dsid_cookie(client).await? {
            log::debug!("Found DSID");
            return Ok(Some(value));
        }
        sleep(Duration::from_millis(100)).await; // Wait before retrying
    }
    log::debug!("Unable to find dsid");
    Ok(None)
}

pub async fn run_client_with_retries(
    mut driver: Driver,
    args: &mut Args,
) -> Result<(), DriverError> {
    let mut attempt_count = 0;
    loop {
        match run_client(args.url.clone(), args.port).await {
            Ok(_) => break,
            Err(err) => {
                if let DriverError::WebDriverConnectionError(e) = err {
                    log::warn!("WebDriverConnectionError encountered: {}. Retrying...", e);
                    log::trace!("{:#?}", e);
                    attempt_count += 1;
                    if attempt_count > 3 {
                        log::warn!("Exceeded maximum retry attempts for WebDriver connection: {e}");
                        log::trace!("{:#?}", e);
                        return Err(DriverError::WebDriverConnectionError(e));
                    }
                    sleep(Duration::from_secs(2)).await; // Delay before retrying
                } else if let DriverError::WebDriverClientError(e) = err {
                    if let fantoccini::error::NewSessionError::SessionNotCreated(e) = &e {
                        if let fantoccini::error::ErrorStatus::SessionNotCreated = e.error {
                            log::error!("Error: Required dependencies are not installed.");
                            log::error!(
                                "Please make sure you have the chrome/ium installed: \nError: {e}"
                            );
                            log::trace!("{:#?}", e);
                            log::error!("Exiting kuvpn...");
                            std::process::exit(1);
                        }
                    }
                    log::warn!(
                        "WebDriverClientError encountered: {}. Restarting driver...",
                        e
                    );
                    log::trace!("{:#?}", e);
                    // Drop the current driver to kill the process
                    drop(driver);
                    // Restart driver
                    driver = Driver::start(&mut args.port).await?;
                    attempt_count = 0; // Reset attempt count after restarting
                } else if let DriverError::ProcessStartError(e) = err {
                    log::warn!(
                        "ProcessStartError encountered: {}. Stopping application.",
                        e
                    );
                    log::trace!("{:#?}", e);
                    return Err(DriverError::ProcessStartError(e)); // Exit on process start error
                } else if let DriverError::WebDriverStartTimeout = err {
                    log::trace!("{:#?}", err);
                    return Err(DriverError::WebDriverStartTimeout);
                }
            }
        }
    }

    Ok(())
}
