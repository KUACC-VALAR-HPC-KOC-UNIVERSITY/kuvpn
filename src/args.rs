use clap::{Parser, ValueEnum};

#[derive(Debug, ValueEnum, Clone)]
pub enum LogLevel {
    /// Informational messages
    Info,
    /// Debugging messages
    Debug,
    /// Detailed stack trace messages
    Stacktrace,
}

/// Simple program to retrieve DSID cookie and execute OpenConnect command
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The URL to the page where we will start logging in and looking for DSID
    #[arg(short, long, default_value = "https://vpn.ku.edu.tr")]
    pub url: String,

    /// The port number for the ChromeDriver
    #[arg(short, long, default_value_t = 9515)]
    pub port: u16,

    /// The level of logging
    #[arg(short, long, value_enum, default_value_t = LogLevel::Info)]
    pub level: LogLevel,

    /// Gives the user the dsid without running openconnect
    #[arg(short, long, default_value_t = false)]
    pub dsid: bool,
}
