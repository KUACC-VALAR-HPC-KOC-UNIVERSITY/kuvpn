mod args;
mod driver;

use args::Args;
use clap::Parser;
use driver::{Driver, DriverError};
use fantoccini::{error::ErrorStatus, ClientBuilder};
use std::env;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), DriverError> {
    let mut args = Args::parse();

    let mut driver = Driver::start(args.browser.clone(), &mut args.port).await?;

    // Wait for the WebDriver to be reachable
    driver
        .wait_for_webdriver(args.port, Duration::from_secs(30))
        .await?;

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
                DriverError::WebDriverStartTimeout => {
                    return Err(DriverError::WebDriverStartTimeout);
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

async fn perform_autologin(c: &fantoccini::Client) -> Result<Option<String>, DriverError> {
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

    // Execute gowelcome(); until it fails or runs 2 times correctly
    let mut successful_executions = 0;
    while successful_executions < 2 {
        match c.execute("gowelcome();", vec![]).await {
            Ok(_) => successful_executions += 1,
            Err(_) => break,
        }
    }

    // Check if we're on the confirmation page for multiple sessions
    match wait_and_click(c, "#btnContinue", Duration::from_secs(30)).await? {
        WaitRes::FoundCookie(cookie) => return Ok(Some(cookie)),
        _ => {}
    }

    Ok(None)
}

async fn get_dsid_cookie(c: &fantoccini::Client) -> Result<Option<String>, DriverError> {
    let script =
        "return document.cookie.split('; ').find(row => row.startsWith('DSID='))?.split('=')[1];";
    let result = c.execute(script, vec![]).await?;
    Ok(result.as_str().map(|s| s.to_string()))
}

fn execute_openconnect(cookie_value: String) -> Result<(), DriverError> {
    let openconnect_command = format!(
        "sudo openconnect --protocol nc -C 'DSID={}' vpn.ku.edu.tr",
        cookie_value
    );
    println!("Executing: {}", openconnect_command);

    // Use std::process::Command to execute openconnect
    use std::process::Command as StdCommand;
    // Spawn the openconnect process in the background
    StdCommand::new("sh")
        .arg("-c")
        .arg(&openconnect_command)
        .status()
        .map_err(DriverError::ProcessStartError)?;

    Ok(())
}

enum WaitRes {
    Found,
    TimeOut,
    FoundCookie(String),
}

async fn wait_and_send_keys(
    c: &fantoccini::Client,
    selector: &str,
    keys: &str,
    timeout: Duration,
) -> Result<WaitRes, DriverError> {
    let start_time = std::time::Instant::now();
    while start_time.elapsed() < timeout {
        if let Some(cookie) = get_dsid_cookie(c).await? {
            return Ok(WaitRes::FoundCookie(cookie));
        }
        match c.find(fantoccini::Locator::Css(selector)).await {
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
                    return Err(driver::DriverError::WebDriverConnectionError(
                        fantoccini::error::CmdError::Standard(e),
                    ));
                }
            }
            Err(e) => return Err(driver::DriverError::WebDriverConnectionError(e)),
        }
        sleep(Duration::from_millis(150)).await;
    }
    Ok(WaitRes::TimeOut)
}

async fn wait_and_click(
    c: &fantoccini::Client,
    selector: &str,
    timeout: Duration,
) -> Result<WaitRes, DriverError> {
    let start_time = std::time::Instant::now();
    while start_time.elapsed() < timeout {
        if let Some(cookie) = get_dsid_cookie(c).await? {
            return Ok(WaitRes::FoundCookie(cookie));
        }
        match c.find(fantoccini::Locator::Css(selector)).await {
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
                    return Err(driver::DriverError::WebDriverConnectionError(
                        fantoccini::error::CmdError::Standard(e),
                    ));
                }
            }
            Err(e) => return Err(driver::DriverError::WebDriverConnectionError(e)),
        }
        sleep(Duration::from_millis(500)).await;
    }
    Ok(WaitRes::TimeOut)
}
