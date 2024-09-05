mod args;
mod client;
mod driver;
mod logger;
mod utils;

use crate::logger::init_logger;
use args::Args;
use clap::Parser;
use client::run_client_with_retries;
use driver::{Driver, DriverError};
use tokio::time::Duration;
use utils::check_dependencies;

#[tokio::main]
async fn main() -> Result<(), DriverError> {
    let mut args = Args::parse();
    init_logger(&args.level);

    check_dependencies(args.dsid)?;
    log::debug!("Dependencies checked successfully.");

    let driver = Driver::start(&mut args.port).await?;
    log::debug!("Driver started successfully.");

    // Wait for the WebDriver to be reachable
    driver
        .wait_for_webdriver(args.port, Duration::from_secs(30))
        .await?;
    log::debug!("WebDriver is reachable.");

    let args = &mut args;
    run_client_with_retries(driver, args, args.dsid).await?;

    log::debug!("Reached end of main.");
    Ok(())
}
