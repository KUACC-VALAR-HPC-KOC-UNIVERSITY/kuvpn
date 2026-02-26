/// kuvpn-win-helper: elevated helper process for Windows.
///
/// Security model
/// ──────────────
/// • Resolves `openconnect.exe` relative to its own location — no caller-supplied
///   exe path is accepted.
/// • Verifies that the resolved binary lives under an admin-protected directory
///   (`%ProgramFiles%`, `%ProgramFiles(x86)%`, or `%SystemRoot%`).  This means
///   a standard user cannot replace the binary without first obtaining elevation.
/// • Validates cookie (`DSID=<value>`) and URL (`https?://…`) before executing
///   anything — malformed inputs are rejected outright.
/// • Exits as soon as the pipe breaks (main process death → automatic cleanup).
///
/// Protocol (newline-delimited JSON over interprocess Named Pipe):
///   → {"op":"start","cookie":"DSID=…","url":"https://…"}
///   ← {"ok":true}  |  {"ok":false,"error":"…"}
///
///   → {"op":"stop"}
///   ← {"ok":true}  |  {"ok":false,"error":"…"}

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

use interprocess::local_socket::{prelude::*, GenericNamespaced};
use serde::{Deserialize, Serialize};

// ── protocol ──────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(tag = "op")]
enum Request {
    #[serde(rename = "start")]
    Start { cookie: String, url: String },
    #[serde(rename = "stop")]
    Stop,
}

#[derive(Serialize)]
struct Response {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

// ── validation ────────────────────────────────────────────────────────────────

/// Accept only `DSID=<non-empty value with no whitespace or newlines>`.
fn validate_cookie(cookie: &str) -> bool {
    if let Some(value) = cookie.strip_prefix("DSID=") {
        !value.is_empty() && value.chars().all(|c| !c.is_whitespace())
    } else {
        false
    }
}

/// Accept only `https://…` or `http://…` with no whitespace or newlines.
fn validate_url(url: &str) -> bool {
    (url.starts_with("https://") || url.starts_with("http://"))
        && !url.contains(|c: char| c.is_whitespace())
}

/// Return true if `path` is under one of the system-administered directories
/// that a standard user cannot write to without elevation.
///
/// This defends against a scenario where the install directory somehow has
/// relaxed permissions: we refuse to execute a binary from a user-writable
/// location even if it happens to be named `openconnect.exe`.
fn is_in_protected_dir(path: &Path) -> bool {
    // Collect the canonical forms of each protected root.
    let protected_env_vars = ["ProgramFiles", "ProgramFiles(x86)", "SystemRoot"];

    let protected_roots: Vec<PathBuf> = protected_env_vars
        .iter()
        .filter_map(|&var| std::env::var(var).ok())
        .filter_map(|s| std::fs::canonicalize(&s).ok())
        .collect();

    let canonical_path = match std::fs::canonicalize(path) {
        Ok(p) => p,
        Err(_) => return false,
    };

    protected_roots
        .iter()
        .any(|root| canonical_path.starts_with(root))
}

// ── entry point ───────────────────────────────────────────────────────────────

fn main() {
    let pipe_name = std::env::args()
        .nth(1)
        .expect("Usage: kuvpn-win-helper <pipe-name>");

    // ── locate openconnect.exe ────────────────────────────────────────────────
    // Only look relative to the helper's own location — no caller-supplied path.
    let helper_dir = std::env::current_exe()
        .expect("cannot resolve helper path")
        .parent()
        .expect("helper has no parent directory")
        .to_path_buf();

    // Check the bundled subdirectory first, then the helper's own directory.
    let openconnect_exe: PathBuf = {
        let bundled = helper_dir.join("openconnect").join("openconnect.exe");
        if bundled.exists() {
            bundled
        } else {
            helper_dir.join("openconnect.exe")
        }
    };

    if !openconnect_exe.exists() {
        eprintln!(
            "kuvpn-win-helper: openconnect.exe not found (looked in {:?})",
            openconnect_exe
        );
        std::process::exit(1);
    }

    // ── verify the binary is in an admin-protected location ──────────────────
    if !is_in_protected_dir(&openconnect_exe) {
        eprintln!(
            "kuvpn-win-helper: refusing to execute {:?} — \
             it is not under a system-protected directory \
             (Program Files / Windows). \
             Please reinstall KUVPN to a standard location.",
            openconnect_exe
        );
        std::process::exit(1);
    }

    // ── connect to the Named Pipe created by the main process ─────────────────
    // Brief retry loop in case ConnectNamedPipe hasn't been called yet.
    let stream: LocalSocketStream = {
        let name = pipe_name
            .to_ns_name::<GenericNamespaced>()
            .expect("invalid pipe name");
        let mut conn = None;
        for _ in 0..30 {
            match LocalSocketStream::connect(name.clone()) {
                Ok(s) => {
                    conn = Some(s);
                    break;
                }
                Err(_) => std::thread::sleep(std::time::Duration::from_millis(200)),
            }
        }
        conn.expect("kuvpn-win-helper: could not connect to Named Pipe")
    };

    let mut reader = BufReader::new(stream);
    let mut child: Option<Child> = None;

    // ── main command loop ─────────────────────────────────────────────────────
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => break, // pipe closed → exit
            Ok(_) => {}
        }

        let response = match serde_json::from_str::<Request>(line.trim()) {
            Ok(Request::Start { cookie, url }) => {
                if !validate_cookie(&cookie) {
                    Response {
                        ok: false,
                        error: Some("invalid cookie format".to_string()),
                    }
                } else if !validate_url(&url) {
                    Response {
                        ok: false,
                        error: Some("invalid URL format".to_string()),
                    }
                } else {
                    match Command::new(&openconnect_exe)
                        .args(["--protocol", "nc", "-C", &cookie, &url])
                        .spawn()
                    {
                        Ok(c) => {
                            child = Some(c);
                            Response { ok: true, error: None }
                        }
                        Err(e) => Response {
                            ok: false,
                            error: Some(e.to_string()),
                        },
                    }
                }
            }

            Ok(Request::Stop) => {
                if let Some(ref mut c) = child {
                    let _ = c.kill();
                    let _ = c.wait();
                }
                child = None;
                let _ = Command::new("taskkill")
                    .args(["/F", "/IM", "openconnect.exe", "/T"])
                    .status();
                Response { ok: true, error: None }
            }

            Err(e) => Response {
                ok: false,
                error: Some(format!("parse error: {e}")),
            },
        };

        let mut json =
            serde_json::to_string(&response).unwrap_or_else(|_| r#"{"ok":false}"#.to_string());
        json.push('\n');
        if reader.get_mut().write_all(json.as_bytes()).is_err() {
            break; // pipe closed
        }
    }
}
