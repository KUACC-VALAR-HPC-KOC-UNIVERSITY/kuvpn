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

    if let Some(cookie_value) = get_dsid_cookie_with_retries(&client).await? {
        client.close().await?;
        println!("DSID cookie found: {}", cookie_value);
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

    Ok(client)
}

async fn get_dsid_cookie_with_retries(
    client: &fantoccini::Client,
) -> Result<Option<String>, DriverError> {
    let start_time = std::time::Instant::now();
    while start_time.elapsed() < Duration::from_secs(30) {
        if let Some(value) = get_dsid_cookie(client).await? {
            return Ok(Some(value));
        }
        sleep(Duration::from_millis(100)).await; // Wait before retrying
    }
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
                    // Drop the current driver to kill the process
                    drop(driver);
                    // Restart driver
                    driver = Driver::start(&mut args.port).await?;
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
