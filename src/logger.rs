use crate::args::LogLevel;

use colored::Colorize;
use log::{Level, LevelFilter};
use std::io::Write;

pub fn init_logger(mode: &LogLevel) {
    let level = match mode {
        LogLevel::Info => LevelFilter::Info,
        LogLevel::Off => LevelFilter::Off,
        LogLevel::Error => LevelFilter::Error,
        LogLevel::Debug => LevelFilter::Debug,
        LogLevel::Warn => LevelFilter::Warn,
        LogLevel::Trace => LevelFilter::Trace,
    };
    env_logger::Builder::new()
        .filter(None, level)
        .format(|buf, record| {
            let level = match record.level() {
                Level::Error => "ERROR".red(),
                Level::Warn => "WARN".yellow(),
                Level::Info => "INFO".green(),
                Level::Debug => "DEBUG".blue(),
                Level::Trace => "TRACE".purple(),
            };
            writeln!(buf, "[{}] - {}", level, record.args())
        })
        .init();
}
