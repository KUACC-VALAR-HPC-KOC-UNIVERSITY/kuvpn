/// Windows elevated helper client.
///
/// Security model
/// ──────────────
/// • The helper binary lives in the same admin-protected `Program Files`
///   directory as the rest of the installer; it verifies this at startup before
///   executing anything.
/// • The IPC protocol sends only `cookie` and `url` — no executable path.  The
///   helper resolves `openconnect.exe` relative to itself, so a caller cannot
///   redirect execution to an arbitrary binary.
/// • Cookie and URL values are validated in the helper before use.
/// • The pipe name is unique per process (PID-based) to avoid collisions when
///   multiple KUVPN instances run simultaneously.  It is not a security measure.
///
/// Lifecycle
/// ─────────
/// `WinElevatedClient::launch()` creates the Named Pipe listener, elevates the
/// helper via a single UAC prompt (via `runas`), waits up to 60 s for the
/// helper to connect, then returns.  All subsequent VPN commands flow through
/// the connected pipe with no further elevation prompts.  Dropping the client
/// closes the stream; the helper detects the broken pipe and exits.

use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use anyhow::Context;
use interprocess::local_socket::{prelude::*, GenericNamespaced, ListenerOptions};
use serde::{Deserialize, Serialize};

// ── protocol ──────────────────────────────────────────────────────────────────

/// Only `cookie` and `url` travel over the wire — no exe path, no arbitrary args.
#[derive(Serialize)]
#[serde(tag = "op")]
enum Request<'a> {
    #[serde(rename = "start")]
    Start { cookie: &'a str, url: &'a str },
    #[serde(rename = "stop")]
    Stop,
}

#[derive(Deserialize)]
struct Response {
    ok: bool,
    error: Option<String>,
}

// ── client ────────────────────────────────────────────────────────────────────

pub struct WinElevatedClient {
    /// Wraps the connected stream for buffered line reading.
    /// We write via `reader.get_mut()` before each read, which is safe because
    /// the protocol is strictly request → response with no buffered leftovers.
    reader: BufReader<LocalSocketStream>,
}

// LocalSocketStream on Windows is a Named Pipe handle — safe to send across threads.
unsafe impl Send for WinElevatedClient {}

impl WinElevatedClient {
    /// Spin up the Named Pipe listener, launch the helper elevated (one UAC
    /// prompt), wait for it to connect, and return the ready client.
    pub fn launch(helper_exe: &Path) -> anyhow::Result<Self> {
        // Unique pipe name — PID-based to avoid conflicts between instances.
        // Not a security boundary; the restricted protocol is the security measure.
        let pipe_name = format!("kuvpn-{}", std::process::id());

        let name = pipe_name
            .clone()
            .to_ns_name::<GenericNamespaced>()
            .context("create pipe name")?;

        let listener = ListenerOptions::new()
            .name(name)
            .create_sync()
            .context("create Named Pipe listener")?;

        // Elevate the helper in a background thread — runas blocks on UAC.
        let helper_exe_owned = helper_exe.to_path_buf();
        let pipe_name_for_helper = pipe_name.clone();
        std::thread::spawn(move || {
            let _ = runas::Command::new(&helper_exe_owned)
                .arg(&pipe_name_for_helper)
                .show(false)
                .status();
        });

        // Wait up to 60 s for the helper to connect (covers UAC interaction time).
        log::info!("Waiting for elevated helper on pipe {}…", pipe_name);

        let (tx, rx) = crossbeam_channel::bounded(1);
        std::thread::spawn(move || {
            let _ = tx.send(listener.accept());
        });

        let stream = match rx.recv_timeout(std::time::Duration::from_secs(60)) {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => anyhow::bail!("Named Pipe accept failed: {}", e),
            Err(_) => anyhow::bail!(
                "Timed out waiting for elevated helper \
                 (UAC denied or kuvpn-win-helper.exe not found)"
            ),
        };

        log::info!("Elevated helper connected.");
        Ok(WinElevatedClient {
            reader: BufReader::new(stream),
        })
    }

    /// Tell the helper to start OpenConnect with the given cookie and URL.
    /// No executable path is sent — the helper resolves it locally.
    pub fn start_openconnect(&mut self, cookie: &str, url: &str) -> anyhow::Result<()> {
        self.send_request(&Request::Start { cookie, url })
    }

    /// Tell the helper to stop OpenConnect.
    pub fn stop_openconnect(&mut self) -> anyhow::Result<()> {
        self.send_request(&Request::Stop)
    }

    // ── internal ──────────────────────────────────────────────────────────────

    fn send_request<S: Serialize>(&mut self, req: &S) -> anyhow::Result<()> {
        // Serialize + write request line.
        let mut json = serde_json::to_string(req).context("serialize request")?;
        json.push('\n');
        // get_mut() is safe here: we always write before reading, so the
        // BufReader's internal buffer is guaranteed to be empty at this point.
        self.reader
            .get_mut()
            .write_all(json.as_bytes())
            .context("write request to helper pipe")?;

        // Read response line.
        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .context("read response from helper pipe")?;

        let resp: Response =
            serde_json::from_str(line.trim()).context("parse helper response")?;

        if resp.ok {
            Ok(())
        } else {
            anyhow::bail!(
                "Helper error: {}",
                resp.error.unwrap_or_else(|| "unknown".to_string())
            )
        }
    }
}
