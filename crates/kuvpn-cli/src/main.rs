//! # KUVPN CLI
//!
//! A clean terminal interface for connecting to Koc University's VPN.

mod args;
mod credentials;

use args::Args;
use clap::Parser;
use console::Style;
use credentials::CliCredentialsProvider;
use indicatif::{ProgressBar, ProgressStyle};
use kuvpn::{
    init_logger, run_login_and_get_dsid, ConnectionStatus, LoginConfig, ParsedLog, SessionConfig,
    VpnSession,
};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::{Duration, Instant};

// ── Terminal styles ───────────────────────────────────────────────────────────

struct CliStyles {
    green: Style,
    red: Style,
    dim: Style,
    bold: Style,
    yellow: Style,
}

impl CliStyles {
    fn new() -> Self {
        Self {
            green: Style::new().green().bold(),
            red: Style::new().red().bold(),
            dim: Style::new().dim(),
            bold: Style::new().bold(),
            yellow: Style::new().yellow().bold(),
        }
    }
}

// ── Log message handling ──────────────────────────────────────────────────────

/// Processes one parsed log entry, printing status updates and forwarding
/// non-status messages to the logger. Mutates `spinner_active` and
/// `connection_start` in place.
fn handle_log(
    parsed: &ParsedLog,
    spinner: &ProgressBar,
    spinner_active: &mut bool,
    connection_start: &mut Option<Instant>,
    styles: &CliStyles,
    interface_name: &str,
) {
    let msg = parsed.message.as_str();

    match msg {
        "Accessing campus gateway..." => {
            if !*spinner_active {
                spinner.enable_steady_tick(Duration::from_millis(80));
                *spinner_active = true;
            }
            spinner.set_message("Accessing campus gateway...");
            return;
        }
        "Initializing tunnel..." => {
            clear_spinner(spinner, spinner_active);
            eprintln!(
                "  {} Accessing campus gateway...",
                styles.green.apply_to("✓")
            );
            eprintln!("  {} Initializing tunnel...", styles.green.apply_to("✓"));
            return;
        }
        "VPN interface already active, monitoring..." => {
            clear_spinner(spinner, spinner_active);
            eprintln!(
                "  {} VPN already active, monitoring connection",
                styles.yellow.apply_to("~"),
            );
            *connection_start = Some(Instant::now());
            return;
        }
        "Connected." => {
            clear_spinner(spinner, spinner_active);
            *connection_start = Some(Instant::now());
            print_connected(interface_name, styles);
            eprintln!("    {}", styles.dim.apply_to("Press Ctrl+C to disconnect"));
            return;
        }
        "Disconnecting..." => {
            clear_spinner(spinner, spinner_active);
            eprintln!("  {} Disconnecting...", styles.dim.apply_to("●"));
            return;
        }
        "Disconnected." => {
            clear_spinner(spinner, spinner_active);
            let duration = connection_start
                .map(|s| {
                    format!(
                        " (session: {})",
                        kuvpn::format_duration_secs(s.elapsed().as_secs())
                    )
                })
                .unwrap_or_default();
            eprintln!(
                "  {} Disconnected{}",
                styles.dim.apply_to("●"),
                styles.dim.apply_to(duration),
            );
            return;
        }
        _ if msg.starts_with("Reconnecting") => {
            *connection_start = None;
            clear_spinner(spinner, spinner_active);
            eprintln!("  {} {}", styles.yellow.apply_to("~"), msg);
            return;
        }
        _ if msg.ends_with("requires a password. Prompting...") => return,
        _ => {}
    }

    if parsed.level == log::Level::Error {
        clear_spinner(spinner, spinner_active);
        eprintln!("  {} {}", styles.red.apply_to("✗"), msg);
        if msg.contains("Full Auto mode unable to complete login")
            || msg.contains("Could not find a handler")
        {
            eprintln!();
            eprintln!(
                "  {} Try the following:",
                styles.bold.apply_to("Suggestions:")
            );
            eprintln!(
                "    {} Switch to manual mode: {}",
                styles.dim.apply_to("•"),
                styles.bold.apply_to("--mode manual"),
            );
            eprintln!(
                "    {} Wipe session cache:    {}",
                styles.dim.apply_to("•"),
                styles.bold.apply_to("--clean"),
            );
        }
        return;
    }

    // Pass through to the logger for non-status messages.
    match parsed.level {
        log::Level::Warn => log::warn!("{}", msg),
        _ => log::info!("{}", msg),
    }
}

fn clear_spinner(spinner: &ProgressBar, active: &mut bool) {
    if *active {
        spinner.finish_and_clear();
        *active = false;
    }
}

#[cfg(unix)]
fn print_connected(interface_name: &str, styles: &CliStyles) {
    let iface =
        kuvpn::get_vpn_interface_name(interface_name).unwrap_or_else(|| interface_name.to_string());
    eprintln!(
        "  {} Connected to KU VPN {}",
        styles.green.apply_to("✓"),
        styles.dim.apply_to(format!("(interface: {})", iface)),
    );
}

#[cfg(not(unix))]
fn print_connected(_interface_name: &str, styles: &CliStyles) {
    eprintln!("  {} Connected to KU VPN", styles.green.apply_to("✓"));
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> ExitCode {
    // VPN helper mode: invoked by the app itself under elevation to manage
    // OpenConnect's lifecycle (single UAC prompt per connection).
    // Must run before clap parses args (it would reject --vpn-helper as unknown).
    #[cfg(windows)]
    if let Some(code) = kuvpn::run_vpn_helper_if_requested() {
        return ExitCode::from(code as u8);
    }

    let args = Args::parse();
    init_logger(args.log.clone().into());

    let styles = CliStyles::new();

    if !args.dsid {
        eprintln!(
            "{} {}",
            styles.bold.apply_to("KUVPN"),
            styles
                .dim
                .apply_to(format!("v{}", env!("CARGO_PKG_VERSION"))),
        );
    }

    if let Err(e) = kuvpn::utils::ensure_single_instance() {
        eprintln!("  {} {}", styles.red.apply_to("✗"), e);
        return ExitCode::FAILURE;
    }

    if args.history {
        return print_history(&styles);
    }

    if args.clean {
        return match kuvpn::utils::wipe_user_data_dir() {
            Ok(_) => {
                eprintln!("  {} Session data wiped", styles.green.apply_to("✓"));
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!(
                    "  {} Failed to wipe session data: {}",
                    styles.red.apply_to("✗"),
                    e
                );
                ExitCode::FAILURE
            }
        };
    }

    if args.dsid {
        return run_get_dsid(&args, &styles);
    }

    run_vpn_session(&args, &styles)
}

fn print_history(styles: &CliStyles) -> ExitCode {
    match kuvpn::load_events() {
        Ok(events) if events.is_empty() => {
            eprintln!(
                "  {} No connection history found.",
                styles.dim.apply_to("●")
            );
            ExitCode::SUCCESS
        }
        Ok(events) => {
            for event in events.iter().rev() {
                let kind = match event.kind {
                    kuvpn::EventKind::Connected => {
                        styles.green.apply_to("Connected   ").to_string()
                    }
                    kuvpn::EventKind::Reconnected => {
                        styles.yellow.apply_to("Reconnected ").to_string()
                    }
                    kuvpn::EventKind::Disconnected => {
                        styles.dim.apply_to("Disconnected").to_string()
                    }
                    kuvpn::EventKind::Cancelled => styles.dim.apply_to("Cancelled   ").to_string(),
                    kuvpn::EventKind::Error => styles.red.apply_to("Error       ").to_string(),
                };
                let dur = event
                    .format_duration_display()
                    .map(|d| {
                        if event.kind == kuvpn::EventKind::Reconnected {
                            format!(" (prev: {})", d)
                        } else {
                            format!(" ({})", d)
                        }
                    })
                    .unwrap_or_default();
                let msg = event
                    .message
                    .as_deref()
                    .map(|m| format!(" — {}", m))
                    .unwrap_or_default();
                eprintln!("  {} [{}]{}{}", kind, event.format_timestamp(), dur, msg);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!(
                "  {} Failed to load history: {}",
                styles.red.apply_to("✗"),
                e
            );
            ExitCode::FAILURE
        }
    }
}

fn run_get_dsid(args: &Args, styles: &CliStyles) -> ExitCode {
    // Save the cursor position before any output (spinner + log lines).
    // On success we restore here and erase to end-of-screen, removing only
    // what this invocation drew — unlike clear_screen() which wipes the
    // entire visible terminal including whatever was there before.
    let term = console::Term::stderr();
    let is_tty = term.is_term();
    if is_tty {
        eprint!("\x1b[s"); // ANSI save cursor
    }

    let spinner = Arc::new(ProgressBar::new_spinner());
    spinner.set_style(spinner_style());
    spinner.set_message("Retrieving DSID...");
    spinner.enable_steady_tick(Duration::from_millis(80));

    let config = LoginConfig {
        headless: args.mode.headless(),
        url: args.url.clone(),
        domain: args.domain.clone(),
        user_agent: "Mozilla/5.0".to_string(),
        no_auto_login: args.mode.no_auto_login(),
        email: args.email.clone(),
    };

    let provider = CliCredentialsProvider {
        spinner: Arc::clone(&spinner),
    };

    match run_login_and_get_dsid(&config, &provider, None, None) {
        Ok(dsid) => {
            spinner.finish_and_clear();
            if is_tty {
                // Restore to saved position then erase from there to
                // end-of-screen — clears the spinner and all log lines.
                eprint!("\x1b[u\x1b[J");
            }
            println!("{}", dsid);
            ExitCode::SUCCESS
        }
        Err(e) => {
            spinner.finish_and_clear();
            eprintln!("  {} Login failed: {}", styles.red.apply_to("✗"), e);
            ExitCode::FAILURE
        }
    }
}

fn run_vpn_session(args: &Args, styles: &CliStyles) -> ExitCode {
    if kuvpn::is_openconnect_running() {
        eprintln!(
            "  {} An OpenConnect process is already running. It will be monitored or replaced.",
            styles.yellow.apply_to("!")
        );
    }

    let config = SessionConfig {
        url: args.url.clone(),
        domain: args.domain.clone(),
        user_agent: "Mozilla/5.0".to_string(),
        headless: args.mode.headless(),
        no_auto_login: args.mode.no_auto_login(),
        email: args.email.clone(),
        openconnect_path: args.openconnect_path.clone(),
        escalation_tool: args.run_command.clone(),
        interface_name: args.interface_name.clone(),
        tunnel_mode: match args.tunnel_mode {
            args::CliTunnelMode::Full => kuvpn::TunnelMode::Full,
            args::CliTunnelMode::Manual => kuvpn::TunnelMode::Manual(args.vpnc_script.clone()),
        },
    };

    let mut cli_log_file = kuvpn::get_user_data_dir()
        .ok()
        .and_then(|d| kuvpn::FileLogger::open(d.join("kuvpn.log")));

    let spinner = Arc::new(ProgressBar::new_spinner());
    spinner.set_style(spinner_style());

    let session = VpnSession::new(config);
    let (log_tx, log_rx) = crossbeam_channel::unbounded();
    session.set_logs_tx(log_tx);

    let provider = Arc::new(CliCredentialsProvider {
        spinner: Arc::clone(&spinner),
    });
    let _join_handle = session.connect(provider);

    // Clone the session so the ctrlc handler can call session.cancel(), which
    // also kills the browser process (not just sets the token).
    let cancel_token = session.cancel_token();
    let cancel_session = session.clone();
    ctrlc::set_handler(move || cancel_session.cancel()).ok();

    let mut connection_start: Option<Instant> = None;
    let mut spinner_active = false;

    loop {
        drain_logs(
            &log_rx,
            &spinner,
            &mut spinner_active,
            &mut connection_start,
            styles,
            &args.interface_name,
            &mut cli_log_file,
        );

        if session.is_finished() {
            // One final drain: session.cleanup() sends "Disconnected." before
            // setting status, so there may be messages still in the channel.
            drain_logs(
                &log_rx,
                &spinner,
                &mut spinner_active,
                &mut connection_start,
                styles,
                &args.interface_name,
                &mut cli_log_file,
            );
            clear_spinner(&spinner, &mut spinner_active);
            return if session.status() == ConnectionStatus::Error {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            };
        }

        // Safety net: if the user cancelled but the session thread is still
        // blocked (e.g. waiting on a CDP call to a dead browser), don't spin
        // forever. Give it a brief window to flush any final log messages, then
        // exit. Normally the SIGKILL above makes this unnecessary, but this
        // handles edge cases (PID not yet stored, slow OS socket teardown, etc).
        if cancel_token.is_cancelled() {
            std::thread::sleep(Duration::from_millis(500));
            drain_logs(
                &log_rx,
                &spinner,
                &mut spinner_active,
                &mut connection_start,
                styles,
                &args.interface_name,
                &mut cli_log_file,
            );
            clear_spinner(&spinner, &mut spinner_active);
            return ExitCode::SUCCESS;
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}

fn drain_logs(
    log_rx: &crossbeam_channel::Receiver<String>,
    spinner: &ProgressBar,
    spinner_active: &mut bool,
    connection_start: &mut Option<Instant>,
    styles: &CliStyles,
    interface_name: &str,
    log_file: &mut Option<kuvpn::FileLogger>,
) {
    while let Ok(raw) = log_rx.try_recv() {
        if let Some(ref mut f) = log_file {
            f.write_line(&raw);
        }
        if let Some(path) = raw.strip_prefix("Diagnostic|") {
            clear_spinner(spinner, spinner_active);
            eprintln!(
                "  {} Automation diagnostic saved: {}",
                styles.dim.apply_to("●"),
                path
            );
        } else if let Some(parsed) = ParsedLog::parse(&raw) {
            handle_log(
                &parsed,
                spinner,
                spinner_active,
                connection_start,
                styles,
                interface_name,
            );
        } else {
            log::info!("{}", raw);
        }
    }
}

fn spinner_style() -> ProgressStyle {
    ProgressStyle::default_spinner()
        .template("{spinner:.cyan} {msg}")
        .expect("spinner template is always valid")
}
