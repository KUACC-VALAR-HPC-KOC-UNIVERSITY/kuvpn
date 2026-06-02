use super::AuthTab;
use crate::utils::{CancellationToken, CredentialsProvider};
use std::collections::HashSet;
use std::thread::sleep;
use std::time::Duration;

impl AuthTab {
    /// Shared polling loop for both push-approval variants.
    ///
    /// Reads the display number via `number_js`, notifies the provider, then polls
    /// `still_showing_js` every `poll_interval` until the push page is gone
    /// (element hidden or URL changes) or the operation is cancelled.
    fn poll_mfa_push(
        &self,
        provider: &dyn CredentialsProvider,
        cancel_token: Option<&CancellationToken>,
        number_js: &str,
        still_showing_js: &str,
        poll_interval: Duration,
    ) -> anyhow::Result<()> {
        let number = self.eval_string_or(number_js, "")?;
        provider.on_mfa_push(&number);

        let prev_url = self.get_url();
        loop {
            if let Some(token) = cancel_token {
                if token.is_cancelled() {
                    provider.on_mfa_complete();
                    return Err(anyhow::anyhow!("Operation cancelled by user"));
                }
            }

            sleep(poll_interval);

            let still_showing = self.eval_bool(still_showing_js)?;
            let new_url = self.get_url();
            if !still_showing || new_url != prev_url {
                provider.on_mfa_complete();
                log::info!("Push page finished, moving on...");
                break;
            }
        }

        Ok(())
    }

    /// Handles authenticator push approval (SAOTCAS flow).
    pub(crate) fn handle_authenticator_push_approval(
        &self,
        provider: &dyn CredentialsProvider,
        cancel_token: Option<&CancellationToken>,
    ) -> anyhow::Result<bool> {
        // Structural detection: the SAOTCAS title element is unique to this
        // push-approval flow.  The number display is optional — some push
        // variants just ask the user to tap "Approve" without a number.
        let is_push_page = self.eval_bool(
            r#"(function() {
    var title = document.getElementById('idDiv_SAOTCAS_Title');
    return !!(title && title.offsetParent !== null);
})()"#,
        )?;

        if is_push_page {
            self.poll_mfa_push(
                provider,
                cancel_token,
                r#"(function() {
    var el = document.getElementById('idRichContext_DisplaySign');
    return el ? el.innerText.trim() : '';
})()"#,
                r#"(function() {
    var el = document.getElementById('idRichContext_DisplaySign');
    return !!(el && el.offsetParent !== null);
})()"#,
                Duration::from_secs(1),
            )?;
            return Ok(true);
        }

        Ok(false)
    }

    /// Handles the verification-code choice page ("Verify your identity").
    ///
    /// Prefers the Authenticator app; falls back to SMS, then the first available
    /// proof button when only telephony options (SMS/voice) are offered.
    pub(crate) fn handle_verification_code_choice(&self) -> anyhow::Result<bool> {
        let is_proof_choice_page = self.eval_bool(
            r#"(function() {
    var title = document.getElementById('idDiv_SAOTCS_Title');
    return !!(title && title.innerText.trim().toLowerCase().includes('verify your identity'));
})()"#,
        )?;

        if is_proof_choice_page {
            let selected = self.eval_string(
                r#"(function() {
    // Prefer authenticator / mobile app
    var els = document.querySelectorAll('div[role="button"], .table[role="button"], button, input[type="button"]');
    for (var i = 0; i < els.length; i++) {
        var text = els[i].innerText.toLowerCase();
        if (text.includes('mobile app') || text.includes('authenticator')) {
            els[i].click();
            return 'authenticator';
        }
    }
    // Fall back to SMS
    var sms = document.querySelector('[data-value="OneWaySMS"]');
    if (sms) { sms.click(); return 'sms'; }
    // Fall back to first available proof button
    var first = document.querySelector('#idDiv_SAOTCS_Proofs .table[role="button"]');
    if (first) { first.click(); return 'first_proof'; }
    return null;
})()"#,
            )?;
            match selected.as_deref() {
                Some("authenticator") => log::info!("Proof choice: clicked authenticator"),
                Some("sms") => log::info!("Proof choice: no authenticator available, selected SMS"),
                Some("first_proof") => log::info!("Proof choice: selected first available proof"),
                _ => log::warn!("Proof choice page detected but no proof button found"),
            }
            sleep(Duration::from_millis(500));
            return Ok(true);
        }

        Ok(false)
    }

    /// Handles the "Use an app instead" link.
    pub(crate) fn handle_use_app_instead(&self) -> anyhow::Result<bool> {
        let is_visible = self.eval_bool(
            r#"(function() {
    var el = document.getElementById('idA_PWD_SwitchToRemoteNGC');
    return !!(el && el.offsetParent !== null);
})()"#,
        )?;

        if is_visible {
            self.eval(
                r#"var el=document.getElementById('idA_PWD_SwitchToRemoteNGC'); if(el){el.click();}"#,
            )?;
            log::info!("Clicked 'Use an app instead'");
            sleep(Duration::from_millis(400));
            return Ok(true);
        }

        Ok(false)
    }

    /// Handles authenticator NGC push notifications.
    pub(crate) fn handle_authenticator_ngc_push(
        &self,
        provider: &dyn CredentialsProvider,
        cancel_token: Option<&CancellationToken>,
    ) -> anyhow::Result<bool> {
        // Structural detection: the polling description element indicates an
        // active NGC push.  The number display is optional — some push
        // variants just ask the user to tap "Approve" without a number.
        let is_ngc_push = self.eval_bool(
            r#"(function() {
    var polling = document.getElementById('idDiv_RemoteNGC_PollingDescription');
    return !!(polling && polling.offsetParent !== null);
})()"#,
        )?;

        if is_ngc_push {
            self.poll_mfa_push(
                provider,
                cancel_token,
                r#"(function() {
    var el=document.getElementById('idRemoteNGC_DisplaySign');
    return (el && el.offsetParent !== null) ? el.innerText.trim() : '';
})()"#,
                r#"(function() {
    var num = document.getElementById('idRemoteNGC_DisplaySign');
    var poll = document.getElementById('idDiv_RemoteNGC_PollingDescription');
    var numVisible = num && num.offsetParent !== null;
    var pollVisible = poll && poll.offsetParent !== null;
    return !!(numVisible || pollVisible);
})()"#,
                Duration::from_millis(400),
            )?;
            return Ok(true);
        }

        Ok(false)
    }

    /// Handles OTP/verification code entry (SMS, email, TOTP).
    ///
    /// Detects `input[name="otc"]` which Microsoft uses for all one-time code inputs.
    pub(crate) fn handle_otp_entry(
        &self,
        provider: &dyn CredentialsProvider,
    ) -> anyhow::Result<bool> {
        let is_otp_page = self.eval_bool(
            r#"(function() {
    var input = document.querySelector('input[name="otc"]');
    return !!(input && input.offsetParent !== null);
})()"#,
        )?;

        if is_otp_page {
            let error_text = self.eval_string(
                r#"(function() {
    var el = document.getElementById('idSpan_SAOTCC_Error_OTC');
    if (el && el.offsetParent !== null) {
        var t = el.innerText.trim();
        if (t.length > 0) return t;
    }
    return null;
})()"#,
            )?;

            let prompt = self
                .eval_string(
                    r#"(function() {
    var desc = document.getElementById('idDiv_SAOTCC_Description')
        || document.getElementById('idDiv_SAOTCS_Description');
    if (desc && desc.innerText.trim().length > 0) return desc.innerText.trim();
    var title = document.getElementById('idDiv_SAOTCC_Title')
        || document.getElementById('idDiv_SAOTCS_Title');
    if (title && title.innerText.trim().length > 0) return title.innerText.trim();
    return null;
})()"#,
                )?
                .map(|s| format!("{}: ", s))
                .unwrap_or_else(|| "Enter verification code: ".to_string());

            let prompt = match error_text {
                Some(err) => format!("{}\n\n{}", err, prompt),
                None => prompt,
            };

            log::info!("OTP entry page detected, requesting code from user");

            // Inject a watcher that flags when the OTP input disappears
            self.inject_input_watcher(r#"input[name="otc"]"#);

            let code = match provider.request_text(&prompt) {
                Some(c) => c,
                None => {
                    self.clear_input_watcher();
                    return Ok(false); // prompt dismissed (page changed)
                }
            };

            self.clear_input_watcher();
            self.fill_input_value(r#"input[name="otc"]"#, &code)?;
            sleep(Duration::from_millis(250));

            // Try the dedicated OTP submit button first, then fall back to Next
            self.eval(
                r#"(function() {
    var btn = document.querySelector('#idSubmit_SAOTCC_Continue')
        || document.querySelector('#idSubmit_SAOTCS_Continue')
        || document.querySelector('#idSIButton9');
    if (btn) { btn.focus(); btn.click(); }
})()"#,
            )?;

            return Ok(true);
        }

        Ok(false)
    }

    /// Handles NGC error and switches to password authentication.
    pub(crate) fn handle_ngc_error_use_password(
        &self,
        handled: &mut HashSet<&'static str>,
    ) -> anyhow::Result<bool> {
        let is_ngc_error = self.eval_bool(
            r#"(function() {
    var header = document.getElementById('loginHeader');
    var errorBlock = document.getElementById('idDiv_RemoteNGC_PageDescription');
    var pollingIndicator = document.getElementById('idDiv_RemoteNGC_PollingDescription');
    var pollingActive = pollingIndicator && pollingIndicator.offsetParent !== null;

    var textMatch = (
        (header && header.innerText.toLowerCase().includes("request wasn't sent")) ||
        (errorBlock && errorBlock.innerText.toLowerCase().includes("couldn't send"))
    );

    var structuralMatch = (
        errorBlock && errorBlock.offsetParent !== null && !pollingActive
    );

    return !!(textMatch || structuralMatch);
})()"#,
        )?;

        if is_ngc_error {
            let clicked = self
                .eval_string(
                    r#"(function() {
    var selectors = [
        '#idA_PWD_SwitchToPassword',
        '#signInAnotherWay',
        '#idA_PWD_SwitchToCredPicker'
    ];
    for (var i = 0; i < selectors.length; i++) {
        var el = document.querySelector(selectors[i]);
        if (el && el.offsetParent !== null) {
            el.click();
            return selectors[i];
        }
    }
    return null;
})()"#,
                )?
                .unwrap_or_default();

            if !clicked.is_empty() {
                log::info!("NGC error page, switching to password via {}", clicked);
                handled.insert("use_app_instead");
                sleep(Duration::from_millis(400));
                return Ok(true);
            }
        }

        Ok(false)
    }
}
