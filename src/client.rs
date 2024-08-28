use crate::driver::DriverError;
use crate::login::perform_autologin;
use crate::utils::{execute_openconnect, get_dsid_cookie, skip_host_checker};
use fantoccini::ClientBuilder;
use serde_json;
use tokio::time::{sleep, Duration};

const CHROME_PROFILE_RELATIVE_PATH: &str = ".config/kuvpn/profile";

pub async fn run_client(url: String, port: u16) -> Result<(), DriverError> {
    let home_dir = std::env::var("HOME").expect("Could not find home directory");
    let chrome_profile_path = std::path::Path::new(&home_dir).join(CHROME_PROFILE_RELATIVE_PATH);
    let chrome_profile_path_str = chrome_profile_path.to_str().expect("Invalid path");

    let mut capabilities = serde_json::Map::new();

    // Add Chrome options for kiosk mode and custom profile directory
    let mut chrome_options = serde_json::Map::new();
    chrome_options.insert(
        "args".to_string(),
        serde_json::Value::Array(vec![
            serde_json::Value::String("--kiosk".to_string()),
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

    let c = ClientBuilder::rustls()?
        .capabilities(capabilities)
        .connect(&format!("http://localhost:{}", port))
        .await
        .map_err(DriverError::WebDriverClientError)?;

    c.goto(&url).await?;

    skip_host_checker(&c).await;

    perform_autologin(&c).await?;

    let mut cookie_value = None;
    let start_time = std::time::Instant::now();
    while start_time.elapsed() < Duration::from_secs(30) {
        if let Some(value) = get_dsid_cookie(&c).await? {
            cookie_value = Some(value);
            break;
        }
        sleep(Duration::from_millis(100)).await; // Wait before retrying
    }

    if let Some(cookie_value) = cookie_value {
        // Close the WebDriver client immediately
        c.close().await?;

        println!("DSID cookie found: {}", cookie_value);
        execute_openconnect(cookie_value)?;

        return Ok(());
    }

    Ok(())
}
