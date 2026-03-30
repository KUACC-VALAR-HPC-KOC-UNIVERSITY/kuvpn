use crate::app::KuVpnGui;
use crate::theme::Palette;
use crate::types::Message;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Color, Element, Font, Length, Padding};

/// Stable ID for the console log scrollable; used to programmatically snap to bottom.
pub static CONSOLE_SCROLL_ID: std::sync::LazyLock<iced::widget::Id> =
    std::sync::LazyLock::new(|| iced::widget::Id::new("console_logs"));

fn log_line_color(line: &str, p: Palette) -> Color {
    if line.starts_with("[ERR]") || line.starts_with("[!]") {
        p.danger
    } else if line.starts_with("[WRN]") {
        p.warning
    } else if line.starts_with("[INF]") || line.starts_with("[*]") {
        p.text_muted
    } else if line.starts_with("[DBG]") || line.starts_with("[TRC]") {
        p.text_disabled
    } else {
        p.text_muted
    }
}

/// Splits `"[XXX] message text"` into `("[XXX]", "message text")`.
/// Lines without a bracket prefix are returned as `("", line)`.
fn split_log_prefix(line: &str) -> (&str, &str) {
    if line.starts_with('[') {
        if let Some(pos) = line.find("] ") {
            return (&line[..pos + 1], &line[pos + 2..]);
        }
    }
    ("", line)
}

impl KuVpnGui {
    pub fn view_console(&self) -> Element<'_, Message> {
        let s = self.styler();
        let p = s.p;

        let log_lines: Vec<Element<'_, Message>> = self
            .logs
            .iter()
            .map(|line| {
                let color = log_line_color(line, p);
                let (prefix, message) = split_log_prefix(line);
                row![
                    text(prefix).font(Font::MONOSPACE).size(12).color(color),
                    text(message)
                        .font(Font::MONOSPACE)
                        .size(12)
                        .color(color)
                        .width(Length::Fill),
                ]
                .spacing(6)
                .into()
            })
            .collect();

        container(
            column![
                row![
                    text("Session Logs")
                        .size(14)
                        .color(p.text)
                        .width(Length::Fill),
                    button(text("Copy All").size(11).color(p.text))
                        .padding([8, 14])
                        .on_press(Message::CopyLogs)
                        .style(s.btn_secondary()),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                scrollable(
                    container(column(log_lines).spacing(2))
                        .width(Length::Fill)
                        .padding(Padding {
                            top: 0.0,
                            right: 24.0,
                            bottom: 16.0,
                            left: 0.0,
                        }),
                )
                .height(Length::Fill)
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::default(),
                ))
                .on_scroll(|vp| Message::ConsoleScrolled(vp.relative_offset()))
                .id(CONSOLE_SCROLL_ID.clone())
                .style(s.scrollbar()),
            ]
            .spacing(12),
        )
        .padding(24)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(s.card())
        .into()
    }
}
