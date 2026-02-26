//! # KUVPN CLI
//!
//! A clean terminal interface for connecting to Koc University's VPN.

mod args;

use args::Args;
use clap::Parser;
use console::{Key, Style, Term};
use dialoguer::Input;
use indicatif::{ProgressBar, ProgressStyle};
use kuvpn::utils::CredentialsProvider;
use kuvpn::{
    init_logger, run_login_and_get_dsid, ConnectionStatus, ParsedLog, SessionConfig, VpnSession,
};
use log::info;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Instant;

/// Format a duration into a human-readable string like "1h 23m 45s".
fn format_duration(duration: std::time::Duration) -> String {
    let total_secs = duration.as_secs();
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

/// Reads a password from stdin with masked output (asterisks).
fn read_masked_password(prompt: &str) -> String {
    let term = Term::stderr();
    let _ = term.write_str(prompt);
    let _ = term.write_str(": ");
    let _ = term.flush();

    let mut password = String::new();

    while let Ok(key) = term.read_key() {
        match key {
            Key::Enter => {
                break;
            }
            Key::Backspace => {
                if !password.is_empty() {
                    password.pop();
                    let _ = term.clear_chars(1);
                    let _ = term.flush();
                }
            }
            Key::Char(c) => {
                if c == '\x03' {
                    // Ctrl+C
                    std::process::exit(130);
                }
                password.push(c);
                let _ = term.write_str("*");
                let _ = term.flush();
            }
            _ => {}
        }
    }
    password
}

/// CLI credentials provider that can suspend the spinner before prompting.
struct CliCredentialsProvider {
    spinner: Arc<ProgressBar>,
}

impl CredentialsProvider for CliCredentialsProvider {
    fn request_text(&self, msg: &str) -> String {
        self.spinner.finish_and_clear();
        let result = Input::new()
            .with_prompt(msg.trim_end_matches(": ").trim_end_matches(':'))
            .interact_text()
            .unwrap_or_default();
        // Clean up the prompt line after input
        let term = Term::stderr();
        let _ = term.clear_last_lines(1);
        result
    }

    fn request_password(&self, msg: &str) -> String {
        self.spinner.finish_and_clear();
        let result = read_masked_password(msg.trim_end_matches(": ").trim_end_matches(':'));
        // Clean up the prompt line after input
        let term = Term::stderr();
        let _ = term.clear_line();
        let _ = term.write_str("\r");
        result
    }

    fn on_mfa_push(&self, code: &str) {
        self.spinner.finish_and_clear();
        let bold = Style::new().bold();
        let cyan = Style::new().cyan().bold();
        eprintln!(
            "{} Enter {} in Authenticator",
            bold.apply_to(">>"),
            cyan.apply_to(code),
        );
    }

    fn on_mfa_complete(&self) {
        let term = Term::stderr();
        let _ = term.clear_last_lines(1);
    }
}

/// The main entry point of the application.
fn main() -> ExitCode {
    let args = Args::parse();
    init_logger(args.level.clone().into());

    let green = Style::new().green().bold();
    let red = Style::new().red().bold();
    let dim = Style::new().dim();
    let bold = Style::new().bold();
    let yellow = Style::new().yellow().bold();

    // Print banner
    eprintln!(
        "{} {}",
        bold.apply_to("KUVPN"),
        dim.apply_to(format!("v{}", env!("CARGO_PKG_VERSION"))),
    );

    // Ensure only one instance is running
    if let Err(e) = kuvpn::utils::ensure_single_instance() {
        eprintln!("  {} {}", red.apply_to("✗"), e);
        return ExitCode::FAILURE;
    }

    // Handle the clean session option.
    if args.clean {
        match kuvpn::utils::wipe_user_data_dir() {
            Ok(_) => {
                eprintln!("  {} Session data wiped", green.apply_to("✓"));
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                eprintln!("  {} Failed to wipe session data: {}", red.apply_to("✗"), e);
                return ExitCode::FAILURE;
            }
        }
    }

    if args.get_dsid {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        spinner.set_message("Retrieving DSID...");
        spinner.enable_steady_tick(std::time::Duration::from_millis(80));

        match run_login_and_get_dsid(
            !args.disable_headless,
            &args.url,
            &args.domain,
            "Mozilla/5.0",
            args.no_auto_login,
            args.email,
            &kuvpn::utils::TerminalCredentialsProvider,
            None,
            None,
        ) {
            Ok(dsid) => {
                spinner.finish_and_clear();
                println!("{}", dsid);
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                spinner.finish_and_clear();
                eprintln!("  {} Login failed: {}", red.apply_to("✗"), e);
                return ExitCode::FAILURE;
            }
        }
    }

    let config = SessionConfig {
        url: args.url.clone(),
        domain: args.domain.clone(),
        user_agent: "Mozilla/5.0".to_string(),
        headless: !args.disable_headless,
        no_auto_login: args.no_auto_login,
        email: args.email.clone(),
        #[cfg(not(windows))]
        openconnect_path: args.openconnect_path.clone(),
        escalation_tool: args.run_command.clone(),
        interface_name: args.interface_name.clone(),
    };

    // Create a shared spinner so the credentials provider can suspend it
    let spinner = Arc::new(ProgressBar::new_spinner());
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );

    let session = VpnSession::new(config);
    let (log_tx, log_rx) = crossbeam_channel::unbounded();
    session.set_logs_tx(log_tx);

    let provider = Arc::new(CliCredentialsProvider {
        spinner: Arc::clone(&spinner),
    });
    let _join_handle = session.connect(provider);

    // Set up Ctrl+C handler for graceful disconnect
    let cancel_session = session.cancel_token();
    ctrlc::set_handler(move || {
        cancel_session.cancel();
    })
    .ok();

    let mut connection_start: Option<Instant> = None;
    let mut spinner_active = false;

    loop {
        while let Ok(log_msg) = log_rx.try_recv() {
            if let Some(parsed) = ParsedLog::parse(&log_msg) {
                let msg = &parsed.message;

                // Handle key status messages directly (bypass log system)
                match msg.as_str() {
                    "Accessing campus gateway..." => {
                        if !spinner_active {
                            spinner.enable_steady_tick(std::time::Duration::from_millis(80));
                            spinner_active = true;
                        }
                        spinner.set_message("Accessing campus gateway...");
                        continue;
                    }
                    "Initializing tunnel..." => {
                        if spinner_active {
                            spinner.finish_and_clear();
                            eprintln!("  {} Accessing campus gateway...", green.apply_to("✓"),);
                            spinner_active = false;
                        }
                        eprintln!("  {} Initializing tunnel...", green.apply_to("✓"),);
                        continue;
                    }
                    "VPN interface already active, monitoring..." => {
                        if spinner_active {
                            spinner.finish_and_clear();
                            spinner_active = false;
                        }
                        eprintln!(
                            "  {} VPN already active, monitoring connection",
                            yellow.apply_to("~"),
                        );
                        connection_start = Some(Instant::now());
                        continue;
                    }
                    "Connected." => {
                        if spinner_active {
                            spinner.finish_and_clear();
                            spinner_active = false;
                        }
                        connection_start = Some(Instant::now());

                        #[cfg(unix)]
                        {
                            let iface = kuvpn::get_vpn_interface_name(&args.interface_name)
                                .unwrap_or_else(|| args.interface_name.clone());
                            eprintln!(
                                "  {} Connected to KU VPN {}",
                                green.apply_to("✓"),
                                dim.apply_to(format!("(interface: {})", iface)),
                            );
                        }
                        #[cfg(not(unix))]
                        eprintln!("  {} Connected to KU VPN", green.apply_to("✓"),);

                        eprintln!("    {}", dim.apply_to("Press Ctrl+C to disconnect"),);
                        continue;
                    }
                    "Disconnecting..." => {
                        if spinner_active {
                            spinner.finish_and_clear();
                        }
                        spinner.set_message("Disconnecting...");
                        spinner.enable_steady_tick(std::time::Duration::from_millis(80));
                        spinner_active = true;
                        continue;
                    }
                    "Disconnected." => {
                        if spinner_active {
                            spinner.finish_and_clear();
                            spinner_active = false;
                        }
                        let duration_str = connection_start
                            .map(|s| format!(" (session: {})", format_duration(s.elapsed())))
                            .unwrap_or_default();
                        eprintln!(
                            "  {} Disconnected{}",
                            dim.apply_to("●"),
                            dim.apply_to(duration_str),
                        );
                        continue;
                    }
                    _ => {
                        // Suppress password prompt log messages (provider handles the prompt)
                        if msg.ends_with("requires a password. Prompting...") {
                            continue;
                        }
                    }
                }

                // Handle errors - always print
                if parsed.level == log::Level::Error {
                    if spinner_active {
                        spinner.finish_and_clear();
                        spinner_active = false;
                    }
                    eprintln!("  {} {}", red.apply_to("✗"), msg);

                    // Show recovery suggestions for auth errors
                    if msg.contains("Full Auto mode unable to complete login")
                        || msg.contains("Could not find a handler")
                    {
                        eprintln!();
                        eprintln!("  {} Try the following:", bold.apply_to("Suggestions:"));
                        eprintln!(
                            "    {} Switch to manual mode: {}",
                            dim.apply_to("•"),
                            bold.apply_to("--no-auto-login --disable-headless"),
                        );
                        eprintln!(
                            "    {} Wipe session cache:    {}",
                            dim.apply_to("•"),
                            bold.apply_to("--clean"),
                        );
                    }
                    continue;
                }

                // Other messages go through the log system
                match parsed.level {
                    log::Level::Warn => log::warn!("{}", msg),
                    _ => info!("{}", msg),
                }
            } else {
                info!("{}", log_msg);
            }
        }

        if session.is_finished() {
            if spinner_active {
                spinner.finish_and_clear();
            }
            if session.status() == ConnectionStatus::Error {
                return ExitCode::FAILURE;
            }
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    ExitCode::SUCCESS
}
