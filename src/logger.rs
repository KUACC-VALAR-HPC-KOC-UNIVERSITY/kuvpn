use crate::args::LogLevel;
use chrono::Local;
use colored::*;
use log::{Level, LevelFilter};
use std::io::Write;

pub fn init_logger(mode: &LogLevel) {
    let level = match mode {
        LogLevel::Info => LevelFilter::Info,
        LogLevel::Debug => LevelFilter::Debug,
        LogLevel::Stacktrace => LevelFilter::Trace,
    };
    env_logger::Builder::new()
        .filter(None, level)
        .format(|buf, record| {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S"); // Update this line
            let level = match record.level() {
                Level::Error => "ERROR".red(),
                Level::Warn => "WARN".yellow(),
                Level::Info => "INFO".green(),
                Level::Debug => "DEBUG".blue(),
                Level::Trace => "TRACE".purple(),
            };
            writeln!(buf, "{} [{}] - {}", timestamp, level, record.args())
        })
        .init();
}
