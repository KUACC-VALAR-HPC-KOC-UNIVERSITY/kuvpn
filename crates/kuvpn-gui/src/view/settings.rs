use crate::app::KuVpnGui;
use crate::types::{
    btn_secondary, btn_segment_selected, btn_segment_unselected, card, custom_scrollbar, Message,
    SegmentPosition, COLOR_TEXT, COLOR_TEXT_DIM, ICON_INFO_SVG, ICON_REFRESH_SVG, ICON_TRASH_SVG,
};
#[cfg(not(windows))]
use crate::types::{COLOR_SUCCESS, COLOR_WARNING};
use iced::widget::{button, column, container, row, scrollable, svg, text, text_input};
use iced::{Alignment, Border, Color, Element, Length, Padding};

impl KuVpnGui {
    pub fn view_advanced_settings(&self) -> Element<'_, Message> {
        let is_locked = false; // Allow editing settings while connected

        let locked_hint = if is_locked {
            container(
                row![
                    svg(svg::Handle::from_memory(ICON_INFO_SVG))
                        .width(14)
                        .height(14)
                        .style(|_, _| svg::Style {
                            color: Some(COLOR_TEXT_DIM)
                        }),
                    text("Settings locked during active session.")
                        .size(11)
                        .color(COLOR_TEXT_DIM),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
            .padding(5)
        } else {
            container(iced::widget::Space::new().height(0))
        };

        let section_label = |label: &'static str| -> Element<'_, Message> {
            container(
                text(label)
                    .size(10)
                    .color(Color::from_rgb(0.45, 0.45, 0.45)),
            )
            .padding([8, 0])
            .into()
        };

        let divider = || -> Element<'_, Message> {
            container(iced::widget::Space::new().height(0))
                .width(Length::Fill)
                .height(Length::Fixed(1.0))
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.20, 0.20, 0.20).into()),
                    ..Default::default()
                })
                .into()
        };

        // Inline notification shown when the Test button replaced the entered path.
        #[cfg(not(windows))]
        let oc_path_notif: Element<'_, Message> = if let Some(msg) = &self.oc_path_notification {
            container(
                row![
                    svg(svg::Handle::from_memory(ICON_INFO_SVG))
                        .width(13)
                        .height(13)
                        .style(|_, _| svg::Style {
                            color: Some(COLOR_SUCCESS)
                        }),
                    text(msg.as_str()).size(10).color(COLOR_SUCCESS),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .padding([6, 110])
            .style(|_| container::Style {
                background: Some(Color::from_rgba(0.42, 0.55, 0.35, 0.07).into()),
                border: Border {
                    radius: 6.0.into(),
                    color: Color::from_rgba(0.42, 0.55, 0.35, 0.25),
                    width: 1.0,
                },
                ..Default::default()
            })
            .into()
        } else {
            iced::widget::Space::new().height(0).into()
        };
        #[cfg(windows)]
        let oc_path_notif: Element<'_, Message> = iced::widget::Space::new().height(0).into();

        let settings_content = column![
                // Header
                row![
                    text("Configuration")
                        .size(16)
                        .color(COLOR_TEXT)
                        .width(Length::Fill),
                    locked_hint,
                ]
                .align_y(Alignment::Center),
                // --- Browser Section ---
                section_label("BROWSER"),
                self.view_unified_field(
                    "Login Email:",
                    "email@ku.edu.tr",
                    &self.settings.email,
                    "Optional. Saves your email so you don't re-enter after wiping session. Works in Full Auto and Visual Auto modes",
                    is_locked,
                    Message::EmailChanged
                ),
                self.view_unified_control(
                    "Login Mode:",
                    self.view_segmented_control(
                        &["Full Auto", "Visual Auto", "Manual"],
                        &[0.0, 1.0, 2.0],
                        self.settings.login_mode_val,
                        is_locked,
                        Message::LoginModeChanged
                    ),
                    "Full Auto: headless automation | Visual Auto: visible browser for debugging | Manual: you handle login"
                ),
                divider(),
                // --- Network Section ---
                section_label("NETWORK"),
                self.view_unified_field(
                    "Gateway URL:",
                    "https://vpn.example.com",
                    &self.settings.url,
                    "VPN gateway server address. Default: https://vpn.ku.edu.tr",
                    is_locked,
                    Message::UrlChanged
                ),
                self.view_unified_field(
                    "DSID Domain:",
                    "vpn.example.com",
                    &self.settings.domain,
                    "Domain for authentication cookie. Must match Gateway URL domain",
                    is_locked,
                    Message::DomainChanged
                ),
                {
                    #[cfg(not(windows))]
                    {
                        let oc_placeholder = "openconnect";
                        let oc_helper = "Path to OpenConnect executable. Default: 'openconnect' (searches system PATH)";

                        column![
                            row![
                                text("OC Path:").size(11).width(Length::Fixed(100.0)),
                                text_input(oc_placeholder, &self.settings.openconnect_path)
                                    .on_input(if is_locked {
                                        |_| Message::Tick
                                    } else {
                                        Message::OpenConnectPathChanged
                                    })
                                    .padding(10)
                                    .width(Length::Fill)
                                    .style(move |_theme, status| {
                                        let mut style = text_input::default(_theme, status);
                                        style.background =
                                            iced::Background::Color(Color::from_rgb(0.08, 0.08, 0.08));
                                        style.border = Border {
                                            color: match status {
                                                text_input::Status::Active => Color::from_rgb(0.20, 0.20, 0.20),
                                                text_input::Status::Focused { is_hovered } => {
                                                    if is_hovered {
                                                        Color::from_rgb(0.35, 0.35, 0.35)
                                                    } else {
                                                        Color::from_rgb(0.30, 0.30, 0.30)
                                                    }
                                                }
                                                text_input::Status::Hovered => {
                                                    Color::from_rgb(0.25, 0.25, 0.25)
                                                }
                                                text_input::Status::Disabled => {
                                                    Color::from_rgb(0.15, 0.15, 0.15)
                                                }
                                            },
                                            width: 1.0,
                                            radius: 6.0.into(),
                                        };
                                        style
                                    }),
                                button(text(if self.oc_test_result == Some(true) {
                                    "✓"
                                } else if self.oc_test_result == Some(false) {
                                    "✗"
                                } else {
                                    "Test"
                                }).size(11))
                                .padding([8, 12])
                                .on_press(if is_locked {
                                    Message::Tick
                                } else {
                                    Message::TestOpenConnect
                                })
                                .style(if self.oc_test_result == Some(true) {
                                    button::success
                                } else if self.oc_test_result == Some(false) {
                                    button::danger
                                } else {
                                    button::secondary
                                }),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            container(
                                text(oc_helper)
                                    .size(10)
                                    .color(Color::from_rgb(0.50, 0.50, 0.50))
                            )
                            .padding([0, 110])
                        ]
                        .spacing(4)
                    }
                    #[cfg(windows)]
                    {
                        iced::widget::Space::new()
                            .width(Length::Shrink)
                            .height(Length::Shrink)
                    }
                },
                oc_path_notif,
                divider(),
                // --- System Section ---
                section_label("SYSTEM"),
                self.view_unified_control(
                    "Log Level:",
                    self.view_segmented_control(
                        &["Off", "Error", "Warn", "Info", "Debug", "Trace"],
                        &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0],
                        self.settings.log_level_val,
                        false,
                        Message::LogLevelSliderChanged
                    ),
                    "Console verbosity. Info: normal usage | Debug/Trace: troubleshooting | Off: disable logs"
                ),
                {
                    #[cfg(not(windows))]
                    {
                        if self.available_escalation_tools.is_empty() {
                            self.view_unified_control(
                                "Elevation:",
                                container(
                                    row![
                                        svg(svg::Handle::from_memory(ICON_INFO_SVG))
                                            .width(13)
                                            .height(13)
                                            .style(|_, _| svg::Style {
                                                color: Some(COLOR_WARNING)
                                            }),
                                        text("No privilege tool found! Install sudo or pkexec.")
                                            .size(11)
                                            .color(COLOR_WARNING),
                                    ]
                                    .spacing(6)
                                    .align_y(Alignment::Center),
                                )
                                .into(),
                                "Install sudo or pkexec so OpenConnect can create the VPN tunnel as root",
                            )
                        } else {
                            self.view_unified_control(
                                "Elevation:",
                                self.view_segmented_control_str(
                                    &self.available_escalation_tools,
                                    &self.settings.escalation_tool,
                                    is_locked,
                                    Message::EscalationToolChanged,
                                ),
                                "Privilege tool used to run OpenConnect as root. Only installed tools are shown",
                            )
                        }
                    }
                    #[cfg(windows)]
                    {
                        iced::widget::Space::new()
                            .width(Length::Shrink)
                            .height(Length::Shrink)
                    }
                },
                self.view_unified_control(
                    "Close to Tray:",
                    self.view_segmented_control(
                        &["Yes", "No"],
                        &[1.0, 0.0],
                        if self.settings.close_to_tray {
                            1.0
                        } else {
                            0.0
                        },
                        false,
                        |val| Message::CloseToTrayToggled(val > 0.5)
                    ),
                    "Yes: window X button minimizes to tray | No: X button quits app (disconnects VPN)"
                ),
                self.view_unified_control(
                    "Auto-hide:",
                    self.view_segmented_control(
                        &["Yes", "No"],
                        &[1.0, 0.0],
                        if self.settings.auto_hide_after_prompt {
                            1.0
                        } else {
                            0.0
                        },
                        false,
                        |val| Message::AutoHideAfterPromptToggled(val > 0.5)
                    ),
                    "Yes: window hides again after a prompt resolves when it was auto-shown from tray"
                ),
                self.view_unified_control(
                    "Window Style:",
                    self.view_segmented_control(
                        &["System", "Custom"],
                        &[0.0, 1.0],
                        if self.settings.use_client_decorations {
                            1.0
                        } else {
                            0.0
                        },
                        false,
                        |val| Message::ClientDecorationsToggled(val > 0.5)
                    ),
                    "System: native OS titlebar and window borders | Custom: frameless with custom titlebar"
                ),
                divider(),
                // --- Actions Section ---
                section_label("ACTIONS"),
                row![
                    button(
                        row![
                            svg(svg::Handle::from_memory(ICON_TRASH_SVG))
                                .width(13)
                                .height(13)
                                .style(|_, _| svg::Style {
                                    color: Some(COLOR_TEXT)
                                }),
                            text("WIPE SESSION").size(11).color(COLOR_TEXT),
                        ]
                        .spacing(7)
                        .align_y(Alignment::Center)
                    )
                    .padding([10, 14])
                    .on_press(Message::ClearSessionPressed)
                    .style(btn_secondary),
                    button(
                        row![
                            svg(svg::Handle::from_memory(ICON_REFRESH_SVG))
                                .width(13)
                                .height(13)
                                .style(|_, _| svg::Style {
                                    color: Some(COLOR_TEXT)
                                }),
                            text("RESET DEFAULTS").size(11).color(COLOR_TEXT),
                        ]
                        .spacing(7)
                        .align_y(Alignment::Center)
                    )
                    .padding([10, 14])
                    .on_press(if is_locked {
                        Message::Tick
                    } else {
                        Message::ResetSettings
                    })
                    .style(btn_secondary),
                ]
                .spacing(10)
            ]
            .spacing(12);

        container(
            scrollable(container(settings_content).padding(Padding {
                top: 0.0,
                right: 24.0,
                bottom: 0.0,
                left: 0.0,
            }))
            .height(Length::Fill)
            .style(custom_scrollbar),
        )
        .padding(24)
        .width(Length::Fill)
        .style(card)
        .into()
    }

    fn view_unified_field<'a>(
        &self,
        label: &'a str,
        placeholder: &'a str,
        value: &'a str,
        helper_text: &'a str,
        locked: bool,
        on_change: fn(String) -> Message,
    ) -> Element<'a, Message> {
        column![
            row![
                text(label).size(11).width(Length::Fixed(100.0)),
                text_input(placeholder, value)
                    .on_input(if locked { |_| Message::Tick } else { on_change })
                    .padding(10)
                    .width(Length::Fill)
                    .style(move |_theme, status| {
                        let mut style = text_input::default(_theme, status);
                        style.background =
                            iced::Background::Color(Color::from_rgb(0.08, 0.08, 0.08));
                        style.border = Border {
                            color: match status {
                                text_input::Status::Active => Color::from_rgb(0.20, 0.20, 0.20),
                                text_input::Status::Focused { is_hovered } => {
                                    if is_hovered {
                                        Color::from_rgb(0.35, 0.35, 0.35)
                                    } else {
                                        Color::from_rgb(0.30, 0.30, 0.30)
                                    }
                                }
                                text_input::Status::Hovered => Color::from_rgb(0.25, 0.25, 0.25),
                                text_input::Status::Disabled => Color::from_rgb(0.15, 0.15, 0.15),
                            },
                            width: 1.0,
                            radius: 6.0.into(),
                        };
                        style
                    }),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            container(
                text(helper_text)
                    .size(10)
                    .color(Color::from_rgb(0.50, 0.50, 0.50))
            )
            .padding([0, 110])
        ]
        .spacing(4)
        .into()
    }

    fn view_unified_control<'a>(
        &self,
        label: &'a str,
        control: Element<'a, Message>,
        helper_text: &'a str,
    ) -> Element<'a, Message> {
        column![
            row![text(label).size(11).width(Length::Fixed(100.0)), control,]
                .spacing(10)
                .align_y(Alignment::Center),
            container(
                text(helper_text)
                    .size(10)
                    .color(Color::from_rgb(0.50, 0.50, 0.50))
            )
            .padding([0, 110])
        ]
        .spacing(4)
        .into()
    }

    fn view_segmented_control<'a>(
        &self,
        options: &'a [&'static str],
        values: &'a [f32],
        current_value: f32,
        locked: bool,
        on_change: fn(f32) -> Message,
    ) -> Element<'a, Message> {
        let buttons: Vec<Element<'a, Message>> = options
            .iter()
            .zip(values.iter())
            .enumerate()
            .map(|(idx, (label, &value))| {
                let is_selected = (current_value - value).abs() < 0.1;

                let position = if options.len() == 1 {
                    SegmentPosition::Single
                } else if idx == 0 {
                    SegmentPosition::Left
                } else if idx == options.len() - 1 {
                    SegmentPosition::Right
                } else {
                    SegmentPosition::Middle
                };

                button(text(*label).size(11))
                    .padding([9, 12])
                    .height(Length::Fixed(34.0))
                    .on_press(if locked {
                        Message::Tick
                    } else {
                        on_change(value)
                    })
                    .style(move |theme, status| {
                        if is_selected {
                            btn_segment_selected(theme, status, position)
                        } else {
                            btn_segment_unselected(theme, status, position)
                        }
                    })
                    .into()
            })
            .collect();

        row(buttons).spacing(-1.0).into()
    }

    fn view_segmented_control_str<'a>(
        &self,
        options: &'a [&'static str],
        current_value: &str,
        locked: bool,
        on_change: fn(String) -> Message,
    ) -> Element<'a, Message> {
        let buttons: Vec<Element<'a, Message>> = options
            .iter()
            .enumerate()
            .map(|(idx, &label)| {
                let is_selected = current_value == label;

                let position = if options.len() == 1 {
                    SegmentPosition::Single
                } else if idx == 0 {
                    SegmentPosition::Left
                } else if idx == options.len() - 1 {
                    SegmentPosition::Right
                } else {
                    SegmentPosition::Middle
                };

                button(text(label).size(11))
                    .padding([9, 12])
                    .height(Length::Fixed(34.0))
                    .on_press(if locked {
                        Message::Tick
                    } else {
                        on_change(label.to_string())
                    })
                    .style(move |theme, status| {
                        if is_selected {
                            btn_segment_selected(theme, status, position)
                        } else {
                            btn_segment_unselected(theme, status, position)
                        }
                    })
                    .into()
            })
            .collect();

        row(buttons).spacing(-1.0).into()
    }
}
