use crate::driver::DriverError;
use crate::login::perform_autologin;
use crate::utils::{execute_openconnect, get_dsid_cookie};
use fantoccini::ClientBuilder;
use tokio::time::{sleep, Duration};

pub async fn run_client(url: String, port: u16) -> Result<(), DriverError> {
    let c = ClientBuilder::rustls()?
        .connect(&format!("http://localhost:{}", port))
        .await
        .map_err(DriverError::WebDriverClientError)?;

    c.goto(&url).await?;

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
