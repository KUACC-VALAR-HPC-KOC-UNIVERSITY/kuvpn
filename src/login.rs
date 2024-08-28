use crate::driver::DriverError;
use crate::utils::skip_host_checker;
use crate::wait::{wait_and_click, wait_and_send_keys, WaitRes};
use fantoccini::Client;
use std::env;
use tokio::time::Duration;

pub async fn perform_autologin(c: &Client) -> Result<Option<String>, DriverError> {
    let (username, password) = match (env::var("KUVPN_USERNAME"), env::var("KUVPN_PASSWORD")) {
        (Ok(u), Ok(p)) if !u.is_empty() && !p.is_empty() => (u, p),
        _ => {
            println!(
                "KUVPN_USERNAME and/or KUVPN_PASSWORD not set or empty. Skipping login process."
            );
            return Ok(None);
        }
    };

    // Wait for the email input field to appear and then fill it
    match wait_and_send_keys(
        c,
        "input[name='loginfmt']",
        &username,
        Duration::from_secs(10),
    )
    .await?
    {
        WaitRes::FoundCookie(cookie) => return Ok(Some(cookie)),
        _ => {}
    }

    // Click the "Next" button
    match wait_and_click(c, "#idSIButton9", Duration::from_secs(10)).await? {
        WaitRes::FoundCookie(cookie) => return Ok(Some(cookie)),
        _ => {}
    }

    // Wait for the password input field to appear and then fill it
    match wait_and_send_keys(
        c,
        "input[name='passwd']",
        &password,
        Duration::from_secs(10),
    )
    .await?
    {
        WaitRes::FoundCookie(cookie) => return Ok(Some(cookie)),
        _ => {}
    }

    // Click the "Sign in" button
    match wait_and_click(c, "#idSIButton9", Duration::MAX).await? {
        WaitRes::FoundCookie(cookie) => return Ok(Some(cookie)),
        _ => {}
    }

    // Click the "No: For Stay Signed in?" button as, it will not remember anyways
    match wait_and_click(c, "#idSIButton9", Duration::MAX).await? {
        WaitRes::FoundCookie(cookie) => return Ok(Some(cookie)),
        _ => {}
    }

    skip_host_checker(c).await;

    // Check if we're on the confirmation page for multiple sessions
    match wait_and_click(c, "#btnContinue", Duration::from_secs(30)).await? {
        WaitRes::FoundCookie(cookie) => return Ok(Some(cookie)),
        _ => {}
    }

    Ok(None)
}
