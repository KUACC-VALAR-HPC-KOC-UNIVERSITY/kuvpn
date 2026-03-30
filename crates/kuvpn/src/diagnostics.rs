use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// Thread-local slot where browser automation code stores the path of a
// freshly saved diagnostic bundle. `SessionThread::acquire_dsid` reads
// and clears this to forward the path over the structured log channel.
thread_local! {
    pub(crate) static PENDING_DIAG_PATH: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// A snapshot of the browser state at the moment of an automation failure.
#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosticBundle {
    pub timestamp: String,
    pub url: String,
    pub page_title: String,
    pub page_html: String,
    pub error: String,
}

impl DiagnosticBundle {
    /// Saves this bundle to `<user-data-parent>/diagnostics/<timestamp>.json`.
    /// Returns the path to the saved file.
    pub fn save(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let dir = crate::utils::get_user_data_dir()?
            .parent()
            .ok_or("user data dir has no parent")?
            .join("diagnostics");
        std::fs::create_dir_all(&dir)?;

        let filename = format!("{}.json", self.timestamp.replace(':', "-"));
        let path = dir.join(filename);
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(path)
    }
}

/// Returns the current UTC time formatted as `"YYYY-MM-DDTHH:MM:SS"`.
pub(crate) fn now_iso() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let secs_of_day = ts % 86400;
    let hour = secs_of_day / 3600;
    let min = (secs_of_day % 3600) / 60;
    let sec = secs_of_day % 60;

    let (year, month, day) = crate::history::days_to_ymd(ts / 86400);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
        year, month, day, hour, min, sec
    )
}
