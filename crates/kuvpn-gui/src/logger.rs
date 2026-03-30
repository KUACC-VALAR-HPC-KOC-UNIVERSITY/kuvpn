use std::sync::Mutex;
use tokio::sync::mpsc;

pub struct GuiLogger {
    pub tx: Mutex<Option<mpsc::Sender<String>>>,
    pub user_level: Mutex<log::LevelFilter>,
}

impl GuiLogger {
    pub fn set_tx(&self, tx: mpsc::Sender<String>) {
        if let Ok(mut guard) = self.tx.lock() {
            *guard = Some(tx);
        }
    }

    pub fn set_level(&self, level: log::LevelFilter) {
        if let Ok(mut guard) = self.user_level.lock() {
            *guard = level;
        }
    }

    pub fn get_level(&self) -> log::LevelFilter {
        self.user_level
            .lock()
            .map(|g| *g)
            .unwrap_or(log::LevelFilter::Info)
    }
}

impl log::Log for GuiLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Trace
    }
    fn log(&self, record: &log::Record) {
        // Suppress Iced/wgpu/winit internal logs.
        // Forward kuvpn crate output and headless_chrome (browser automation) logs.
        let target = record.target();
        if !target.starts_with("kuvpn") && !target.starts_with("headless_chrome") {
            return;
        }
        let Ok(guard) = self.tx.lock() else { return };
        if let Some(tx) = &*guard {
            let level = match record.level() {
                log::Level::Error => "Error",
                log::Level::Warn => "Warn",
                log::Level::Info => "Info",
                log::Level::Debug => "Debug",
                log::Level::Trace => "Trace",
            };
            let _ = tx.try_send(format!("{}|{}", level, record.args()));
        }
    }
    fn flush(&self) {}
}

pub static GUI_LOGGER: GuiLogger = GuiLogger {
    tx: Mutex::new(None),
    user_level: Mutex::new(log::LevelFilter::Info),
};

pub static LOGGER_INIT: std::sync::Once = std::sync::Once::new();
