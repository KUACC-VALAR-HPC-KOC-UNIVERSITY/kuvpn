//! # VPN Connection Tool Library
//!
//! This library provides functionality for fetching DSID cookies from a specified URL
//! using a browser and establishing VPN connections via OpenConnect.

pub mod browser;
pub mod diagnostics;
pub mod dsid;
pub mod error;
pub mod file_logger;
pub mod handlers;
#[cfg(windows)]
pub mod helper;
pub mod history;
pub mod logger;
pub mod openconnect;
pub mod session;
pub mod utils;

// Re-export commonly used items
pub use dsid::{run_login_and_get_dsid, LoginConfig};
pub use error::{AuthError, ErrorCategory};
#[cfg(windows)]
pub use helper::run_vpn_helper_if_requested;
pub use history::{
    append_event, clear_events, format_duration_secs, format_timestamp_unix, load_events,
    ConnectionEvent, EventKind,
};
pub use file_logger::FileLogger;
pub use logger::init_logger;
#[cfg(unix)]
pub use openconnect::{
    find_askpass, is_conflicting_vpn_active, list_available_escalation_tools,
    needs_password_prompt, resolve_escalation_tool,
};
pub use openconnect::{
    get_openconnect_pid, get_vpn_interface_name, is_openconnect_running, is_vpn_interface_up,
    kill_process, locate_openconnect, OpenConnectRunner,
};
pub use session::{ConnectionStatus, ParsedLog, SessionConfig, TunnelMode, VpnSession};
pub use utils::{get_user_data_dir, has_session_data, wipe_user_data_dir};
