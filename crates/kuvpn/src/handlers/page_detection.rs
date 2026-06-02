use super::AuthTab;

impl AuthTab {
    /// Checks if an input element matching `selector` is currently visible.
    pub(crate) fn is_input_visible(&self, selector: &str) -> anyhow::Result<bool> {
        let sel = crate::utils::js_escape(selector);
        self.eval_bool(&format!(
            "(function(){{ var el=document.querySelector('{sel}'); return !!(el && el.offsetParent!==null); }})()"
        ))
    }

    /// Checks if any invalid-username error message is visible.
    pub(crate) fn is_invalid_username_visible(&self) -> anyhow::Result<bool> {
        self.eval_bool(
            r#"!!(document.getElementById('usernameError')
                && (
                    document.getElementById('usernameError').innerText.includes("We couldn't find an account with that username.")
                    || document.getElementById('usernameError').innerText.toLowerCase().includes("enter a valid email address")
                    || document.getElementById('usernameError').innerText.toLowerCase().includes("enter a valid phone number")
                    || document.getElementById('usernameError').innerText.toLowerCase().includes("enter a valid skype name")
                    || document.getElementById('usernameError').innerText.includes("This username may be incorrect")
                    || document.getElementById('usernameError').innerText.includes("Make sure you typed it correctly")
                )
            )"#,
        )
    }

    /// Checks for the softer username warning ("This username may be incorrect…").
    /// Returns the warning text if visible, otherwise `None`.
    pub(crate) fn is_username_warning_visible(&self) -> anyhow::Result<Option<String>> {
        self.eval_string(
            r#"(function() {
            const usernameError = document.getElementById('usernameError');
            if (usernameError) {
                const text = usernameError.innerText;
                if (text.includes("This username may be incorrect") ||
                    text.includes("Make sure you typed it correctly")) {
                    return text;
                }
            }

            const alerts = document.querySelectorAll('[role="alert"], .alert-error, .error-message');
            for (let alert of alerts) {
                const text = alert.innerText;
                if (text.includes("This username may be incorrect") ||
                    text.includes("Make sure you typed it correctly")) {
                    return text;
                }
            }

            return null;
        })()"#,
        )
    }

    /// Checks if the "Your account or password is incorrect" error is visible.
    pub(crate) fn is_incorrect_password_visible(&self) -> anyhow::Result<bool> {
        self.eval_bool(
            "!!(document.getElementById('passwordError') && \
             document.getElementById('passwordError').innerText.includes('Your account or password is incorrect.'))",
        )
    }

    /// Checks if the page is an Azure AD ConvergedError page.
    pub(crate) fn is_azure_error_page(&self) -> anyhow::Result<bool> {
        self.eval_bool(
            r#"(function() {
            const pageMeta = document.querySelector('meta[name="PageID"]');
            return !!(pageMeta && pageMeta.content === 'ConvergedError');
        })()"#,
        )
    }

    /// Extracts the human-readable error details from an Azure AD error page.
    pub(crate) fn get_azure_error_details(&self) -> anyhow::Result<String> {
        self.eval_string_or(
            r#"(function() {
            if (window.$Config) {
                const config = window.$Config;
                const errorCode = config.iErrorCode;
                const mainMessage = config.strMainMessage || '';
                const exceptionMessage = config.strServiceExceptionMessage || '';

                if (errorCode === 900561) {
                    return 'Authentication protocol error. This usually happens after incorrect credentials.\n\nPlease check your username and password and try again.';
                } else if (errorCode === 50126 || errorCode === 50053) {
                    return 'Too many incorrect password attempts.\n\nYour account may be temporarily locked. Please wait a few minutes and try again.';
                } else if (errorCode === 50055) {
                    return 'Password has expired.\n\nPlease reset your password.';
                } else if (exceptionMessage && exceptionMessage.includes('AADSTS')) {
                    return 'Azure AD error: ' + exceptionMessage;
                } else if (mainMessage) {
                    return mainMessage;
                }
                return 'Azure AD error (code: ' + errorCode + ')';
            }
            return null;
        })()"#,
            "Azure AD authentication error occurred",
        )
    }
}
