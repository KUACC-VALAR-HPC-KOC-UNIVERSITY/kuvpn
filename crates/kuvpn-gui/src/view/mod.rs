pub mod actions;
pub mod console;
pub mod history;
pub mod modal;
pub mod settings;
pub mod status;

use crate::app::KuVpnGui;
use crate::styles::Styler;
use crate::types::{
    Message, SegmentPosition, Tab, ICON_CLOCK_SVG, ICON_SETTINGS_SVG, ICON_SHIELD_SVG,
    ICON_TERMINAL_SVG,
};
use iced::widget::{button, column, container, mouse_area, row, stack, svg, text, Column, Space};
use iced::{Alignment, Border, Color, Element, Length};
use kuvpn::ConnectionStatus;

/// Builds a single tab-bar segment button.  Accepts a `Styler` so it can
/// reference palette colours for the icon tint without touching global constants.
fn tab_button(
    icon: &'static [u8],
    label: &'static str,
    tab: Tab,
    position: SegmentPosition,
    active: bool,
    s: Styler,
) -> Element<'static, Message> {
    let icon_color = if active { Color::WHITE } else { s.p.text_muted };
    button(
        container(
            row![
                svg(svg::Handle::from_memory(icon))
                    .width(15)
                    .height(15)
                    .style(move |_theme: &iced::Theme, _status| svg::Style {
                        color: Some(icon_color),
                    }),
                text(label).size(13),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .center_x(Length::Fill),
    )
    .padding([10, 0])
    .width(Length::Fill)
    .on_press(Message::TabChanged(tab))
    .style(s.btn_segment(position, active))
    .into()
}

impl KuVpnGui {
    fn view_title_bar(&self) -> Element<'_, Message> {
        let s = self.styler();
        let p = s.p;

        let (dot_color, bar_label) = match self.status {
            ConnectionStatus::Connected => (p.success, "Connected"),
            ConnectionStatus::Connecting => (p.warning, "Connecting..."),
            ConnectionStatus::Disconnecting => (p.warning, "Disconnecting..."),
            ConnectionStatus::Error => (p.danger, "Error"),
            ConnectionStatus::Disconnected => (p.text_muted, "Ready"),
        };

        let text_color = p.text;
        let mut title_row = row![
            svg(svg::Handle::from_memory(crate::types::KU_LOGO_BYTES))
                .width(20)
                .height(20)
                .style(move |_, _| svg::Style {
                    color: Some(text_color)
                }),
            text("KUVPN").size(14).color(p.text),
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
            text(bar_label).size(11).color(p.text_muted),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        if let Some(code) = &self.mfa_info {
            title_row = title_row.push(text(format!("MFA: {}", code)).size(11).color(p.warning));
        }

        let title_bar_content = title_row
            .push(Space::new().width(Length::Fill))
            .push(
                button(text("−").size(18).color(p.text))
                    .padding([0, 12])
                    .on_press(Message::MinimizeWindow)
                    .style(s.minimize_btn()),
            )
            .push(
                button(text("✕").size(14).color(p.text))
                    .padding([0, 12])
                    .on_press(Message::ToggleVisibility {
                        from_close_request: true,
                    })
                    .style(s.close_btn()),
            )
            .padding([8, 12]);

        mouse_area(
            container(title_bar_content)
                .width(Length::Fill)
                .style(s.title_bar()),
        )
        .on_press(Message::DragWindow)
        .into()
    }

    pub fn view(&self, _id: iced::window::Id) -> Element<'_, Message> {
        let s = self.styler();
        let use_csd = self.settings.use_client_decorations;

        let tab_bar = self.view_tab_bar();

        let tab_content = match self.current_tab {
            Tab::Connection => self.view_connection_tab(),
            Tab::Settings => self.view_advanced_settings(),
            Tab::History => self.view_history(),
            Tab::Console => self.view_console(),
        };

        // Build the column without placeholder spacers so that when no banners are
        // visible there is no phantom gap between the title bar and the tab bar.
        let mut col: Column<'_, Message> = Column::new().spacing(12).width(Length::Fill);

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
                .style(s.window_bg_bordered())
        } else {
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(s.window_bg())
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
        let s = self.styler();
        let p = s.p;
        container(
            row![
                svg(svg::Handle::from_memory(crate::types::ICON_INFO_SVG))
                    .width(14)
                    .height(14)
                    .style(move |_, _| svg::Style {
                        color: Some(p.warning)
                    }),
                text(msg).size(12).color(p.warning),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding(10)
        .style(s.warning_banner())
        .into()
    }

    fn view_tab_bar(&self) -> Element<'_, Message> {
        let s = self.styler();
        row![
            tab_button(
                ICON_SHIELD_SVG,
                "Connection",
                Tab::Connection,
                SegmentPosition::Left,
                self.current_tab == Tab::Connection,
                s,
            ),
            tab_button(
                ICON_SETTINGS_SVG,
                "Settings",
                Tab::Settings,
                SegmentPosition::Middle,
                self.current_tab == Tab::Settings,
                s,
            ),
            tab_button(
                ICON_CLOCK_SVG,
                "History",
                Tab::History,
                SegmentPosition::Middle,
                self.current_tab == Tab::History,
                s,
            ),
            tab_button(
                ICON_TERMINAL_SVG,
                "Console",
                Tab::Console,
                SegmentPosition::Right,
                self.current_tab == Tab::Console,
                s,
            ),
        ]
        .spacing(0)
        .into()
    }

    fn view_connection_tab(&self) -> Element<'_, Message> {
        let s = self.styler();
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
            .style(s.card())
            .into()
    }
}
