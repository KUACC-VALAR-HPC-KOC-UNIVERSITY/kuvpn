use fantoccini::ClientBuilder;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{sleep, Duration};

struct ChromeDriver {
    process: tokio::process::Child,
}

impl Drop for ChromeDriver {
    fn drop(&mut self) {
        // Attempt to kill the ChromeDriver process
        let _ = self.process.kill();
    }
}

async fn start_chromedriver() -> Result<ChromeDriver, std::io::Error> {
    let process = Command::new("chromedriver")
        .arg("--port=9515")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Wait a moment to ensure ChromeDriver is fully started
    sleep(Duration::from_secs(2)).await;

    Ok(ChromeDriver { process })
}

#[tokio::main]
async fn main() -> Result<(), fantoccini::error::CmdError> {
    let _driver = start_chromedriver()
        .await
        .expect("failed to start ChromeDriver");
    let c = ClientBuilder::native()
        .connect("http://localhost:9515")
        .await
        .expect("failed to connect to WebDriver");

    // Go to the VPN login page
    c.goto("https://vpn.ku.edu.tr").await?;

    loop {
        // Execute JavaScript to get the DSID cookie
        let script = "return document.cookie.split('; ').find(row => row.startsWith('DSID='))?.split('=')[1];";
        let result = c.execute(script, vec![]).await?;

        // Extract the DSID cookie from the result
        let dsid_cookie = result.as_str().map(|s| s.to_string());

        if let Some(cookie_value) = dsid_cookie {
            c.close().await?;

            println!("DSID cookie found: {}", cookie_value);

            // Construct the OpenConnect command
            let openconnect_command = format!(
                "sudo openconnect --protocol nc -C 'DSID={}' vpn.ku.edu.tr",
                cookie_value
            );
            println!("Command to execute: {}", openconnect_command);

            // Optionally execute the command using std::process::Command
            use std::process::Command as StdCommand;
            StdCommand::new("sh")
                .arg("-c")
                .arg(openconnect_command)
                .status()
                .expect("Failed to execute command");
            break;
        }
    }

    Ok(())
}
