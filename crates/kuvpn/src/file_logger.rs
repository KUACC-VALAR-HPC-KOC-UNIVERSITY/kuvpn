//! Rotating file logger for persistent session logs.
//!
//! Writes timestamped lines to a log file, rotating to `<name>.1` when the
//! file exceeds [`MAX_LOG_BYTES`].

use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

/// Maximum log file size before rotation (10 MiB).
const MAX_LOG_BYTES: u64 = 10 * 1024 * 1024;

/// A simple rotating file logger. Writes one timestamped line per call.
/// When the file reaches [`MAX_LOG_BYTES`], it is renamed to `<path>.1` and
/// a fresh file is started. Only one backup (`*.1`) is kept.
pub struct FileLogger {
    path: PathBuf,
    writer: BufWriter<File>,
}

impl FileLogger {
    /// Opens (or creates) a log file at `path`. Returns `None` on I/O error.
    pub fn open(path: PathBuf) -> Option<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .ok()?;
        Some(Self {
            path,
            writer: BufWriter::new(file),
        })
    }

    /// Appends a timestamped line to the log file and flushes immediately.
    /// Rotates the file if it has grown past [`MAX_LOG_BYTES`].
    pub fn write_line(&mut self, line: &str) {
        let ts = crate::history::format_timestamp_unix(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
        let _ = writeln!(self.writer, "[{}] {}", ts, line);
        let _ = self.writer.flush();
        self.rotate_if_needed();
    }

    fn rotate_if_needed(&mut self) {
        let too_big = fs::metadata(&self.path)
            .map(|m| m.len() >= MAX_LOG_BYTES)
            .unwrap_or(false);
        if !too_big {
            return;
        }
        // Rename current log to .1 (overwrites any previous backup) and reopen.
        let backup = self.path.with_extension("log.1");
        let _ = fs::rename(&self.path, &backup);
        if let Ok(file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            self.writer = BufWriter::new(file);
        }
    }
}
