//! # VPN Connection Tool Library
//!
//! This library provides functionality for fetching DSID cookies from a specified URL
//! using a browser and establishing VPN connections via OpenConnect.

pub mod browser;
pub mod dsid;
pub mod error;
pub mod handlers;
pub mod logger;
pub mod openconnect;
pub mod session;
pub mod utils;
#[cfg(windows)]
pub mod win_elevated;

// Re-export commonly used items
pub use dsid::run_login_and_get_dsid;
pub use error::{AuthError, ErrorCategory};
pub use logger::init_logger;
pub use openconnect::{
    execute_openconnect, get_openconnect_pid, get_vpn_interface_name, is_openconnect_running,
    is_vpn_interface_up, kill_process, locate_openconnect,
};
#[cfg(unix)]
pub use openconnect::{
    find_askpass, list_available_escalation_tools, needs_password_prompt, resolve_escalation_tool,
};
pub use session::{ConnectionStatus, ParsedLog, SessionConfig, VpnSession};
pub use utils::get_user_data_dir;
