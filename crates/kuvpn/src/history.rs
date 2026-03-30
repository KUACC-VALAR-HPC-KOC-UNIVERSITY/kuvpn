//! Connection event history.
//!
//! Persists connect/disconnect events to a JSON file in the user data directory.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Connected,
    Reconnected,
    Disconnected,
    Cancelled,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionEvent {
    /// Unix timestamp (seconds) when the event occurred.
    pub timestamp_unix: u64,
    pub kind: EventKind,
    /// Duration in seconds (set on Disconnected / Error).
    pub duration_secs: Option<u64>,
    /// Error message (set on Error).
    pub message: Option<String>,
}

impl ConnectionEvent {
    pub fn now(kind: EventKind) -> Self {
        Self {
            timestamp_unix: now_unix(),
            kind,
            duration_secs: None,
            message: None,
        }
    }

    /// Formats the event timestamp as `"YYYY-MM-DD HH:MM:SS"` (UTC).
    pub fn format_timestamp(&self) -> String {
        format_timestamp_unix(self.timestamp_unix)
    }

    /// Formats the session duration as a human-readable string (e.g. `"1h 23m 45s"`),
    /// or `None` if no duration was recorded.
    pub fn format_duration_display(&self) -> Option<String> {
        self.duration_secs.map(format_duration_secs)
    }
}

pub(crate) fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Converts a count of days since 1970-01-01 to `(year, month, day)`.
///
/// Uses the proleptic Gregorian algorithm from the civil::date paper.
pub(crate) fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Formats a Unix timestamp (seconds since epoch) as `"YYYY-MM-DD HH:MM:SS"` (UTC).
pub fn format_timestamp_unix(unix: u64) -> String {
    let secs_of_day = unix % 86400;
    let hour = secs_of_day / 3600;
    let min = (secs_of_day % 3600) / 60;
    let sec = secs_of_day % 60;
    let (year, month, day) = days_to_ymd(unix / 86400);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hour, min, sec
    )
}

/// Formats a duration in seconds as a human-readable string (e.g. `"1h 23m 45s"`).
pub fn format_duration_secs(total_secs: u64) -> String {
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

fn history_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(crate::utils::kuvpn_data_dir()?.join("history.json"))
}

/// Appends one event to the on-disk history file.
pub fn append_event(event: &ConnectionEvent) -> Result<(), Box<dyn std::error::Error>> {
    let path = history_path()?;
    let mut events: Vec<ConnectionEvent> = if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                log::warn!("history.json parse error (treating as empty): {}", e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };
    events.push(event.clone());
    std::fs::write(&path, serde_json::to_string_pretty(&events)?)?;
    Ok(())
}

/// Loads all events from the on-disk history file.
pub fn load_events() -> Result<Vec<ConnectionEvent>, Box<dyn std::error::Error>> {
    let path = history_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path)?;
    match serde_json::from_str(&content) {
        Ok(v) => Ok(v),
        Err(e) => {
            log::warn!("history.json parse error (returning empty history): {}", e);
            Ok(Vec::new())
        }
    }
}

/// Removes the history file.
pub fn clear_events() -> Result<(), Box<dyn std::error::Error>> {
    let path = history_path()?;
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}
