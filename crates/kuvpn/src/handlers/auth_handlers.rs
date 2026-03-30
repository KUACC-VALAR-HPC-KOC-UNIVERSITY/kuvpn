use super::AuthTab;
use crate::utils::CredentialsProvider;
use std::thread::sleep;
use std::time::Duration;

impl AuthTab {
    /// Fills an input field on screen and clicks a button.
    pub(crate) fn fill_on_screen_and_click(
        &self,
        input_selector: &str,
        msg: &str,
        button_selector: &str,
        is_password: bool,
        is_email: bool,
        value: Option<&String>,
        provider: &dyn CredentialsProvider,
    ) -> anyhow::Result<()> {
        if self.is_input_visible(input_selector)? {
            let value = if let Some(v) = value {
                v.to_owned()
            } else {
                // Inject a watcher that flags when THIS input disappears.
                // The page guard (set in dsid.rs) reads window.__kuvpn_input_gone
                // and dismisses the prompt if it becomes true.
                self.inject_input_watcher(input_selector);

                let result = if is_password {
                    provider.request_password(msg)
                } else if is_email {
                    provider.request_email(msg)
                } else {
                    provider.request_text(msg)
                };

                // Clean up the watcher regardless of outcome
                self.clear_input_watcher();

                match result {
                    Some(v) => v,
                    None => return Ok(()), // prompt dismissed (page changed)
                }
            };

            self.fill_input_value(input_selector, &value)?;
            sleep(Duration::from_millis(250));

            let js_btn = format!(
                "var btn=document.querySelector('{btn}'); if(btn){{btn.focus();btn.click();}}",
                btn = button_selector
            );
            self.eval(&js_btn)?;
        }

        Ok(())
    }

    /// Clicks the "Keep me signed in" button if present.
    pub(crate) fn click_kmsi_if_present(&self) -> anyhow::Result<bool> {
        let visible = self.eval_bool(
            r#"(function() {
    var btn = document.querySelector('#idSIButton9');
    return !!(btn && btn.offsetParent !== null && btn.value === 'Yes');
})()"#,
        )?;

        if visible {
            log::info!("Detected KMSI – pressing Yes...");
            self.eval(
                "var chk=document.querySelector('#KmsiCheckboxField'); \
                 if(chk && !chk.checked){chk.click();}",
            )?;
            self.eval(
                "var btn=document.querySelector('#idSIButton9'); \
                 if(btn){btn.focus();btn.click();}",
            )?;
            sleep(Duration::from_millis(500));
            return Ok(true);
        }

        Ok(false)
    }

    /// Handles the session-conflict page (existing VPN session detected).
    pub(crate) fn handle_session_conflict(&self) -> anyhow::Result<bool> {
        let is_conflict_page = self.eval_bool(
            r#"(function() {
    var form = document.querySelector('#DSIDConfirmForm');
    var btn = document.querySelector('#btnContinue');
    return !!(form && btn);
})()"#,
        )?;

        if is_conflict_page {
            log::info!("Detected existing VPN session. Continuing...");
            self.eval(r#"var btn=document.getElementById('btnContinue'); if(btn){btn.click();}"#)?;
            sleep(Duration::from_millis(500));
            return Ok(true);
        }

        Ok(false)
    }

    /// Detects the "Request denied" Authenticator page and presses Next.
    pub(crate) fn handle_remote_ngc_denied_next(&self) -> anyhow::Result<bool> {
        let is_denied_page = self.eval_bool(
            r#"(function() {
            var header = document.getElementById('loginHeader');
            var desc = document.getElementById('idDiv_RemoteNGC_PageDescription');
            var form = document.getElementById('i0281');
            var btn = document.getElementById('idSIButton9');
            return !!(
                form && btn &&
                header &&
                header.innerText.trim().toLowerCase() === "request denied" &&
                desc && desc.innerText.toLowerCase().includes("but you denied it")
            );
        })()"#,
        )?;

        if is_denied_page {
            log::info!("Authenticator denied page detected. Pressing Next...");
            self.eval(
                "var btn=document.getElementById('idSIButton9'); \
                 if(btn){btn.focus();btn.click();}",
            )?;
            sleep(Duration::from_millis(500));
            return Ok(true);
        }

        Ok(false)
    }

    /// Detects the "Pick an account" picker and selects the first account.
    pub(crate) fn handle_pick_account(&self) -> anyhow::Result<bool> {
        let is_picker = self.eval_bool(
            r#"(function() {
            var header = document.getElementById('loginHeader');
            var form = document.getElementById('i0281');
            var tiles = document.querySelectorAll(
                '#tilesHolder .tile[role="listitem"], #tilesHolder .tile-container .table[role="button"]'
            );
            return !!(
                form &&
                header &&
                header.innerText.trim().toLowerCase() === "pick an account" &&
                tiles.length > 0
            );
        })()"#,
        )?;

        if is_picker {
            log::info!("'Pick an account' page detected. Selecting the first account...");
            self.eval(
                r#"(function() {
                var btn = document.querySelector(
                    '#tilesHolder .tile-container .table[role="button"], \
                     #tilesHolder .tile[role="listitem"] .table[role="button"]'
                );
                if (btn) { btn.focus(); btn.click(); }
            })()"#,
            )?;
            sleep(Duration::from_millis(500));
            return Ok(true);
        }

        Ok(false)
    }
}
