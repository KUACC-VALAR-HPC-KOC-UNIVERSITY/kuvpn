use clap::{Parser, ValueEnum};

#[derive(Debug, ValueEnum, Clone)]
pub enum LogLevel {
    /// No logs
    Off,
    /// Informational messages
    Info,
    /// Warning messages
    Warn,
    /// Debugging messages
    Debug,
    /// Error messages
    Error,
    /// Detailed stacktrace messages
    Trace,
}

impl From<LogLevel> for log::LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Off => log::LevelFilter::Off,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Trace => log::LevelFilter::Trace,
        }
    }
}

/// Simple program to retrieve DSID cookie and execute OpenConnect command
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The URL to the page where we will start logging in and looking for DSID
    #[arg(short, long, default_value = "https://vpn.ku.edu.tr")]
    pub url: String,

    /// The level of logging
    #[arg(short, long, value_enum, default_value_t = LogLevel::Error)]
    pub level: LogLevel,

    /// The Domain of the DSID found
    #[arg(long, default_value = "vpn.ku.edu.tr")]
    pub domain: String,

    /// Gives the user the dsid without running openconnect
    #[arg(short, long, default_value_t = false)]
    pub get_dsid: bool,

    /// Gets DSID without headless mode
    #[arg(short, long, default_value_t = false)]
    pub disable_headless: bool,

    /// Delete session information
    #[arg(short, long, default_value_t = false)]
    pub clean: bool,

    /// Command to run openconnect with (e.g., sudo, pkexec, or a custom script)
    #[arg(short, long)]
    pub run_command: Option<String>,

    /// Path or command name for openconnect. Defaults to 'openconnect'.
    /// Can be a relative or absolute path.
    #[cfg(not(windows))]
    #[arg(long, default_value = "openconnect")]
    pub openconnect_path: String,

    /// Disable automatic login handlers and only poll for DSID in a headful browser
    #[arg(long, default_value_t = false)]
    pub no_auto_login: bool,

    /// Email for login (optional)
    #[arg(long, default_value = None)]
    pub email: Option<String>,

    /// Name for the TUN/TAP interface created by openconnect
    #[arg(long, default_value = "kuvpn0")]
    pub interface_name: String,
}
