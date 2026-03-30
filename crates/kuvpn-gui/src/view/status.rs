use crate::app::KuVpnGui;
use crate::types::{
    Message, ICON_CLOCK_SVG, ICON_INFO_SVG, ICON_PHONE_SVG, ICON_REFRESH_SVG,
    ICON_SHIELD_CHECK_SVG, ICON_SHIELD_SVG, ICON_SHIELD_X_SVG,
};
use iced::widget::{button, column, container, row, stack, svg, text, Space};
use iced::{Alignment, Border, Color, Element, Font, Length, Shadow, Vector};
use kuvpn::ConnectionStatus;

impl KuVpnGui {
    /// Hero status area: circle + status text + subtitle
    pub fn view_status_circle(&self) -> Element<'_, Message> {
        let s = self.styler();
        let p = s.p;

        let (color, icon_svg, status_text) = match self.status {
            ConnectionStatus::Disconnected => (p.text_muted, ICON_SHIELD_SVG, "Public Access"),
            ConnectionStatus::Connecting => (p.warning, ICON_REFRESH_SVG, "Joining Campus..."),
            ConnectionStatus::Connected => (p.success, ICON_SHIELD_CHECK_SVG, "KU Network Active"),
            ConnectionStatus::Disconnecting => (p.warning, ICON_REFRESH_SVG, "Disconnecting..."),
            ConnectionStatus::Error => {
                let error_type = match self.error_category {
                    Some(kuvpn::ErrorCategory::Authentication) => "Authentication Error",
                    Some(kuvpn::ErrorCategory::Connection) => "Connection Error",
                    Some(kuvpn::ErrorCategory::System) => "System Error",
                    None => "Error",
                };
                (p.danger, ICON_SHIELD_X_SVG, error_type)
            }
        };

        // Glow derived from the same `color` as the circle border so all active
        // states (connected, connecting, disconnecting, error) stay in sync with
        // the current theme accent / danger colour automatically.
        // Light themes: halved alpha so the glow doesn't look harsh on a pale background.
        let light = p.bg.r > 0.5;
        let glow = match self.status {
            ConnectionStatus::Disconnected => Shadow {
                color: Color::TRANSPARENT,
                offset: Vector::ZERO,
                blur_radius: 0.0,
            },
            ConnectionStatus::Connected => Shadow {
                color: Color::from_rgba(color.r, color.g, color.b, if light { 0.20 } else { 0.40 }),
                offset: Vector::ZERO,
                blur_radius: 25.0,
            },
            ConnectionStatus::Connecting | ConnectionStatus::Disconnecting => Shadow {
                color: Color::from_rgba(color.r, color.g, color.b, if light { 0.18 } else { 0.35 }),
                offset: Vector::ZERO,
                blur_radius: 20.0,
            },
            ConnectionStatus::Error => Shadow {
                color: Color::from_rgba(color.r, color.g, color.b, if light { 0.18 } else { 0.35 }),
                offset: Vector::ZERO,
                blur_radius: 20.0,
            },
        };

        let mut icon_display = svg(svg::Handle::from_memory(icon_svg))
            .width(60)
            .height(60)
            .style(move |_, _| svg::Style { color: Some(color) });

        if self.status == ConnectionStatus::Connecting
            || self.status == ConnectionStatus::Disconnecting
        {
            icon_display = icon_display.rotation(self.rotation);
        }

        let subtitle = match self.status {
            ConnectionStatus::Connected => "Internal Resources Available".to_string(),
            ConnectionStatus::Error => {
                if self.automation_warning.is_some() {
                    "See details below".to_string()
                } else {
                    self.error_message
                        .clone()
                        .unwrap_or_else(|| "Check console for details".to_string())
                }
            }
            ConnectionStatus::Connecting | ConnectionStatus::Disconnecting => {
                self.status_message.clone()
            }
            _ => "Koc University Access Restricted".to_string(),
        };

        // For connecting/disconnecting states the status_message can be arbitrarily
        // long (raw OpenConnect log lines). Truncate to a safe char count, then
        // render with a right-edge gradient fade so the cutoff looks intentional.
        let surface_color = p.surface;
        let text_muted = p.text_muted;
        let subtitle_element: Element<'_, Message> = if matches!(
            self.status,
            ConnectionStatus::Connecting | ConnectionStatus::Disconnecting
        ) {
            use iced::gradient;

            // Unicode-safe truncation with an ellipsis character.
            let msg: String = if subtitle.chars().count() > 58 {
                let truncated: String = subtitle.chars().take(55).collect();
                format!("{}…", truncated)
            } else {
                subtitle
            };

            stack![
                // Text layer — centered, no wrapping, clipped at the container edge.
                container(
                    text(msg)
                        .size(12)
                        .color(text_muted)
                        .wrapping(iced::widget::text::Wrapping::None),
                )
                .width(Length::Fill)
                .center_x(Length::Fill)
                .clip(true),
                // Gradient overlay — transparent for most of the width, then
                // fades into the card surface colour at the right edge.
                container(Space::new())
                    .width(Length::Fill)
                    .height(Length::Fixed(18.0))
                    .style(move |_| container::Style {
                        background: Some(iced::Background::Gradient(iced::Gradient::Linear(
                            gradient::Linear::new(std::f32::consts::FRAC_PI_2)
                                .add_stop(0.0, Color::TRANSPARENT)
                                .add_stop(0.68, Color::TRANSPARENT)
                                .add_stop(1.0, surface_color),
                        ))),
                        ..Default::default()
                    }),
            ]
            .width(Length::Fill)
            .into()
        } else {
            text(subtitle)
                .size(12)
                .color(text_muted)
                .align_x(iced::alignment::Horizontal::Center)
                .wrapping(iced::widget::text::Wrapping::Word)
                .width(Length::Fill)
                .into()
        };

        column![
            container(
                container(icon_display)
                    .width(Length::Fixed(140.0))
                    .height(Length::Fixed(140.0))
                    .center_x(Length::Fixed(140.0))
                    .center_y(Length::Fixed(140.0))
                    .style(move |_| container::Style {
                        border: Border {
                            color,
                            width: 2.5,
                            radius: 70.0.into(),
                        },
                        shadow: glow,
                        ..Default::default()
                    })
            )
            .width(Length::Fill)
            .center_x(Length::Fill),
            text(status_text)
                .size(18)
                .color(color)
                .align_x(iced::alignment::Horizontal::Center)
                .width(Length::Fill),
            subtitle_element,
        ]
        .spacing(8)
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .into()
    }

    /// Connection details as inline pills
    pub fn view_connection_details(&self) -> Element<'_, Message> {
        let s = self.styler();
        let p = s.p;

        if let Some(start_time) = self.connection_start {
            let elapsed = start_time.elapsed();
            let duration_str = kuvpn::format_duration_secs(elapsed.as_secs());

            let pill_style = s.pill(p.success);

            let duration_pill = container(
                row![
                    svg(svg::Handle::from_memory(ICON_CLOCK_SVG))
                        .width(14)
                        .height(14)
                        .style(move |_, _| svg::Style {
                            color: Some(p.success)
                        }),
                    text(duration_str).size(12).color(p.success),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .padding([6, 14])
            .style(pill_style);

            #[allow(unused_mut)]
            let mut details_row = row![duration_pill].spacing(8).align_y(Alignment::Center);

            #[cfg(unix)]
            if let Some(ref iface_name) = self.active_interface {
                let iface_display = iface_name.clone();
                let interface_pill = container(
                    row![
                        svg(svg::Handle::from_memory(crate::types::ICON_GLOBE_SVG))
                            .width(14)
                            .height(14)
                            .style(move |_, _| svg::Style {
                                color: Some(p.success)
                            }),
                        text(iface_display).size(12).color(p.success),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding([6, 14])
                .style(s.pill(p.success));
                details_row = details_row.push(interface_pill);
            }

            container(details_row)
                .width(Length::Fill)
                .center_x(Length::Fill)
                .into()
        } else {
            iced::widget::Space::new().height(0).into()
        }
    }

    /// MFA approval banner with phone icon and prominent code
    pub fn view_mfa_card<'a>(&'a self, code: &'a str) -> Element<'a, Message> {
        let s = self.styler();
        let p = s.p;
        container(
            column![
                row![
                    svg(svg::Handle::from_memory(ICON_PHONE_SVG))
                        .width(22)
                        .height(22)
                        .style(move |_, _| svg::Style {
                            color: Some(p.warning)
                        }),
                    text("Approve Sign-In").size(14).color(p.warning),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                container(
                    row![
                        text("Enter ").size(12).color(p.text_muted),
                        container(text(code).size(20).color(p.warning).font(Font::MONOSPACE))
                            .padding([2, 10])
                            .style(s.code_badge(p.warning)),
                        text(" in Authenticator").size(12).color(p.text_muted),
                    ]
                    .spacing(4)
                    .align_y(Alignment::Center)
                )
                .width(Length::Fill)
                .center_x(Length::Fill),
            ]
            .spacing(10)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([14, 18])
        .style(s.mfa_card())
        .into()
    }

    /// Warning/automation banner
    pub fn view_warning_card<'a>(&'a self, warning: &'a str) -> Element<'a, Message> {
        let s = self.styler();
        let p = s.p;
        let mut inner = column![
            row![
                svg(svg::Handle::from_memory(ICON_INFO_SVG))
                    .width(18)
                    .height(18)
                    .style(move |_, _| svg::Style {
                        color: Some(p.warning)
                    }),
                text("Automation Issue").size(13).color(p.warning),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            text(warning)
                .size(11)
                .color(p.text)
                .wrapping(iced::widget::text::Wrapping::Word),
        ]
        .spacing(6);

        if self.last_diagnostic_path.is_some() {
            inner = inner.push(
                button(text("Open diagnostics folder").size(11).color(p.warning))
                    .on_press(Message::OpenDiagnosticsFolder)
                    .padding([4, 8])
                    .style(move |_, _| iced::widget::button::Style {
                        background: None,
                        border: Border {
                            color: Color::from_rgba(p.warning.r, p.warning.g, p.warning.b, 0.5),
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }),
            );
        }

        container(inner)
            .width(Length::Fill)
            .padding([12, 16])
            .style(s.warning_card())
            .into()
    }
}
