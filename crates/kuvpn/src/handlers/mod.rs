pub mod auth_handlers;
pub mod generic_detection;
pub mod mfa_handlers;
pub mod page_detection;

use headless_chrome::Tab;
use std::sync::Arc;

/// Thin wrapper around a Chrome tab that hosts every authentication handler
/// as a method.  Eliminates the repeating `(tab: &Tab, ...)` first argument
/// across all handler functions and consolidates the low-level JS helpers.
pub(crate) struct AuthTab(pub(crate) Arc<Tab>);

impl AuthTab {
    pub(crate) fn new(tab: Arc<Tab>) -> Self {
        Self(tab)
    }

    // ── Low-level JS evaluation helpers ──────────────────────────────────

    /// Evaluates JS and returns the boolean result.
    /// Returns `false` if the script returns null/undefined.
    pub(crate) fn eval_bool(&self, js: &str) -> anyhow::Result<bool> {
        Ok(self
            .0
            .evaluate(js, false)?
            .value
            .and_then(|v| v.as_bool())
            .unwrap_or(false))
    }

    /// Evaluates JS and returns the string result, or `None` if null/undefined.
    pub(crate) fn eval_string(&self, js: &str) -> anyhow::Result<Option<String>> {
        Ok(self
            .0
            .evaluate(js, false)?
            .value
            .and_then(|v| v.as_str().map(|s| s.to_string())))
    }

    /// Evaluates JS and returns the string result, or `fallback` if null/undefined.
    pub(crate) fn eval_string_or(&self, js: &str, fallback: &str) -> anyhow::Result<String> {
        Ok(self
            .eval_string(js)?
            .unwrap_or_else(|| fallback.to_string()))
    }

    /// Evaluates JS for its side effects; propagates errors, discards the return value.
    pub(crate) fn eval(&self, js: &str) -> anyhow::Result<()> {
        self.0.evaluate(js, false)?;
        Ok(())
    }

    // ── Tab state helpers ─────────────────────────────────────────────────

    pub(crate) fn get_url(&self) -> String {
        self.0.get_url()
    }

    /// Captures a diagnostic snapshot of the current browser state.
    ///
    /// Collects URL, page title, full HTML, and the triggering error message.
    /// The result can be saved to disk with [`crate::diagnostics::DiagnosticBundle::save`].
    pub(crate) fn capture_diagnostics(&self, error: &str) -> crate::diagnostics::DiagnosticBundle {
        let url = self.get_url();
        let page_title = self
            .eval_string("document.title")
            .ok()
            .flatten()
            .unwrap_or_default();
        let page_html = self
            .eval_string("document.documentElement.outerHTML")
            .ok()
            .flatten()
            .unwrap_or_default();
        crate::diagnostics::DiagnosticBundle {
            timestamp: crate::diagnostics::now_iso(),
            url,
            page_title,
            page_html,
            error: error.to_string(),
        }
    }

    /// Checks for a DSID cookie in the tab's current cookie jar.
    pub(crate) fn poll_dsid(&self, domain: &str) -> anyhow::Result<Option<String>> {
        let cookies = self
            .0
            .get_cookies()
            .map_err(|e| anyhow::anyhow!("Browser error: {}", e))?;
        if let Some(cookie) = cookies
            .iter()
            .find(|c| c.name == "DSID" && c.domain.contains(domain))
        {
            return Ok(Some(cookie.value.clone()));
        }
        Ok(None)
    }

    // ── Input watcher helpers ─────────────────────────────────────────────

    /// Injects a watcher that sets `window.__kuvpn_input_gone = true`
    /// when the given CSS selector becomes invisible.
    pub(crate) fn inject_input_watcher(&self, selector: &str) {
        let sel = crate::utils::js_escape(selector);
        let js = format!(
            r#"(function(){{
    window.__kuvpn_input_gone = false;
    if (window.__kuvpn_watch_iv) clearInterval(window.__kuvpn_watch_iv);
    window.__kuvpn_watch_iv = setInterval(function() {{
        var el = document.querySelector('{sel}');
        if (!el || el.offsetParent === null) {{
            window.__kuvpn_input_gone = true;
            clearInterval(window.__kuvpn_watch_iv);
        }}
    }}, 50);
}})()"#
        );
        self.0.evaluate(&js, false).ok();
    }

    /// Clears the input-watcher interval set by `inject_input_watcher`.
    pub(crate) fn clear_input_watcher(&self) {
        self.0
            .evaluate(
                "if(window.__kuvpn_watch_iv){clearInterval(window.__kuvpn_watch_iv);}",
                false,
            )
            .ok();
    }

    /// Fills a DOM input element with `value` and dispatches input/change events.
    pub(crate) fn fill_input_value(&self, selector: &str, value: &str) -> anyhow::Result<()> {
        let val = crate::utils::js_escape(value);
        let js = format!(
            r#"
var el = document.querySelector('{selector}');
if (el) {{
    el.focus();
    el.value = '{val}';
    el.dispatchEvent(new Event('input', {{ bubbles: true }}));
    el.dispatchEvent(new Event('change', {{ bubbles: true }}));
}}"#
        );
        self.0.evaluate(&js, false)?;
        Ok(())
    }
}
