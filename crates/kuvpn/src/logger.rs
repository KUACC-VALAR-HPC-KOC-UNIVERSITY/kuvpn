use colored::Colorize;
use log::{Level, LevelFilter};
use std::io::Write;

pub fn init_logger(level: LevelFilter) {
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
