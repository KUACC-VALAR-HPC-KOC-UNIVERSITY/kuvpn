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

    /// Gives the user the dsid without running openconnect
    #[arg(short, long, default_value_t = false)]
    pub dsid: bool,

    /// Delete session information
    #[arg(short, long, default_value_t = false)]
    pub clean: bool,

    /// User agent for browser
    #[arg(short, long, default_value = "Mozilla/5.0")]
    pub agent: String,
}
