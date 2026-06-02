use console::{Key, Style, Term};
use dialoguer::Input;
use indicatif::ProgressBar;
use kuvpn::utils::CredentialsProvider;
use std::sync::Arc;

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
pub(crate) struct CliCredentialsProvider {
    pub(crate) spinner: Arc<ProgressBar>,
}

impl CredentialsProvider for CliCredentialsProvider {
    fn request_text(&self, msg: &str) -> Option<String> {
        self.spinner.finish_and_clear();
        let result = Input::new()
            .with_prompt(msg.trim_end_matches(": ").trim_end_matches(':'))
            .interact_text()
            .unwrap_or_default();
        // Clean up the prompt line after input
        let term = Term::stderr();
        let _ = term.clear_last_lines(1);
        Some(result)
    }

    fn request_password(&self, msg: &str) -> Option<String> {
        self.spinner.finish_and_clear();
        let result = read_masked_password(msg.trim_end_matches(": ").trim_end_matches(':'));
        // Clean up the prompt line after input
        let term = Term::stderr();
        let _ = term.clear_line();
        let _ = term.write_str("\r");
        Some(result)
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
