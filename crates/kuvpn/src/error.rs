/// High-level error category for UI display purposes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Authentication errors (login phase) - username, password, MFA issues
    Authentication,
    /// Connection errors (OpenConnect phase) - VPN connection issues
    Connection,
    /// System errors - browser, network, etc.
    System,
}

/// Error types returned by the authentication flow.
/// Categorized by severity and whether they require manual intervention.
#[derive(Debug, Clone)]
pub enum AuthError {
    /// Simple credential errors that can be fixed by re-entering credentials
    InvalidUsername {
        message: String,
    },
    UsernameWarning {
        warning_text: String,
    },
    IncorrectPassword {
        message: String,
    },

    /// Errors that may require manual mode or session cleanup
    AuthenticationFailed {
        reason: String,
        suggest_manual_mode: bool,
        suggest_clear_cache: bool,
    },

    /// Technical/system errors
    BrowserError {
        message: String,
    },
    Timeout {
        message: String,
    },
    Cancelled,

    /// Unexpected errors
    Unknown {
        message: String,
    },
}

impl AuthError {
    /// Returns the high-level category of this error
    pub fn category(&self) -> ErrorCategory {
        match self {
            AuthError::InvalidUsername { .. }
            | AuthError::UsernameWarning { .. }
            | AuthError::IncorrectPassword { .. }
            | AuthError::AuthenticationFailed { .. } => ErrorCategory::Authentication,
            AuthError::BrowserError { .. } | AuthError::Timeout { .. } | AuthError::Cancelled => {
                ErrorCategory::System
            }
            AuthError::Unknown { .. } => ErrorCategory::System,
        }
    }

    /// Returns true if this is a simple credential error that doesn't need manual mode
    pub fn is_simple_credential_error(&self) -> bool {
        matches!(
            self,
            AuthError::InvalidUsername { .. }
                | AuthError::UsernameWarning { .. }
                | AuthError::IncorrectPassword { .. }
        )
    }

    /// Returns true if manual mode should be suggested
    pub fn should_suggest_manual_mode(&self) -> bool {
        match self {
            AuthError::AuthenticationFailed {
                suggest_manual_mode,
                ..
            } => *suggest_manual_mode,
            AuthError::BrowserError { .. } | AuthError::Timeout { .. } => true,
            _ => false,
        }
    }

    /// Returns true if clearing cache should be suggested
    pub fn should_suggest_clear_cache(&self) -> bool {
        match self {
            AuthError::AuthenticationFailed {
                suggest_clear_cache,
                ..
            } => *suggest_clear_cache,
            AuthError::BrowserError { .. } => true,
            _ => false,
        }
    }

    /// Returns the user-facing error message
    pub fn user_message(&self) -> String {
        match self {
            AuthError::InvalidUsername { message } => {
                format!(
                    "Invalid username.\n\n{}\n\nPlease check your email address and try again.",
                    message
                )
            }
            AuthError::UsernameWarning { warning_text } => {
                format!(
                    "Username verification warning.\n\n{}\n\nPlease verify your email address is correct and try again.",
                    warning_text
                )
            }
            AuthError::IncorrectPassword { message } => {
                format!(
                    "Incorrect password.\n\n{}\n\nPlease re-enter your password and try again.",
                    message
                )
            }
            AuthError::AuthenticationFailed { reason, .. } => {
                format!("Authentication failed.\n\n{}", reason)
            }
            AuthError::BrowserError { message } => {
                format!("Browser error.\n\n{}", message)
            }
            AuthError::Timeout { message } => {
                format!("Connection timeout.\n\n{}", message)
            }
            AuthError::Cancelled => "Operation cancelled by user.".to_string(),
            AuthError::Unknown { message } => {
                format!("An error occurred.\n\n{}", message)
            }
        }
    }

    /// Returns detailed troubleshooting suggestions based on error type
    pub fn troubleshooting_steps(&self) -> Vec<String> {
        let mut steps = Vec::new();

        if self.is_simple_credential_error() {
            // No extra steps needed for simple credential errors
            return steps;
        }

        if self.should_suggest_manual_mode() {
            steps.push("Try switching to manual mode from settings".to_string());
        }

        if self.should_suggest_clear_cache() {
            steps.push("Try clearing the session cache (wipe browser profile)".to_string());
        }

        if matches!(self, AuthError::Unknown { .. }) {
            steps.push("Check console/logs for more details".to_string());
        }

        steps
    }

    /// Returns the full error message with troubleshooting steps
    pub fn full_message(&self) -> String {
        let mut msg = self.user_message();
        let steps = self.troubleshooting_steps();

        if !steps.is_empty() {
            msg.push_str("\n\nTroubleshooting steps:");
            for (i, step) in steps.iter().enumerate() {
                msg.push_str(&format!("\n{}. {}", i + 1, step));
            }
        }

        // Always add "Check console for details" for non-simple errors
        if !self.is_simple_credential_error() {
            msg.push_str("\n\nCheck console/logs for more details.");
        }

        msg
    }
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.full_message())
    }
}

impl std::error::Error for AuthError {}

impl From<anyhow::Error> for AuthError {
    fn from(err: anyhow::Error) -> Self {
        AuthError::Unknown {
            message: err.to_string(),
        }
    }
}
