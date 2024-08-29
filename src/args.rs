use clap::{Parser, ValueEnum};

#[derive(Debug, ValueEnum, Clone)]
pub enum LogLevel {
    Info,
    Debug,
    Stacktrace,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = "https://vpn.ku.edu.tr")]
    pub url: String,

    #[arg(short, long, default_value_t = 9515)]
    pub port: u16,

    #[arg(short, long, default_value_t = false)]
    pub debug: bool,

    #[arg(short, long, value_enum, default_value_t = LogLevel::Info)]
    pub level: LogLevel,
}
