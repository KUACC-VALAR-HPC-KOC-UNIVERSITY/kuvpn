use crate::driver::DriverError;
use crate::utils::get_dsid_cookie;
use fantoccini::{error::ErrorStatus, Client, Locator};
use tokio::time::{sleep, Duration};

pub enum WaitRes {
    Found,
    TimeOut,
    FoundCookie(String),
}

pub async fn wait_and_send_keys(
    c: &Client,
    selector: &str,
    keys: &str,
    timeout: Duration,
) -> Result<WaitRes, DriverError> {
    let start_time = std::time::Instant::now();
    while start_time.elapsed() < timeout {
        if let Some(cookie) = get_dsid_cookie(c).await? {
            return Ok(WaitRes::FoundCookie(cookie));
        }
        match c.find(Locator::Css(selector)).await {
            Ok(element) => {
                if let Ok(_) = element.send_keys(keys).await {
                    return Ok(WaitRes::Found);
                }
            }
            Err(fantoccini::error::CmdError::NotJson(_)) => {}
            Err(fantoccini::error::CmdError::Json(_)) => {}
            Err(fantoccini::error::CmdError::InvalidArgument(_, _)) => {}
            Err(fantoccini::error::CmdError::Standard(e)) => {
                if e.error == ErrorStatus::NoSuchElement || e.error == ErrorStatus::NoSuchWindow {
                    continue;
                } else {
                    return Err(DriverError::WebDriverConnectionError(
                        fantoccini::error::CmdError::Standard(e),
                    ));
                }
            }
            Err(e) => return Err(DriverError::WebDriverConnectionError(e)),
        }
        sleep(Duration::from_millis(150)).await;
    }
    Ok(WaitRes::TimeOut)
}

pub async fn wait_and_click(
    c: &Client,
    selector: &str,
    timeout: Duration,
) -> Result<WaitRes, DriverError> {
    let start_time = std::time::Instant::now();
    while start_time.elapsed() < timeout {
        if let Some(cookie) = get_dsid_cookie(c).await? {
            return Ok(WaitRes::FoundCookie(cookie));
        }
        match c.find(Locator::Css(selector)).await {
            Ok(element) => {
                if let Ok(_) = element.click().await {
                    return Ok(WaitRes::Found);
                }
            }
            Err(fantoccini::error::CmdError::NotJson(_)) => {}
            Err(fantoccini::error::CmdError::Json(_)) => {}
            Err(fantoccini::error::CmdError::InvalidArgument(_, _)) => {}
            Err(fantoccini::error::CmdError::Standard(e)) => {
                if e.error == ErrorStatus::NoSuchElement || e.error == ErrorStatus::NoSuchWindow {
                    continue;
                } else {
                    return Err(DriverError::WebDriverConnectionError(
                        fantoccini::error::CmdError::Standard(e),
                    ));
                }
            }
            Err(e) => return Err(DriverError::WebDriverConnectionError(e)),
        }
        sleep(Duration::from_millis(500)).await;
    }
    Ok(WaitRes::TimeOut)
}
