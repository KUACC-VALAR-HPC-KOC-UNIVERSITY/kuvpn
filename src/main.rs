mod args;
mod client;
mod driver;
mod utils;

use args::Args;
use clap::Parser;
use client::run_client_with_retries;
use driver::{Driver, DriverError};
use tokio::time::Duration;
use utils::check_dependencies;

#[tokio::main]
async fn main() -> Result<(), DriverError> {
    check_dependencies()?;

    let mut args = Args::parse();
    let driver = Driver::start(&mut args.port).await?;

    // Wait for the WebDriver to be reachable
    driver
        .wait_for_webdriver(args.port, Duration::from_secs(30))
        .await?;

    run_client_with_retries(driver, &mut args).await?;

    Ok(())
}
