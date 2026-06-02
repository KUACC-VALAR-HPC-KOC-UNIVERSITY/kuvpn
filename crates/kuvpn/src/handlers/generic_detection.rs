use super::AuthTab;

impl AuthTab {
    /// Detects any visible error message using common CSS patterns.
    /// Returns the error text if found, or `None` if the page looks clean.
    pub(crate) fn detect_generic_error(&self) -> anyhow::Result<Option<String>> {
        let text = self.eval_string(
            r#"(function() {
            const errorSelectors = [
                '.error-message',
                '.error-text',
                '.alert-error',
                '.alert-danger',
                '.validation-error',
                '.field-validation-error',
                '.ext-error',
                '.error.ext-error',
                'div[id*="error" i][id*="message" i]',
                'div[id*="error" i]:not([style*="display: none"])',
                'span[id*="error" i]:not([style*="display: none"])',
            ];

            const ariaSelectors = [
                '[role="alert"]',
                '[aria-live="assertive"]',
                '[aria-live="polite"]',
            ];

            const errorKeywords = [
                'error', 'failed', 'failure', 'invalid', 'incorrect',
                'denied', 'blocked', 'expired', 'unable', 'problem',
                'went wrong', 'try again', 'couldn\'t', 'could not',
                'not recognized', 'too many', 'locked', 'suspended',
            ];

            const mfaSafe = [
                'enter the code', 'enter code', 'enter the number',
                'enter number', 'verification code', 'verify your',
                'we sent', 'we texted', 'we emailed',
                'approve sign', 'approve the sign', 'approve a sign',
                'open your authenticator', 'check your authenticator',
                'open your microsoft', 'check your microsoft',
                'notification was sent', 'notification to',
                'authenticator app', 'sign in request',
                'waiting for', 'checking', 'contacting',
            ];

            function isVisible(el) {
                return el && el.offsetParent !== null
                    && el.innerText && el.innerText.trim().length > 5
                    && el.innerText.trim().length < 500;
            }

            function isMfaText(lower) {
                return mfaSafe.some(function(p) { return lower.includes(p); });
            }

            for (var s = 0; s < errorSelectors.length; s++) {
                try {
                    var els = document.querySelectorAll(errorSelectors[s]);
                    for (var i = 0; i < els.length; i++) {
                        if (isVisible(els[i])) {
                            var text = els[i].innerText.trim();
                            if (!isMfaText(text.toLowerCase())) return text;
                        }
                    }
                } catch (e) {}
            }

            for (var s = 0; s < ariaSelectors.length; s++) {
                try {
                    var els = document.querySelectorAll(ariaSelectors[s]);
                    for (var i = 0; i < els.length; i++) {
                        if (isVisible(els[i])) {
                            var text = els[i].innerText.trim();
                            var lower = text.toLowerCase();
                            if (isMfaText(lower)) continue;
                            var hasError = errorKeywords.some(function(k) {
                                return lower.includes(k);
                            });
                            if (hasError) return text;
                        }
                    }
                } catch (e) {}
            }

            return null;
        })()"#,
        )?;
        Ok(text.filter(|t| !t.is_empty()))
    }

    /// Detects if we are stuck on an unexpected page by analysing the page title
    /// and meta tags.  Returns a diagnostic string, or `None` if the page looks normal.
    pub(crate) fn detect_unexpected_page_state(&self) -> anyhow::Result<Option<String>> {
        let text = self.eval_string(
            r#"(function() {
            const title = document.title || '';

            const blockKeywords = [
                'error', 'blocked', 'forbidden', 'denied', 'unauthorized',
                'not found', '404', '403', '500', 'unavailable', 'maintenance'
            ];

            const lowerTitle = title.toLowerCase();
            for (const keyword of blockKeywords) {
                if (lowerTitle.includes(keyword)) {
                    return 'Unexpected page state: ' + title;
                }
            }

            const pageId = document.querySelector('meta[name="PageID"]');
            if (pageId) {
                const content = pageId.content || '';
                if (content.toLowerCase().includes('error')) {
                    return 'Error page detected: ' + content;
                }
            }

            const bodyText = document.body ? document.body.innerText : '';
            if (bodyText.length < 100 && bodyText.toLowerCase().includes('error')) {
                return 'Minimal content error page: ' + bodyText.trim().substring(0, 100);
            }

            return null;
        })()"#,
        )?;
        Ok(text.filter(|t| !t.is_empty()))
    }

    /// Logs current page state (title, URL, visible inputs/buttons/headers) for debugging.
    pub(crate) fn log_page_state(&self) -> anyhow::Result<()> {
        if let Some(state) = self.eval_string(
            r#"(function() {
            const info = {
                title: document.title,
                url: window.location.href,
                pageId: document.querySelector('meta[name="PageID"]')?.content || 'unknown',
                visibleInputs: Array.from(document.querySelectorAll(
                    'input[type="text"], input[type="email"], input[type="password"]'
                ))
                    .filter(el => el.offsetParent !== null)
                    .map(el => ({ type: el.type, name: el.name, id: el.id, placeholder: el.placeholder })),
                visibleButtons: Array.from(document.querySelectorAll('button, input[type="submit"]'))
                    .filter(el => el.offsetParent !== null)
                    .map(el => ({ text: el.innerText || el.value || '', id: el.id, type: el.type }))
                    .slice(0, 5),
                headers: Array.from(document.querySelectorAll('h1, h2, .heading, [role="heading"]'))
                    .filter(el => el.offsetParent !== null)
                    .map(el => el.innerText.trim())
                    .filter(text => text.length > 0)
                    .slice(0, 3),
            };
            return JSON.stringify(info, null, 2);
        })()"#,
        )? {
            log::debug!("[Page State] {}", state);
        }
        Ok(())
    }
}
