use crate::browser::create_browser;
use crate::error::AuthError;
use crate::handlers::AuthTab;
use crate::utils::{CancellationToken, CredentialsProvider};
use headless_chrome::{Browser, Tab};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};

/// Configuration for the browser-based login process.
pub struct LoginConfig {
    pub headless: bool,
    pub url: String,
    pub domain: String,
    pub user_agent: String,
    pub no_auto_login: bool,
    pub email: Option<String>,
}

// ── Private implementation ────────────────────────────────────────────────────

fn get_initial_tab(browser: &Browser) -> anyhow::Result<Arc<Tab>> {
    for _ in 0..10 {
        if let Ok(tabs) = browser.get_tabs().lock() {
            if let Some(t) = tabs.first() {
                return Ok(Arc::clone(t));
            }
        }
        sleep(Duration::from_millis(200));
    }
    browser.new_tab()
}

struct BrowserSession {
    browser: Browser,
    tab: AuthTab,
}

impl BrowserSession {
    fn open(config: &LoginConfig) -> anyhow::Result<Self> {
        let browser = create_browser(&config.user_agent, config.headless, config.no_auto_login)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let raw_tab = get_initial_tab(&browser)?;

        // Force English Accept-Language at the protocol level, overriding OS locale.
        // Azure AD uses this header to pick the UI language; if it's Turkish the
        // English text checks in the automation handlers will all fail.
        raw_tab.call_method(
            headless_chrome::protocol::cdp::Emulation::SetUserAgentOverride {
                user_agent: config.user_agent.clone(),
                accept_language: Some("en-US,en".to_string()),
                platform: None,
                user_agent_metadata: None,
            },
        )?;

        raw_tab.set_default_timeout(Duration::from_secs(30));
        Ok(Self {
            browser,
            tab: AuthTab::new(raw_tab),
        })
    }

    fn close(self) {
        if let Ok(tabs) = self.browser.get_tabs().lock() {
            for tab in tabs.iter() {
                // We don't want to hang here if the browser is already dead.
                let _ = tab.close(true);
            }
        }
        sleep(Duration::from_millis(200));
    }

    fn setup_page_guard(&self, provider: &dyn CredentialsProvider) {
        self.tab
            .0
            .evaluate("window.__kuvpn_input_gone = false;", false)
            .ok();

        let tab_for_guard = std::sync::Arc::clone(&self.tab.0);
        let guard_url = self.tab.get_url();
        provider.set_page_guard(Box::new(move || {
            if tab_for_guard.get_url() != guard_url {
                return false;
            }
            let gone = tab_for_guard
                .evaluate("window.__kuvpn_input_gone === true", false)
                .ok()
                .and_then(|r| r.value)
                .and_then(|v| v.as_bool())
                .unwrap_or(true); // eval failure → assume page changed
            !gone
        }));
    }

    fn try_handle_page(
        &self,
        handled: &mut HashSet<&'static str>,
        email: Option<&String>,
        provider: &dyn CredentialsProvider,
        cancel_token: Option<&CancellationToken>,
    ) -> anyhow::Result<(bool, bool)> {
        // Returns (handler_matched, is_mfa_handler)
        if !handled.contains("pick_account") && self.tab.handle_pick_account()? {
            handled.insert("pick_account");
            return Ok((true, false));
        }

        if !handled.contains("session_conflict") && self.tab.handle_session_conflict()? {
            handled.insert("session_conflict");
            return Ok((true, false));
        }

        // Check for Azure AD error pages (ConvergedError)
        // This must be checked early as it indicates a fatal authentication error
        if self.tab.is_azure_error_page()? {
            handled.insert("azure_error");
            let error_msg = self.tab.get_azure_error_details()?;
            log::warn!("Azure AD error page detected: {}", error_msg);
            return Err(AuthError::AuthenticationFailed {
                reason: error_msg,
                suggest_manual_mode: false, // This is often a simple credential error
                suggest_clear_cache: true,
            }
            .into());
        }

        if self.tab.is_invalid_username_visible()? {
            handled.insert("invalid_username");
            return Err(AuthError::InvalidUsername {
                message: "The username you entered may be incorrect or the account does not exist."
                    .to_string(),
            }
            .into());
        }

        // Check for username warning (softer error that suggests potential typo)
        if let Some(warning_text) = self.tab.is_username_warning_visible()? {
            handled.insert("username_warning");
            return Err(AuthError::UsernameWarning {
                warning_text: warning_text.trim().to_string(),
            }
            .into());
        }

        if self.tab.is_incorrect_password_visible()? {
            handled.insert("incorrect_password");
            log::warn!("Incorrect password detected");
            return Err(AuthError::IncorrectPassword {
                message: "Your account or password is incorrect.".to_string(),
            }
            .into());
        }

        if self.tab.handle_remote_ngc_denied_next()? {
            handled.remove("ngc_push");
            return Ok((true, false));
        }

        if !handled.contains("username") && self.tab.is_input_visible("input[name=\"loginfmt\"]")? {
            self.tab.fill_on_screen_and_click(
                "input[name=\"loginfmt\"]",
                "Username (email): ",
                "#idSIButton9",
                false,
                true,
                email,
                provider,
            )?;
            handled.insert("username");
            return Ok((true, false));
        }

        if !handled.contains("ngc_error_use_password")
            && self.tab.handle_ngc_error_use_password(handled)?
        {
            handled.insert("ngc_error_use_password");
            return Ok((true, false));
        }

        if !handled.contains("use_app_instead") && self.tab.handle_use_app_instead()? {
            handled.insert("use_app_instead");
            return Ok((true, false));
        }

        if !handled.contains("ngc_push")
            && self
                .tab
                .handle_authenticator_ngc_push(provider, cancel_token)?
        {
            handled.insert("ngc_push");
            return Ok((true, true)); // MFA handler
        }

        if !handled.contains("password") && self.tab.is_input_visible("input[name=\"passwd\"]")? {
            self.tab.fill_on_screen_and_click(
                "input[name=\"passwd\"]",
                "KU Password: ",
                "#idSIButton9",
                true,
                false,
                None,
                provider,
            )?;
            handled.insert("password");
            return Ok((true, false));
        }

        if !handled.contains("kmsi") && self.tab.click_kmsi_if_present()? {
            handled.insert("kmsi");
            return Ok((true, false));
        }

        if !handled.contains("push")
            && self
                .tab
                .handle_authenticator_push_approval(provider, cancel_token)?
        {
            handled.insert("push");
            return Ok((true, true)); // MFA handler
        }

        if !handled.contains("verification_code") && self.tab.handle_verification_code_choice()? {
            handled.insert("verification_code");
            return Ok((true, false));
        }

        // Handle OTP/code entry pages (SMS, email, TOTP from authenticator app).
        // Must run before detect_generic_error, as the code-entry page contains instructional
        // text in aria-live regions that would otherwise trigger a false-positive error.
        // Not added to `handled`: if the user enters a wrong code the page stays at the same
        // URL and we need to re-prompt (the handler itself blocks on user input, so it won't
        // loop without user interaction).
        if self.tab.handle_otp_entry(provider)? {
            return Ok((true, false));
        }

        // Generic fallback detection for unexpected errors or page states
        // This catches scenarios we haven't explicitly coded for
        if !handled.contains("generic_error") {
            if let Some(error_msg) = self.tab.detect_generic_error()? {
                handled.insert("generic_error");
                log::warn!("Generic error detected: {}", error_msg);
                return Err(AuthError::AuthenticationFailed {
                    reason: format!("Unexpected error encountered:\n\n{}", error_msg),
                    suggest_manual_mode: true,
                    suggest_clear_cache: true,
                }
                .into());
            }
        }

        if !handled.contains("unexpected_state") {
            if let Some(state_msg) = self.tab.detect_unexpected_page_state()? {
                handled.insert("unexpected_state");
                log::warn!("Unexpected page state: {}", state_msg);
                return Err(AuthError::AuthenticationFailed {
                    reason: format!("Unexpected page state:\n\n{}", state_msg),
                    suggest_manual_mode: true,
                    suggest_clear_cache: false,
                }
                .into());
            }
        }

        Ok((false, false))
    }

    /// Captures a diagnostic snapshot and saves it to disk, then stores the
    /// saved path in the thread-local slot for the session thread to pick up.
    fn capture_and_save_diagnostics(&self, error: &str) {
        let bundle = self.tab.capture_diagnostics(error);
        match bundle.save() {
            Ok(path) => {
                log::warn!("Diagnostic bundle saved to: {:?}", path);
                crate::diagnostics::PENDING_DIAG_PATH.with(|cell| {
                    *cell.borrow_mut() = Some(path);
                });
            }
            Err(e) => {
                log::warn!("Failed to save diagnostic bundle: {}", e);
            }
        }
    }

    fn run_login(
        &self,
        config: &LoginConfig,
        provider: &dyn CredentialsProvider,
        cancel_token: Option<&CancellationToken>,
    ) -> anyhow::Result<String> {
        const MAX_RETRIES: usize = 20;
        const STUCK_DURATION: Duration = Duration::from_secs(3);
        const MAX_RESETS: usize = 2; // maximum page resets allowed

        log::info!("Navigating to: {}", config.url);
        self.tab.0.navigate_to(&config.url)?;

        if let Err(e) = self.tab.0.wait_until_navigated() {
            log::warn!("Initial navigation wait timed out: {}, continuing...", e);
        }

        let mut handled: HashSet<&'static str> = HashSet::new();
        let mut last_url = String::new();
        let mut retries = 0;
        let mut reset_count = 0;
        let mut stuck_since: Option<Instant> = None;

        loop {
            if let Some(token) = cancel_token {
                if token.is_cancelled() {
                    log::info!("Cancellation requested, closing browser.");
                    return Err(AuthError::Cancelled.into());
                }
            }

            // Use poll_dsid as a heartbeat too. If it fails, the browser is likely gone.
            match self.tab.poll_dsid(&config.domain) {
                Ok(Some(dsid)) => {
                    log::info!("Found valid DSID, quitting.");
                    return Ok(dsid);
                }
                Ok(None) => {} // Keep going
                Err(e) => {
                    log::warn!("Browser heartbeat lost (manual close?): {}", e);
                    return Err(AuthError::BrowserError {
                        message: format!("Browser connection lost: {}", e),
                    }
                    .into());
                }
            }

            let current_url = self.tab.get_url();
            if current_url != last_url {
                log::info!("Page: {}", current_url);
                last_url = current_url;
                retries = 0;
                stuck_since = None;
                handled.clear(); // Allow re-authentication if Microsoft loops back
            }

            if !config.no_auto_login {
                // Page guard: dismiss prompts when the page changes underneath.
                // Microsoft login is an SPA (URL constant), so the actual input
                // watcher is injected by fill_on_screen_and_click / handle_otp_entry
                // right before they block on a prompt. They set
                // window.__kuvpn_input_gone = true when their input disappears.
                // Here we just reset the flag and wire up the guard to read it.
                self.setup_page_guard(provider);

                let is_in_mfa_wait = match self.try_handle_page(
                    &mut handled,
                    config.email.as_ref(),
                    provider,
                    cancel_token,
                ) {
                    Ok((true, is_mfa)) => {
                        retries = 0;
                        stuck_since = None;
                        is_mfa
                    }
                    Ok((false, _)) => {
                        retries += 1;
                        if stuck_since.is_none() {
                            stuck_since = Some(Instant::now());
                        }
                        false
                    }
                    Err(e) => {
                        log::warn!("Handler error: {}", e);
                        let cancelled = cancel_token.is_some_and(|t| t.is_cancelled());
                        if !cancelled {
                            self.capture_and_save_diagnostics(&e.to_string());
                        }
                        return Err(e);
                    }
                };

                provider.clear_page_guard();

                // Page reset logic for stuck automation (only in Full Auto mode)
                let is_stuck = stuck_since
                    .map(|t| t.elapsed() >= STUCK_DURATION)
                    .unwrap_or(false);
                if is_stuck && !is_in_mfa_wait {
                    reset_count += 1;
                    stuck_since = None;

                    if reset_count > MAX_RESETS {
                        let reason = format!(
                            "Full Auto mode unable to complete login after {} page resets. \
                            The authentication flow may have changed or network issues occurred.",
                            MAX_RESETS
                        );
                        self.capture_and_save_diagnostics(&reason);
                        return Err(AuthError::AuthenticationFailed {
                            reason,
                            suggest_manual_mode: true,
                            suggest_clear_cache: true,
                        }
                        .into());
                    }

                    log::warn!(
                        "Authentication stuck (no progress for {}s). Resetting page... (attempt {}/{})",
                        STUCK_DURATION.as_secs(),
                        reset_count,
                        MAX_RESETS
                    );

                    // Log current page state to help diagnose the issue
                    if let Err(e) = self.tab.log_page_state() {
                        log::debug!("Failed to log page state: {}", e);
                    }

                    // Navigate back to initial login URL
                    if let Err(e) = self.tab.0.navigate_to(&config.url) {
                        log::warn!(
                            "Reset navigation failed: {}, continuing with current page",
                            e
                        );
                    } else {
                        let _ = self.tab.0.wait_until_navigated();
                        handled.clear(); // Clear handler history to allow re-triggering
                        retries = 0;
                        stuck_since = None;
                        last_url = String::new();
                    }
                }

                if retries > MAX_RETRIES {
                    return Err(AuthError::AuthenticationFailed {
                        reason: format!(
                            "Could not find a handler for the current page after {} retries: {}",
                            MAX_RETRIES, last_url
                        ),
                        suggest_manual_mode: true,
                        suggest_clear_cache: false,
                    }
                    .into());
                }
            }

            sleep(Duration::from_millis(400));
        }
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Runs the browser-based login flow and returns the DSID cookie on success.
pub fn run_login_and_get_dsid(
    config: &LoginConfig,
    provider: &dyn CredentialsProvider,
    cancel_token: Option<CancellationToken>,
    browser_pid_out: Option<Arc<Mutex<Option<u32>>>>,
) -> anyhow::Result<String> {
    let session = BrowserSession::open(config).map_err(|e| AuthError::BrowserError {
        message: format!("Failed to create browser: {}", e),
    })?;

    if let Some(ref pid_holder) = browser_pid_out {
        if let Some(pid) = session.browser.get_process_id() {
            if let Ok(mut guard) = pid_holder.lock() {
                *guard = Some(pid);
            }
        }
    }

    let result = session.run_login(config, provider, cancel_token.as_ref());
    session.close();
    result
}
