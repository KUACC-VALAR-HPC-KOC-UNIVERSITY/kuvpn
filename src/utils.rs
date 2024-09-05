use crate::driver::DriverError;
use fantoccini::Client;

pub async fn get_dsid_cookie(c: &Client) -> Result<Option<String>, DriverError> {
    let script =
        "return document.cookie.split('; ').find(row => row.startsWith('DSID='))?.split('=')[1];";
    let result = c.execute(script, vec![]).await?;
    Ok(result.as_str().map(|s| s.to_string()))
}

pub async fn skip_host_checker(c: &Client) {
    // Execute gowelcome(); until it fails or runs 2 times correctly
    log::debug!("Executing gowelcome();...");
    let mut successful_executions = 0;
    while successful_executions < 3 {
        match c.execute("gowelcome();", vec![]).await {
            Ok(_) => {
                successful_executions += 1;
                log::debug!("Successful_execution number: {}...", successful_executions);
            }
            Err(_) => {
                log::debug!("Unsuccessful_execution of gowelcome, skipping...");
                return;
            }
        }
    }
}

pub fn check_dependencies(dsid: bool) -> Result<(), DriverError> {
    let chromedriver_installed = std::process::Command::new("which")
        .arg("chromedriver")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    let openconnect_installed = std::process::Command::new("which")
        .arg("openconnect")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if !chromedriver_installed || !(openconnect_installed || dsid) {
        log::error!("Error: Required dependencies are not installed.");
        if !chromedriver_installed {
            log::error!("Please install chromedriver.");
        }
        if !(openconnect_installed || dsid) {
            log::error!("Please install openconnect.");
        }
        std::process::exit(1);
    }

    Ok(())
}

pub fn execute_openconnect(cookie_value: String) -> Result<(), DriverError> {
    let openconnect_command = format!(
        "sudo openconnect --protocol nc -C 'DSID={}' vpn.ku.edu.tr",
        cookie_value
    );
    log::info!("Executing: {}", openconnect_command);

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
