pub mod actions;
pub mod console;
pub mod modal;
pub mod settings;
pub mod status;

use crate::app::KuVpnGui;
use crate::types::{
    btn_segment_selected, btn_segment_unselected, Message, SegmentPosition, Tab, COLOR_BG,
    COLOR_SUCCESS, COLOR_SURFACE, COLOR_TEXT, COLOR_TEXT_DIM, COLOR_WARNING, ICON_SETTINGS_SVG,
    ICON_SHIELD_SVG, ICON_TERMINAL_SVG,
};
use iced::widget::{button, column, container, mouse_area, row, stack, svg, text, Column, Space};
use iced::{Alignment, Border, Color, Element, Length, Shadow};
use kuvpn::ConnectionStatus;

impl KuVpnGui {
    fn view_title_bar(&self) -> Element<'_, Message> {
        let (dot_color, bar_label) = match self.status {
            ConnectionStatus::Connected => (COLOR_SUCCESS, "Connected"),
            ConnectionStatus::Connecting => (COLOR_WARNING, "Connecting..."),
            ConnectionStatus::Disconnecting => (COLOR_WARNING, "Disconnecting..."),
            ConnectionStatus::Error => (Color::from_rgb(0.8, 0.2, 0.2), "Error"),
            ConnectionStatus::Disconnected => (COLOR_TEXT_DIM, "Ready"),
        };

        let title_bar_content = row![
            svg(svg::Handle::from_memory(crate::types::KU_LOGO_BYTES))
                .width(20)
                .height(20)
                .style(|_, _| svg::Style {
                    color: Some(iced::Color::WHITE)
                }),
            text("KUVPN").size(14).color(iced::Color::WHITE),
            container(Space::new().width(0).height(0))
                .width(6)
                .height(6)
                .style(move |_| container::Style {
                    background: Some(dot_color.into()),
                    border: Border {
                        radius: 3.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            text(bar_label)
                .size(11)
                .color(Color::from_rgb(0.45, 0.45, 0.45)),
            Space::new().width(Length::Fill),
            button(text("−").size(18).color(COLOR_TEXT))
                .padding([0, 12])
                .on_press(Message::MinimizeWindow)
                .style(|_theme, status| button::Style {
                    background: Some(Color::TRANSPARENT.into()),
                    text_color: COLOR_TEXT,
                    border: Border::default(),
                    shadow: Shadow::default(),
                    ..match status {
                        button::Status::Hovered => button::Style {
                            background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.1).into()),
                            ..Default::default()
                        },
                        _ => Default::default(),
                    }
                }),
            button(text("✕").size(14).color(COLOR_TEXT))
                .padding([0, 12])
                .on_press(Message::ToggleVisibility {
                    from_close_request: true,
                })
                .style(|_theme, status| button::Style {
                    background: Some(Color::TRANSPARENT.into()),
                    text_color: COLOR_TEXT,
                    border: Border::default(),
                    shadow: Shadow::default(),
                    ..match status {
                        button::Status::Hovered => button::Style {
                            background: Some(Color::from_rgba(0.8, 0.2, 0.2, 0.8).into()),
                            text_color: Color::WHITE,
                            ..Default::default()
                        },
                        _ => Default::default(),
                    }
                }),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .padding([8, 12]);

        mouse_area(
            container(title_bar_content)
                .width(Length::Fill)
                .style(|_| container::Style {
                    background: Some(COLOR_SURFACE.into()),
                    ..Default::default()
                }),
        )
        .on_press(Message::DragWindow)
        .into()
    }

    pub fn view(&self, _id: iced::window::Id) -> Element<'_, Message> {
        let use_csd = self.settings.use_client_decorations;

        let tab_bar = self.view_tab_bar();

        let tab_content = match self.current_tab {
            Tab::Connection => self.view_connection_tab(),
            Tab::Settings => self.view_settings_tab(),
            Tab::Console => self.view_console_tab(),
        };

        // Build the column without placeholder spacers so that when no banners are
        // visible there is no phantom gap between the title bar and the tab bar.
        let mut col: Column<'_, Message> = Column::new().spacing(12).width(Length::Fill);

        #[cfg(not(windows))]
        if self.oc_test_result == Some(false) {
            col = col.push(self.view_warning_banner(
                "OpenConnect not found! Please install it or set path in Settings.",
            ));
        }

        #[cfg(not(windows))]
        if self.available_escalation_tools.is_empty() {
            col = col.push(self.view_warning_banner(
                "No privilege tool found! Install sudo or pkexec to use the VPN.",
            ));
        }

        col = col.push(tab_bar);
        col = col.push(tab_content);

        let content = container(col)
            .padding(16)
            .width(Length::Fill)
            .max_width(680.0)
            .center_x(Length::Fill);

        let window_content = if use_csd {
            let title_bar = self.view_title_bar();
            container(column![title_bar, content].width(Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_| container::Style {
                    background: Some(COLOR_BG.into()),
                    text_color: Some(COLOR_TEXT),
                    border: Border {
                        color: Color::from_rgb(0.20, 0.20, 0.20),
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                })
        } else {
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_| container::Style {
                    background: Some(COLOR_BG.into()),
                    text_color: Some(COLOR_TEXT),
                    ..Default::default()
                })
        };

        let main_container = container(window_content)
            .width(Length::Fill)
            .height(Length::Fill);

        if let Some(req) = &self.pending_request {
            stack![main_container, self.view_modal(req)].into()
        } else {
            main_container.into()
        }
    }

    /// Renders a compact warning banner with an info icon and a single line of text.
    fn view_warning_banner<'a>(&self, msg: &'a str) -> Element<'a, Message> {
        container(
            row![
                svg(svg::Handle::from_memory(crate::types::ICON_INFO_SVG))
                    .width(14)
                    .height(14)
                    .style(|_, _| svg::Style {
                        color: Some(crate::types::COLOR_WARNING)
                    }),
                text(msg).size(12).color(crate::types::COLOR_WARNING),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding(10)
        .style(|_| container::Style {
            background: Some(crate::types::COLOR_SURFACE.into()),
            border: Border {
                color: crate::types::COLOR_WARNING,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .into()
    }

    fn view_tab_bar(&self) -> Element<'_, Message> {
        let conn_active = self.current_tab == Tab::Connection;
        let conn_btn = button(
            container(
                row![
                    svg(svg::Handle::from_memory(ICON_SHIELD_SVG))
                        .width(15)
                        .height(15)
                        .style(move |_theme: &iced::Theme, _status| svg::Style {
                            color: Some(if conn_active {
                                iced::Color::WHITE
                            } else {
                                COLOR_TEXT_DIM
                            })
                        }),
                    text("Connection").size(13),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .center_x(Length::Fill),
        )
        .padding([10, 0])
        .width(Length::Fill)
        .on_press(Message::TabChanged(Tab::Connection))
        .style(move |theme, status| {
            if conn_active {
                btn_segment_selected(theme, status, SegmentPosition::Left)
            } else {
                btn_segment_unselected(theme, status, SegmentPosition::Left)
            }
        });

        let settings_active = self.current_tab == Tab::Settings;
        let settings_btn = button(
            container(
                row![
                    svg(svg::Handle::from_memory(ICON_SETTINGS_SVG))
                        .width(15)
                        .height(15)
                        .style(move |_theme: &iced::Theme, _status| svg::Style {
                            color: Some(if settings_active {
                                iced::Color::WHITE
                            } else {
                                COLOR_TEXT_DIM
                            })
                        }),
                    text("Settings").size(13),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .center_x(Length::Fill),
        )
        .padding([10, 0])
        .width(Length::Fill)
        .on_press(Message::TabChanged(Tab::Settings))
        .style(move |theme, status| {
            if settings_active {
                btn_segment_selected(theme, status, SegmentPosition::Middle)
            } else {
                btn_segment_unselected(theme, status, SegmentPosition::Middle)
            }
        });

        let console_active = self.current_tab == Tab::Console;
        let console_btn = button(
            container(
                row![
                    svg(svg::Handle::from_memory(ICON_TERMINAL_SVG))
                        .width(15)
                        .height(15)
                        .style(move |_theme: &iced::Theme, _status| svg::Style {
                            color: Some(if console_active {
                                iced::Color::WHITE
                            } else {
                                COLOR_TEXT_DIM
                            })
                        }),
                    text("Console").size(13),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .center_x(Length::Fill),
        )
        .padding([10, 0])
        .width(Length::Fill)
        .on_press(Message::TabChanged(Tab::Console))
        .style(move |theme, status| {
            if console_active {
                btn_segment_selected(theme, status, SegmentPosition::Right)
            } else {
                btn_segment_unselected(theme, status, SegmentPosition::Right)
            }
        });

        row![conn_btn, settings_btn, console_btn].spacing(0).into()
    }

    fn view_connection_tab(&self) -> Element<'_, Message> {
        let status_hero = self.view_status_circle();
        let action = self.view_actions();

        let mut content = column![]
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill);

        // Top spacer — pushes hero to vertical center
        content = content.push(Space::new().height(Length::Fill));

        // Status hero (circle + text + subtitle)
        content = content.push(status_hero);

        // Connection details pills when connected
        if self.status == ConnectionStatus::Connected {
            content = content.push(Space::new().height(14));
            content = content.push(self.view_connection_details());
        }

        // Bottom spacer — pushes banners + button to bottom
        content = content.push(Space::new().height(Length::Fill));

        // MFA banner
        if let Some(code) = &self.mfa_info {
            content = content.push(self.view_mfa_card(code));
            content = content.push(Space::new().height(10));
        }

        // Automation warning
        if let Some(warning) = &self.automation_warning {
            content = content.push(self.view_warning_card(warning));
            content = content.push(Space::new().height(10));
        }

        // Action button
        content = content.push(action);

        container(content)
            .padding([20, 20])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(crate::types::card)
            .into()
    }

    fn view_settings_tab(&self) -> Element<'_, Message> {
        self.view_advanced_settings()
    }

    fn view_console_tab(&self) -> Element<'_, Message> {
        self.view_console()
    }
}
