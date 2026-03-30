use crate::app::KuVpnGui;
use crate::types::{InputRequest, Message, ICON_EYE_OFF_SVG, ICON_EYE_SVG, ICON_LOCK_SVG};
use iced::widget::{button, column, container, stack, svg, text, text_input};
use iced::{Alignment, Border, Color, Element, Length, Padding, Shadow};

impl KuVpnGui {
    pub fn view_modal<'a>(&self, req: &'a InputRequest) -> Element<'a, Message> {
        let s = self.styler();
        let p = s.p;

        // Determine context-aware title
        let title = if req.is_password {
            if req.msg.contains("password to start the VPN tunnel") {
                "System Password"
            } else {
                "KU Account"
            }
        } else {
            "KU Authentication"
        };

        let show = self.show_password_held;

        let input_field: Element<'_, Message> = if req.is_password {
            let icon_bytes = if show { ICON_EYE_SVG } else { ICON_EYE_OFF_SVG };
            let icon_color = if show { p.text } else { p.text_muted };

            let eye_btn = button(
                svg(svg::Handle::from_memory(icon_bytes))
                    .width(18)
                    .height(18)
                    .style(move |_, _| svg::Style {
                        color: Some(icon_color),
                    }),
            )
            .padding(Padding {
                top: 0.0,
                right: 12.0,
                bottom: 0.0,
                left: 0.0,
            })
            .on_press(Message::ShowPasswordHeld(!show))
            .style(|_, _| button::Style {
                background: Some(Color::TRANSPARENT.into()),
                border: Border::default(),
                shadow: Shadow::default(),
                ..Default::default()
            });

            stack![
                text_input("Password", &self.current_input)
                    .on_input(Message::InputChanged)
                    .secure(!show)
                    .on_submit(Message::SubmitInput)
                    .padding(Padding {
                        top: 12.0,
                        right: 44.0,
                        bottom: 12.0,
                        left: 12.0,
                    })
                    .width(Length::Fill),
                container(eye_btn)
                    .align_x(Alignment::End)
                    .align_y(Alignment::Center)
                    .width(Length::Fill)
                    .height(Length::Fill),
            ]
            .width(Length::Fill)
            .into()
        } else {
            text_input("username@ku.edu.tr", &self.current_input)
                .on_input(Message::InputChanged)
                .on_submit(Message::SubmitInput)
                .padding(12)
                .into()
        };

        let modal_content = container(
            column![
                iced::widget::row![
                    svg(svg::Handle::from_memory(ICON_LOCK_SVG))
                        .width(26)
                        .height(26)
                        .style(move |_, _| svg::Style {
                            color: Some(p.accent)
                        }),
                    text(title).size(20),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
                text(&req.msg).size(14).color(p.text),
                input_field,
                button(text("VERIFY").size(14).color(Color::WHITE))
                    .width(Length::Fill)
                    .padding([12, 16])
                    .on_press(Message::SubmitInput)
                    .style(s.btn_primary())
            ]
            .spacing(20)
            .padding(32),
        )
        .width(Length::Fixed(400.0))
        .style(s.modal_card());

        container(modal_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(s.modal_overlay())
            .into()
    }
}
