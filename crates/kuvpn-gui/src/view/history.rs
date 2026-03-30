use crate::app::KuVpnGui;
use crate::styles::Styler;
use crate::types::{Message, ICON_CLOCK_SVG, ICON_TRASH_SVG};
use iced::widget::{button, column, container, row, scrollable, svg, text, Space};
use iced::{Alignment, Element, Length, Padding};

impl KuVpnGui {
    pub fn view_history(&self) -> Element<'_, Message> {
        let s = self.styler();
        let p = s.p;

        let heading = row![
            text("Connection History")
                .size(14)
                .color(p.text)
                .width(Length::Fill),
            button(
                row![
                    svg(svg::Handle::from_memory(ICON_TRASH_SVG))
                        .width(13)
                        .height(13)
                        .style(move |_, _| svg::Style {
                            color: Some(p.text)
                        }),
                    text("Clear").size(11).color(p.text),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .padding([8, 12])
            .on_press(Message::ClearHistory)
            .style(s.btn_secondary()),
        ]
        .align_y(Alignment::Center);

        let events_col = if self.history.is_empty() {
            column![
                Space::new().height(Length::Fill),
                container(
                    column![
                        svg(svg::Handle::from_memory(ICON_CLOCK_SVG))
                            .width(32)
                            .height(32)
                            .style(move |_, _| svg::Style {
                                color: Some(p.text_muted)
                            }),
                        text("No connection history yet.")
                            .size(13)
                            .color(p.text_muted),
                        text("Events are recorded when you connect or disconnect.")
                            .size(11)
                            .color(p.text_muted),
                    ]
                    .spacing(8)
                    .align_x(Alignment::Center),
                )
                .center_x(Length::Fill),
                Space::new().height(Length::Fill),
            ]
            .align_x(Alignment::Center)
            .width(Length::Fill)
        } else {
            let mut col = column![].spacing(6).width(Length::Fill);
            // Show newest first
            for event in self.history.iter().rev() {
                col = col.push(view_event_row(event, s));
            }
            col
        };

        // Wrap events in a container with right padding so items don't render
        // under the scrollbar rail when it appears.
        let scrollable_content = container(events_col).width(Length::Fill).padding(Padding {
            top: 0.0,
            right: 16.0,
            bottom: 0.0,
            left: 0.0,
        });

        let content = column![
            heading,
            scrollable(scrollable_content)
                .height(Length::Fill)
                .style(s.scrollbar())
        ]
        .spacing(12)
        .width(Length::Fill)
        .height(Length::Fill);

        container(content)
            .padding(24)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(s.card())
            .into()
    }
}

fn view_event_row<'a>(event: &'a kuvpn::ConnectionEvent, s: Styler) -> Element<'a, Message> {
    use kuvpn::EventKind;

    let p = s.p;

    let (dot_color, kind_label) = match event.kind {
        EventKind::Connected => (p.success, "Connected"),
        EventKind::Reconnected => (p.warning, "Reconnected"),
        EventKind::Disconnected => (p.text_muted, "Disconnected"),
        EventKind::Cancelled => (p.text_muted, "Cancelled"),
        EventKind::Error => (p.danger, "Error"),
    };

    let ts = event.format_timestamp();
    let duration_str = event
        .format_duration_display()
        .map(|d| {
            if event.kind == EventKind::Reconnected {
                format!(" · prev: {}", d)
            } else {
                format!(" · {}", d)
            }
        })
        .unwrap_or_default();

    let mut row_content = row![
        container(Space::new().width(8).height(8))
            .width(8)
            .height(8)
            .style(move |_| iced::widget::container::Style {
                background: Some(dot_color.into()),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        // Fixed width so the timestamp column always starts at the same position
        // regardless of which label ("Disconnected" vs "Error" etc.) is shown.
        text(kind_label)
            .size(12)
            .color(dot_color)
            .width(Length::Fixed(88.0)),
        text(format!("{}{}", ts, duration_str))
            .size(11)
            .color(p.text_muted),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    if let Some(msg) = &event.message {
        row_content = row_content.push(
            text(msg.as_str())
                .size(10)
                .color(p.warning)
                .width(Length::Fill),
        );
    }

    container(row_content)
        .padding([6, 10])
        .width(Length::Fill)
        .style(s.history_row())
        .into()
}
