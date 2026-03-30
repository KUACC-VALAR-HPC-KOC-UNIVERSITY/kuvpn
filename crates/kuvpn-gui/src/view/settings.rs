use crate::app::KuVpnGui;
use crate::styles::Styler;
use crate::theme::{PaletteFamily, Rounding, ShadowDepth};
use crate::types::{Message, SegmentPosition, ICON_INFO_SVG, ICON_REFRESH_SVG, ICON_TRASH_SVG};
use iced::widget::{button, container, pick_list, row, scrollable, svg, text, text_input, Column};
use iced::{Alignment, Border, Color, Element, Length, Padding, Shadow, Vector};

// ── Tooltip helper ────────────────────────────────────────────────────────────

/// Renders a small (i) icon that shows `tip` as a tooltip on hover.
fn info_tip<'a>(tip: &'a str, s: Styler) -> Element<'a, Message> {
    use iced::widget::tooltip;

    let p = s.p;
    let icon = svg(svg::Handle::from_memory(ICON_INFO_SVG))
        .width(14)
        .height(14)
        .style(move |_, _| svg::Style {
            color: Some(p.text_muted),
        });

    let tip_body = container(text(tip).size(11).color(p.text))
        .padding([7, 10])
        .max_width(260.0)
        .style(s.tooltip_container());

    tooltip(icon, tip_body, tooltip::Position::Left)
        .gap(6)
        .into()
}

// ── Main settings view ────────────────────────────────────────────────────────

impl KuVpnGui {
    pub fn view_advanced_settings(&self) -> Element<'_, Message> {
        let s = self.styler();
        let p = s.p;
        let is_locked = false;

        let locked_hint = if is_locked {
            container(
                row![
                    svg(svg::Handle::from_memory(ICON_INFO_SVG))
                        .width(14)
                        .height(14)
                        .style(move |_, _| svg::Style {
                            color: Some(p.text_muted)
                        }),
                    text("Settings locked during active session.")
                        .size(11)
                        .color(p.text_muted),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
            .padding(5)
        } else {
            container(iced::widget::Space::new().height(0))
        };

        let section_label = |label: &'static str| -> Element<'_, Message> {
            container(text(label).size(10).color(p.text_muted))
                .padding([8, 0])
                .into()
        };

        let divider = || -> Element<'_, Message> {
            container(iced::widget::Space::new().height(0))
                .width(Length::Fill)
                .height(Length::Fixed(1.0))
                .style(s.divider())
                .into()
        };

        // Inline notification shown when the Test button resolved a different path.
        let oc_path_notif: Element<'_, Message> = if let Some(msg) = &self.oc_path_notification {
            container(
                row![
                    svg(svg::Handle::from_memory(ICON_INFO_SVG))
                        .width(13)
                        .height(13)
                        .style(move |_, _| svg::Style {
                            color: Some(p.success)
                        }),
                    text(msg.as_str()).size(10).color(p.success),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .padding([6, 110])
            .style(move |_| container::Style {
                background: Some(
                    Color::from_rgba(p.success.r, p.success.g, p.success.b, 0.07).into(),
                ),
                border: Border {
                    radius: 6.0.into(),
                    color: Color::from_rgba(p.success.r, p.success.g, p.success.b, 0.25),
                    width: 1.0,
                },
                ..Default::default()
            })
            .into()
        } else {
            iced::widget::Space::new().height(0).into()
        };

        // ── Header row ────────────────────────────────────────────────────────
        let header = row![
            text("Settings").size(14).color(p.text).width(Length::Fill),
            self.view_segmented_control(
                &["Basic", "Advanced"],
                &[0.0, 1.0],
                if self.settings.advanced_mode {
                    1.0
                } else {
                    0.0
                },
                false,
                |val| Message::AdvancedModeToggled(val > 0.5)
            ),
            locked_hint,
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        // ── Build settings column dynamically ─────────────────────────────────
        let adv = self.settings.advanced_mode;
        let mut col: Column<'_, Message> = Column::new().spacing(12);

        col = col.push(header);

        // ── APPEARANCE section ────────────────────────────────────────────────
        col = col.push(section_label("APPEARANCE"));

        // Family dropdown
        {
            let current_family = self.settings.theme.family;
            col = col.push(
                self.view_unified_control(
                    "Family:",
                    pick_list(
                        PaletteFamily::ALL.to_vec(),
                        Some(current_family),
                        Message::ThemeFamilyChanged,
                    )
                    .style(s.pick_list_style())
                    .menu_style(s.pick_list_menu_style())
                    .width(Length::Fill)
                    .into(),
                    "Color palette family for the application theme.",
                ),
            );
        }

        // Tone row
        col = col.push(self.view_unified_control(
            "Tone:",
            self.view_segmented_control(
                &["Dark", "Light"],
                &[1.0, 0.0],
                if self.settings.theme.dark { 1.0 } else { 0.0 },
                false,
                |val| Message::ThemeToneChanged(val > 0.5),
            ),
            "Dark or Light variant of the selected color family.",
        ));

        // Rounding + Shadow rows — advanced only
        if adv {
            let roundings = [
                Rounding::Square,
                Rounding::Sharp,
                Rounding::Rounded,
                Rounding::Smooth,
                Rounding::Pill,
            ];
            let current_rounding = self.settings.theme.rounding.unwrap_or_default();
            let rounding_buttons: Vec<Element<'_, Message>> = roundings
                .iter()
                .enumerate()
                .map(|(idx, &r)| {
                    let is_selected = current_rounding == r;
                    let position = if idx == 0 {
                        SegmentPosition::Left
                    } else if idx == roundings.len() - 1 {
                        SegmentPosition::Right
                    } else {
                        SegmentPosition::Middle
                    };
                    button(text(r.label()).size(11))
                        .padding([9, 12])
                        .height(Length::Fixed(34.0))
                        .on_press(Message::ThemeRoundingChanged(r))
                        .style(s.btn_segment(position, is_selected))
                        .into()
                })
                .collect();
            col = col.push(self.view_unified_control(
                "Rounding:",
                row(rounding_buttons).spacing(-1.0).into(),
                "Corner radius for buttons, cards, and inputs. Square is fully flat; Pill is heavily rounded.",
            ));

            let shadows = [
                ShadowDepth::None,
                ShadowDepth::Subtle,
                ShadowDepth::Medium,
                ShadowDepth::Elevated,
            ];
            let current_shadow = self.settings.theme.shadow.unwrap_or_default();
            let shadow_buttons: Vec<Element<'_, Message>> = shadows
                .iter()
                .enumerate()
                .map(|(idx, &sd)| {
                    let is_selected = current_shadow == sd;
                    let position = if idx == 0 {
                        SegmentPosition::Left
                    } else if idx == shadows.len() - 1 {
                        SegmentPosition::Right
                    } else {
                        SegmentPosition::Middle
                    };
                    button(text(sd.label()).size(11))
                        .padding([9, 12])
                        .height(Length::Fixed(34.0))
                        .on_press(Message::ThemeShadowChanged(sd))
                        .style(s.btn_segment(position, is_selected))
                        .into()
                })
                .collect();
            col = col.push(self.view_unified_control(
                "Shadow:",
                row(shadow_buttons).spacing(-1.0).into(),
                "Drop-shadow depth for cards and buttons. None is flat; Elevated adds a strong floating shadow.",
            ));
        }

        // BROWSER section
        col = col.push(divider());
        col = col.push(section_label("BROWSER"));
        col = col.push(self.view_unified_field(
            "KU Email:",
            "username@ku.edu.tr",
            &self.settings.email,
            "Your KU email address (e.g. jdoe24@ku.edu.tr). Saved locally so you don't re-enter after wiping session. Used in Full Auto and Visual Auto modes to pre-fill the login form.",
            is_locked,
            Message::EmailChanged,
        ));
        col = col.push(self.view_unified_control(
            "Login Mode:",
            self.view_segmented_control(
                &["Full Auto", "Visual Auto", "Manual"],
                &[0.0, 1.0, 2.0],
                self.settings.login_mode_val,
                is_locked,
                Message::LoginModeChanged,
            ),
            "Full Auto: fully hands-free login via browser automation — no interaction needed.\n\nVisual Auto: same automation but shows the browser window, useful for debugging.\n\nManual: the browser opens and you complete the login yourself.",
        ));
        // NETWORK section (advanced only)
        if adv {
            col = col.push(divider());
            col = col.push(section_label("NETWORK"));
            col = col.push(self.view_unified_field(
                "Gateway URL:",
                "https://vpn.example.com",
                &self.settings.url,
                "The HTTPS address of the KU VPN gateway server. Leave as the default (https://vpn.ku.edu.tr) unless IT Support instructs you to use a different server.",
                is_locked,
                Message::UrlChanged,
            ));
            col = col.push(self.view_unified_field(
                "DSID Domain:",
                "vpn.example.com",
                &self.settings.domain,
                "Hostname used to extract the DSID session cookie after login. Must match the domain of your Gateway URL. Only change this if you changed the Gateway URL.",
                is_locked,
                Message::DomainChanged,
            ));

            // OC Path row (has a custom layout with Test button)
            {
                #[cfg(windows)]
                let oc_placeholder = r"openconnect\openconnect.exe";
                #[cfg(not(windows))]
                let oc_placeholder = "openconnect";

                #[cfg(windows)]
                let oc_tip = "Path to openconnect.exe. Defaults to the bundled binary in the app directory. Click Test to verify or auto-detect the path.";
                #[cfg(not(windows))]
                let oc_tip = "Path to the openconnect binary. Leave blank to use 'openconnect' from your system PATH. Click Test to auto-detect the installed binary.";

                let oc_row = row![
                    text("OC Path:").size(11).width(Length::Fixed(100.0)),
                    text_input(oc_placeholder, &self.settings.openconnect_path)
                        .on_input(if is_locked {
                            |_| Message::Tick
                        } else {
                            Message::OpenConnectPathChanged
                        })
                        .padding(10)
                        .width(Length::Fill)
                        .style(s.text_input()),
                    {
                        let test = self.oc_test_result;
                        let rounding = s.rounding;
                        button(
                            text(if test == Some(true) {
                                "✓"
                            } else if test == Some(false) {
                                "✗"
                            } else {
                                "Test"
                            })
                            .size(11),
                        )
                        .padding([8, 12])
                        .on_press(if is_locked {
                            Message::Tick
                        } else {
                            Message::TestOpenConnect
                        })
                        .style(move |_, status: button::Status| match test {
                            Some(true) => button::Style {
                                background: Some(p.success.into()),
                                text_color: Color::WHITE,
                                border: Border {
                                    radius: rounding.small_radius().into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            Some(false) => button::Style {
                                background: Some(p.danger.into()),
                                text_color: Color::WHITE,
                                border: Border {
                                    radius: rounding.small_radius().into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            None => {
                                let base = button::Style {
                                    background: Some(Color::TRANSPARENT.into()),
                                    text_color: p.text,
                                    border: Border {
                                        color: p.border,
                                        width: rounding.border_width(),
                                        radius: rounding.small_radius().into(),
                                    },
                                    ..Default::default()
                                };
                                match status {
                                    button::Status::Hovered => button::Style {
                                        background: Some(p.surface.into()),
                                        ..base
                                    },
                                    _ => base,
                                }
                            }
                        })
                    },
                    info_tip(oc_tip, s),
                ]
                .spacing(10)
                .align_y(Alignment::Center);

                col = col.push(oc_row);
                col = col.push(oc_path_notif);
            }

            col = col.push(self.view_unified_control(
                "Tunnel Mode:",
                self.view_segmented_control(
                    &["Full", "Manual"],
                    &[1.0, 2.0],
                    self.settings.tunnel_mode_val,
                    is_locked,
                    Message::TunnelModeChanged,
                ),
                "Full: all traffic is routed through the VPN tunnel.\n\nManual: supply your own vpnc-script for full control over routing and DNS.",
            ));
            // VPN Script field — visible only in Manual mode
            if self.settings.is_manual_mode() {
                let script_test = self.vpnc_script_test_result;
                let rounding = s.rounding;
                let script_row = row![
                    text("VPN Script:").size(11).width(Length::Fixed(100.0)),
                    text_input(
                        "/usr/share/vpnc-scripts/vpnc-script",
                        &self.settings.vpnc_script,
                    )
                    .on_input(if is_locked {
                        |_| Message::Tick
                    } else {
                        Message::VpncScriptChanged
                    })
                    .padding(10)
                    .width(Length::Fill)
                    .style(s.text_input()),
                    button(
                        text(if script_test == Some(true) {
                            "✓"
                        } else if script_test == Some(false) {
                            "✗"
                        } else {
                            "Test"
                        })
                        .size(11),
                    )
                    .padding([8, 12])
                    .on_press(if is_locked {
                        Message::Tick
                    } else {
                        Message::TestVpncScript
                    })
                    .style(move |_, status: button::Status| match script_test {
                        Some(true) => button::Style {
                            background: Some(p.success.into()),
                            text_color: Color::WHITE,
                            border: Border {
                                radius: rounding.small_radius().into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        Some(false) => button::Style {
                            background: Some(p.danger.into()),
                            text_color: Color::WHITE,
                            border: Border {
                                radius: rounding.small_radius().into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        None => {
                            let base = button::Style {
                                background: Some(Color::TRANSPARENT.into()),
                                text_color: p.text,
                                border: Border {
                                    color: p.border,
                                    width: rounding.border_width(),
                                    radius: rounding.small_radius().into(),
                                },
                                ..Default::default()
                            };
                            match status {
                                button::Status::Hovered => button::Style {
                                    background: Some(p.surface.into()),
                                    ..base
                                },
                                _ => base,
                            }
                        }
                    }),
                    info_tip("Path to a custom vpnc-script passed to openconnect via --script. The script receives VPN configuration as environment variables and is responsible for all routing and DNS setup. Click Test to verify the file exists.", s),
                ]
                .spacing(10)
                .align_y(Alignment::Center);

                col = col.push(script_row);

                // Warning when path is empty or not yet tested
                let script_warn: Element<'_, Message> = if self
                    .settings
                    .vpnc_script
                    .trim()
                    .is_empty()
                {
                    container(
                        row![
                            svg(svg::Handle::from_memory(ICON_INFO_SVG))
                                .width(13)
                                .height(13)
                                .style(move |_, _| svg::Style { color: Some(p.danger) }),
                            text("A vpnc-script path is required in Manual mode. You must enter a path and click Test before connecting.").size(10).color(p.danger),
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center),
                    )
                    .width(Length::Fill)
                    .padding([6, 110])
                    .style(move |_| container::Style {
                        background: Some(Color::from_rgba(p.danger.r, p.danger.g, p.danger.b, 0.07).into()),
                        border: Border {
                            radius: 6.0.into(),
                            color: Color::from_rgba(p.danger.r, p.danger.g, p.danger.b, 0.25),
                            width: 1.0,
                        },
                        ..Default::default()
                    })
                    .into()
                } else if self.vpnc_script_test_result.is_none() {
                    container(
                        row![
                            svg(svg::Handle::from_memory(ICON_INFO_SVG))
                                .width(13)
                                .height(13)
                                .style(move |_, _| svg::Style {
                                    color: Some(p.warning)
                                }),
                            text("Click Test to verify the script path before connecting.")
                                .size(10)
                                .color(p.warning),
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center),
                    )
                    .width(Length::Fill)
                    .padding([6, 110])
                    .style(move |_| container::Style {
                        background: Some(
                            Color::from_rgba(p.warning.r, p.warning.g, p.warning.b, 0.07).into(),
                        ),
                        border: Border {
                            radius: 6.0.into(),
                            color: Color::from_rgba(p.warning.r, p.warning.g, p.warning.b, 0.25),
                            width: 1.0,
                        },
                        ..Default::default()
                    })
                    .into()
                } else {
                    iced::widget::Space::new().height(0).into()
                };

                col = col.push(script_warn);
            }
        }

        // SYSTEM section
        col = col.push(divider());
        col = col.push(section_label("SYSTEM"));

        // Log Level and Elevation are advanced-only
        if adv {
            col = col.push(self.view_unified_control(
                "Log Level:",
                self.view_segmented_control(
                    &["Off", "Error", "Warn", "Info", "Debug", "Trace"],
                    &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0],
                    self.settings.log_level_val,
                    false,
                    Message::LogLevelSliderChanged,
                ),
                "Controls what gets recorded in the Session Log tab. Info is recommended for everyday use. Debug and Trace add detailed technical output for troubleshooting. Off disables all logging.",
            ));

            #[cfg(not(windows))]
            {
                if self.available_escalation_tools.is_empty() {
                    col = col.push(self.view_unified_control(
                        "Elevation:",
                        container(
                            row![
                                svg(svg::Handle::from_memory(ICON_INFO_SVG))
                                    .width(13)
                                    .height(13)
                                    .style(move |_, _| svg::Style {
                                        color: Some(p.warning)
                                    }),
                                text("No privilege tool found! Install sudo or pkexec.")
                                    .size(11)
                                    .color(p.warning),
                            ]
                            .spacing(6)
                            .align_y(Alignment::Center),
                        )
                        .into(),
                        "The privilege escalation tool used to launch OpenConnect as root (required to create the VPN tunnel). Install sudo or pkexec to proceed.",
                    ));
                } else {
                    col = col.push(self.view_unified_control(
                        "Elevation:",
                        self.view_segmented_control_str(
                            &self.available_escalation_tools,
                            &self.settings.escalation_tool,
                            is_locked,
                            Message::EscalationToolChanged,
                        ),
                        "The privilege escalation tool used to launch OpenConnect as root (required to create the VPN network interface). Only tools installed on your system are listed.",
                    ));
                }
            }
        }

        col = col.push(self.view_unified_control(
            "Close to Tray:",
            self.view_segmented_control(
                &["Yes", "No"],
                &[1.0, 0.0],
                if self.settings.close_to_tray { 1.0 } else { 0.0 },
                false,
                |val| Message::CloseToTrayToggled(val > 0.5),
            ),
            "When Yes, clicking the window × button minimizes to the system tray instead of quitting — the VPN stays connected. When No, clicking × exits the app and disconnects.",
        ));
        col = col.push(self.view_unified_control(
            "Auto-hide:",
            self.view_segmented_control(
                &["Yes", "No"],
                &[1.0, 0.0],
                if self.settings.auto_hide_after_prompt { 1.0 } else { 0.0 },
                false,
                |val| Message::AutoHideAfterPromptToggled(val > 0.5),
            ),
            "When Yes, the window automatically hides after a login prompt resolves, if the window was originally shown automatically from the tray to display that prompt.",
        ));
        col = col.push(self.view_unified_control(
            "Window Style:",
            self.view_segmented_control(
                &["System", "Custom"],
                &[0.0, 1.0],
                if self.settings.use_client_decorations { 1.0 } else { 0.0 },
                false,
                |val| Message::ClientDecorationsToggled(val > 0.5),
            ),
            "System: uses your OS's native window borders and titlebar.\n\nCustom: frameless window with a built-in dark titlebar — matches the app theme better.",
        ));

        // ACTIONS section
        col = col.push(divider());
        col = col.push(section_label("ACTIONS"));
        {
            let fade = self.notif_fade;

            let wipe_btn: Element<'_, Message> = if let Some(success) = self.session_wipe_result {
                let (label, fb_color) = if success {
                    ("✓  WIPED", p.success)
                } else {
                    ("✗  FAILED", p.danger)
                };
                let (r, g, b) = (fb_color.r, fb_color.g, fb_color.b);
                button(
                    text(label)
                        .size(11)
                        .color(Color::from_rgba(1.0, 1.0, 1.0, fade)),
                )
                .padding([10, 14])
                .on_press(Message::ClearSessionPressed)
                .style(move |_, _| button::Style {
                    background: Some(Color::from_rgba(r, g, b, 0.55 * fade).into()),
                    border: Border {
                        radius: 10.0.into(),
                        color: Color::from_rgba(r, g, b, 0.75 * fade),
                        width: 1.5,
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(r, g, b, 0.45 * fade),
                        offset: Vector::new(0.0, 0.0),
                        blur_radius: 12.0,
                    },
                    ..Default::default()
                })
                .into()
            } else {
                button(
                    row![
                        svg(svg::Handle::from_memory(ICON_TRASH_SVG))
                            .width(13)
                            .height(13)
                            .style(move |_, _| svg::Style {
                                color: Some(p.text)
                            }),
                        text("WIPE SESSION").size(11).color(p.text),
                    ]
                    .spacing(7)
                    .align_y(Alignment::Center),
                )
                .padding([10, 14])
                .on_press(Message::ClearSessionPressed)
                .style(s.btn_secondary())
                .into()
            };

            let reset_btn: Element<'_, Message> = if self.reset_notification {
                let sc = p.success;
                button(
                    text("✓  RESTORED")
                        .size(11)
                        .color(Color::from_rgba(1.0, 1.0, 1.0, fade)),
                )
                .padding([10, 14])
                .on_press(if is_locked {
                    Message::Tick
                } else {
                    Message::ResetSettings
                })
                .style(move |_, _| button::Style {
                    background: Some(Color::from_rgba(sc.r, sc.g, sc.b, 0.55 * fade).into()),
                    border: Border {
                        radius: 10.0.into(),
                        color: Color::from_rgba(sc.r, sc.g, sc.b, 0.75 * fade),
                        width: 1.5,
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(sc.r, sc.g, sc.b, 0.45 * fade),
                        offset: Vector::new(0.0, 0.0),
                        blur_radius: 12.0,
                    },
                    ..Default::default()
                })
                .into()
            } else {
                button(
                    row![
                        svg(svg::Handle::from_memory(ICON_REFRESH_SVG))
                            .width(13)
                            .height(13)
                            .style(move |_, _| svg::Style {
                                color: Some(p.text)
                            }),
                        text("RESET DEFAULTS").size(11).color(p.text),
                    ]
                    .spacing(7)
                    .align_y(Alignment::Center),
                )
                .padding([10, 14])
                .on_press(if is_locked {
                    Message::Tick
                } else {
                    Message::ResetSettings
                })
                .style(s.btn_secondary())
                .into()
            };

            col = col.push(row([wipe_btn, reset_btn]).spacing(10));
        }

        container(
            scrollable(container(col).padding(Padding {
                top: 0.0,
                right: 24.0,
                bottom: 0.0,
                left: 0.0,
            }))
            .height(Length::Fill)
            .style(s.scrollbar()),
        )
        .padding(24)
        .width(Length::Fill)
        .style(s.card())
        .into()
    }

    // ── Field helpers ─────────────────────────────────────────────────────────

    fn view_unified_field<'a>(
        &self,
        label: &'a str,
        placeholder: &'a str,
        value: &'a str,
        tooltip_text: &'a str,
        locked: bool,
        on_change: fn(String) -> Message,
    ) -> Element<'a, Message> {
        let s = self.styler();
        let p = s.p;
        row![
            text(label)
                .size(11)
                .width(Length::Fixed(100.0))
                .color(p.text),
            text_input(placeholder, value)
                .on_input(if locked { |_| Message::Tick } else { on_change })
                .padding(10)
                .width(Length::Fill)
                .style(s.text_input()),
            info_tip(tooltip_text, s),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .into()
    }

    fn view_unified_control<'a>(
        &self,
        label: &'a str,
        control: Element<'a, Message>,
        tooltip_text: &'a str,
    ) -> Element<'a, Message> {
        let s = self.styler();
        let p = s.p;
        row![
            text(label)
                .size(11)
                .width(Length::Fixed(100.0))
                .color(p.text),
            control,
            info_tip(tooltip_text, s),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .into()
    }

    // ── Segmented controls ────────────────────────────────────────────────────

    fn view_segmented_control<'a>(
        &self,
        options: &'a [&'static str],
        values: &'a [f32],
        current_value: f32,
        locked: bool,
        on_change: fn(f32) -> Message,
    ) -> Element<'a, Message> {
        let s = self.styler();
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
                    .style(s.btn_segment(position, is_selected))
                    .into()
            })
            .collect();

        row(buttons).spacing(-1.0).into()
    }

    #[cfg(not(windows))]
    fn view_segmented_control_str<'a>(
        &self,
        options: &'a [&'static str],
        current_value: &str,
        locked: bool,
        on_change: fn(String) -> Message,
    ) -> Element<'a, Message> {
        let s = self.styler();
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
                    .style(s.btn_segment(position, is_selected))
                    .into()
            })
            .collect();

        row(buttons).spacing(-1.0).into()
    }
}
