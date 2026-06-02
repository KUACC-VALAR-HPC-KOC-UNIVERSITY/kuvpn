use crate::theme::{Palette, Rounding, ShadowDepth};
use crate::types::SegmentPosition;
use iced::widget::{button, container, pick_list, scrollable, text_input};
use iced::{Background, Border, Color, Shadow, Vector};

/// Lightweight render-time handle.  All inner types are `Copy`, so `Styler`
/// is also `Copy` — all style methods can return `'static` closures by moving
/// the captured palette / rounding / shadow values.
#[derive(Debug, Clone, Copy)]
pub struct Styler {
    pub p: Palette,
    pub rounding: Rounding,
    pub shadow: ShadowDepth,
}

impl Styler {
    // ── Container styles ──────────────────────────────────────────────────────

    /// Card — the main background panel used by every tab.
    pub fn card(self) -> impl Fn(&iced::Theme) -> container::Style + 'static {
        let p = self.p;
        let rounding = self.rounding;
        let shadow = self.shadow;
        // Light themes: softer shadow so it doesn't look like a dark smear on a pale bg.
        let shadow_alpha = if p.bg.r > 0.5 { 0.10 } else { 0.45 };
        move |_| container::Style {
            background: Some(p.surface.into()),
            border: Border {
                color: p.border_subtle,
                width: 1.0,
                radius: rounding.radius().into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, shadow_alpha),
                offset: Vector::new(0.0, shadow.offset()),
                blur_radius: shadow.blur(),
            },
            ..Default::default()
        }
    }

    /// Window background (without border — used when decorations are off).
    pub fn window_bg(self) -> impl Fn(&iced::Theme) -> container::Style + 'static {
        let p = self.p;
        move |_| container::Style {
            background: Some(p.bg.into()),
            text_color: Some(p.text),
            ..Default::default()
        }
    }

    /// Window background with a thin border (used in CSD / client-decorated mode).
    pub fn window_bg_bordered(self) -> impl Fn(&iced::Theme) -> container::Style + 'static {
        let p = self.p;
        move |_| container::Style {
            background: Some(p.bg.into()),
            text_color: Some(p.text),
            border: Border {
                color: p.border_subtle,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }

    /// CSD title bar background.
    pub fn title_bar(self) -> impl Fn(&iced::Theme) -> container::Style + 'static {
        let p = self.p;
        move |_| container::Style {
            background: Some(p.surface.into()),
            ..Default::default()
        }
    }

    /// Tooltip / popover container.
    pub fn tooltip_container(self) -> impl Fn(&iced::Theme) -> container::Style + 'static {
        let p = self.p;
        let rounding = self.rounding;
        move |_| container::Style {
            background: Some(p.surface_raised.into()),
            border: Border {
                radius: rounding.small_radius().into(),
                color: p.border,
                width: 1.0,
            },
            ..Default::default()
        }
    }

    /// Top-of-window warning strip (OpenConnect / escalation tool missing).
    pub fn warning_banner(self) -> impl Fn(&iced::Theme) -> container::Style + 'static {
        let p = self.p;
        move |_| container::Style {
            background: Some(p.surface.into()),
            border: Border {
                color: p.warning,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }
    }

    /// Card shown when automation fails.
    pub fn warning_card(self) -> impl Fn(&iced::Theme) -> container::Style + 'static {
        let p = self.p;
        let rounding = self.rounding;
        move |_| container::Style {
            background: Some(Color::from_rgba(p.warning.r, p.warning.g, p.warning.b, 0.06).into()),
            border: Border {
                color: Color::from_rgba(p.warning.r, p.warning.g, p.warning.b, 0.3),
                width: 1.0,
                radius: rounding.radius().into(),
            },
            ..Default::default()
        }
    }

    /// MFA approval banner.
    pub fn mfa_card(self) -> impl Fn(&iced::Theme) -> container::Style + 'static {
        let p = self.p;
        let rounding = self.rounding;
        move |_| container::Style {
            background: Some(Color::from_rgba(p.warning.r, p.warning.g, p.warning.b, 0.06).into()),
            border: Border {
                color: Color::from_rgba(p.warning.r, p.warning.g, p.warning.b, 0.3),
                width: 1.0,
                radius: rounding.radius().into(),
            },
            ..Default::default()
        }
    }

    /// Individual row in the connection-history list.
    pub fn history_row(self) -> impl Fn(&iced::Theme) -> container::Style + 'static {
        let p = self.p;
        let rounding = self.rounding;
        move |_| container::Style {
            background: Some(p.surface_raised.into()),
            border: Border {
                radius: rounding.small_radius().into(),
                color: p.border_subtle,
                width: 1.0,
            },
            ..Default::default()
        }
    }

    /// Full-screen modal backdrop.
    pub fn modal_overlay(self) -> impl Fn(&iced::Theme) -> container::Style + 'static {
        let p = self.p;
        move |_| container::Style {
            background: Some(p.overlay.into()),
            ..Default::default()
        }
    }

    /// Floating modal card.
    pub fn modal_card(self) -> impl Fn(&iced::Theme) -> container::Style + 'static {
        let p = self.p;
        let rounding = self.rounding;
        move |_| container::Style {
            background: Some(p.surface.into()),
            border: Border {
                radius: rounding.radius().into(),
                color: p.accent,
                width: 1.0,
            },
            ..Default::default()
        }
    }

    /// Rounded pill with a tinted fill — used for connection-details badges.
    pub fn pill(self, color: Color) -> impl Fn(&iced::Theme) -> container::Style + 'static {
        move |_| container::Style {
            background: Some(Color::from_rgba(color.r, color.g, color.b, 0.08).into()),
            border: Border {
                radius: 16.0.into(),
                color: Color::from_rgba(color.r, color.g, color.b, 0.2),
                width: 1.0,
            },
            ..Default::default()
        }
    }

    /// Horizontal divider line.
    pub fn divider(self) -> impl Fn(&iced::Theme) -> container::Style + 'static {
        let p = self.p;
        move |_| container::Style {
            background: Some(p.border_subtle.into()),
            ..Default::default()
        }
    }

    /// Inline code badge (e.g. MFA code display).
    pub fn code_badge(self, color: Color) -> impl Fn(&iced::Theme) -> container::Style + 'static {
        move |_| container::Style {
            background: Some(Color::from_rgba(color.r, color.g, color.b, 0.12).into()),
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    // ── Scrollbar ─────────────────────────────────────────────────────────────

    pub fn scrollbar(
        self,
    ) -> impl Fn(&iced::Theme, scrollable::Status) -> scrollable::Style + 'static {
        let p = self.p;
        move |_, status| {
            let scroller_color = if matches!(status, scrollable::Status::Hovered { .. }) {
                p.border
            } else {
                p.border_subtle
            };

            let rail = scrollable::Rail {
                background: Some(p.bg.into()),
                border: Border {
                    radius: 4.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                scroller: scrollable::Scroller {
                    background: scroller_color.into(),
                    border: Border {
                        radius: 4.0.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                },
            };

            scrollable::Style {
                container: container::Style::default(),
                vertical_rail: rail,
                horizontal_rail: rail,
                gap: None,
                auto_scroll: scrollable::AutoScroll {
                    background: Background::Color(Color::TRANSPARENT),
                    border: Border::default(),
                    shadow: Shadow::default(),
                    icon: Color::TRANSPARENT,
                },
            }
        }
    }

    // ── Button styles ─────────────────────────────────────────────────────────

    /// Filled accent-coloured primary button.
    pub fn btn_primary(self) -> impl Fn(&iced::Theme, button::Status) -> button::Style + 'static {
        let p = self.p;
        let rounding = self.rounding;
        let shadow = self.shadow;
        move |_, status| {
            let base = button::Style {
                background: Some(p.accent.into()),
                text_color: Color::WHITE,
                border: Border {
                    radius: rounding.radius().into(),
                    ..Default::default()
                },
                shadow: Shadow {
                    color: Color::from_rgba(p.accent.r, p.accent.g, p.accent.b, 0.3),
                    offset: Vector::new(0.0, shadow.offset()),
                    blur_radius: shadow.blur(),
                },
                ..Default::default()
            };
            match status {
                button::Status::Hovered => button::Style {
                    background: Some(p.accent_hover.into()),
                    shadow: Shadow {
                        color: Color::from_rgba(p.accent.r, p.accent.g, p.accent.b, 0.5),
                        offset: Vector::new(0.0, shadow.offset() * 2.0),
                        blur_radius: shadow.blur() * 1.5,
                    },
                    ..base
                },
                button::Status::Pressed => button::Style {
                    background: Some(
                        Color::from_rgba(
                            (p.accent.r * 0.75).min(1.0),
                            (p.accent.g * 0.75).min(1.0),
                            (p.accent.b * 0.75).min(1.0),
                            1.0,
                        )
                        .into(),
                    ),
                    shadow: Shadow {
                        color: Color::from_rgba(p.accent.r, p.accent.g, p.accent.b, 0.2),
                        offset: Vector::new(0.0, 1.0),
                        blur_radius: 4.0,
                    },
                    ..base
                },
                _ => base,
            }
        }
    }

    /// Ghost / outlined secondary button.
    pub fn btn_secondary(self) -> impl Fn(&iced::Theme, button::Status) -> button::Style + 'static {
        let p = self.p;
        let rounding = self.rounding;
        let s_base = if p.bg.r > 0.5 { 0.06 } else { 0.20 };
        let s_hover = if p.bg.r > 0.5 { 0.10 } else { 0.30 };
        let s_press = if p.bg.r > 0.5 { 0.04 } else { 0.15 };
        move |_, status| {
            let base = button::Style {
                background: Some(Color::TRANSPARENT.into()),
                text_color: p.text,
                border: Border {
                    color: p.border,
                    width: rounding.border_width(),
                    radius: rounding.radius().into(),
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, s_base),
                    offset: Vector::new(0.0, 1.0),
                    blur_radius: 4.0,
                },
                ..Default::default()
            };
            match status {
                button::Status::Hovered => button::Style {
                    background: Some(p.surface.into()),
                    border: Border {
                        color: p.border,
                        width: rounding.border_width(),
                        radius: rounding.radius().into(),
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, s_hover),
                        offset: Vector::new(0.0, 2.0),
                        blur_radius: 8.0,
                    },
                    ..base
                },
                button::Status::Pressed => button::Style {
                    background: Some(p.surface_raised.into()),
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, s_press),
                        offset: Vector::new(0.0, 1.0),
                        blur_radius: 2.0,
                    },
                    ..base
                },
                _ => base,
            }
        }
    }

    /// Segmented-control button. Handles both selected and unselected states in
    /// one method, combining the old `btn_segment_selected` / `btn_segment_unselected`.
    pub fn btn_segment(
        self,
        position: SegmentPosition,
        selected: bool,
    ) -> impl Fn(&iced::Theme, button::Status) -> button::Style + 'static {
        let p = self.p;
        let r = self.rounding.small_radius(); // outer corners follow the rounding
        move |_, status| {
            let radius = match position {
                SegmentPosition::Left => iced::border::Radius {
                    top_left: r,
                    top_right: 0.0,
                    bottom_right: 0.0,
                    bottom_left: r,
                },
                SegmentPosition::Middle => iced::border::Radius {
                    top_left: 0.0,
                    top_right: 0.0,
                    bottom_right: 0.0,
                    bottom_left: 0.0,
                },
                SegmentPosition::Right => iced::border::Radius {
                    top_left: 0.0,
                    top_right: r,
                    bottom_right: r,
                    bottom_left: 0.0,
                },
                SegmentPosition::Single => iced::border::Radius {
                    top_left: r,
                    top_right: r,
                    bottom_right: r,
                    bottom_left: r,
                },
            };

            if selected {
                button::Style {
                    background: Some(p.accent.into()),
                    text_color: Color::WHITE,
                    border: Border {
                        radius,
                        ..Default::default()
                    },
                    shadow: Shadow::default(),
                    ..Default::default()
                }
            } else {
                let base = button::Style {
                    background: Some(Color::TRANSPARENT.into()),
                    text_color: p.text,
                    border: Border {
                        color: p.border,
                        width: 1.0,
                        radius,
                    },
                    shadow: Shadow::default(),
                    ..Default::default()
                };
                match status {
                    button::Status::Hovered => button::Style {
                        background: Some(p.surface_raised.into()),
                        border: Border {
                            color: p.border,
                            width: 1.0,
                            radius,
                        },
                        ..base
                    },
                    button::Status::Pressed => button::Style {
                        background: Some(p.surface.into()),
                        ..base
                    },
                    _ => base,
                }
            }
        }
    }

    // ── Text input ────────────────────────────────────────────────────────────

    pub fn text_input(
        self,
    ) -> impl Fn(&iced::Theme, text_input::Status) -> text_input::Style + 'static {
        let p = self.p;
        let rounding = self.rounding;
        move |theme, status| {
            let mut style = text_input::default(theme, status);
            style.background = Background::Color(p.surface_raised);
            style.border = Border {
                color: match status {
                    text_input::Status::Active => p.border_subtle,
                    text_input::Status::Focused { .. } => p.border,
                    text_input::Status::Hovered => p.border,
                    text_input::Status::Disabled => p.border_subtle,
                },
                width: rounding.border_width(),
                radius: rounding.small_radius().into(),
            };
            style
        }
    }

    // ── Pick list (dropdown) ──────────────────────────────────────────────────

    pub fn pick_list_style(
        self,
    ) -> impl Fn(&iced::Theme, pick_list::Status) -> pick_list::Style + 'static {
        let p = self.p;
        let rounding = self.rounding;
        move |_, status| pick_list::Style {
            text_color: p.text,
            placeholder_color: p.text_muted,
            handle_color: p.text_muted,
            background: Background::Color(p.surface_raised),
            border: Border {
                color: match status {
                    pick_list::Status::Active => p.border_subtle,
                    pick_list::Status::Hovered | pick_list::Status::Opened { .. } => p.accent,
                },
                width: rounding.border_width(),
                radius: rounding.small_radius().into(),
            },
        }
    }

    pub fn pick_list_menu_style(
        self,
    ) -> impl Fn(&iced::Theme) -> iced::overlay::menu::Style + 'static {
        let p = self.p;
        let rounding = self.rounding;
        move |_| iced::overlay::menu::Style {
            text_color: p.text,
            background: Background::Color(p.surface),
            border: Border {
                color: p.border,
                width: 1.0,
                radius: rounding.small_radius().into(),
            },
            selected_text_color: Color::WHITE,
            selected_background: Background::Color(p.accent),
            shadow: Shadow::default(),
        }
    }

    // ── CSD title-bar buttons ─────────────────────────────────────────────────

    pub fn minimize_btn(self) -> impl Fn(&iced::Theme, button::Status) -> button::Style + 'static {
        let p = self.p;
        move |_, status| {
            let hover_bg = Color::from_rgba(p.text.r, p.text.g, p.text.b, 0.10);
            button::Style {
                background: Some(Color::TRANSPARENT.into()),
                text_color: p.text,
                border: Border::default(),
                shadow: Shadow::default(),
                ..match status {
                    button::Status::Hovered => button::Style {
                        background: Some(hover_bg.into()),
                        ..Default::default()
                    },
                    _ => Default::default(),
                }
            }
        }
    }

    pub fn close_btn(self) -> impl Fn(&iced::Theme, button::Status) -> button::Style + 'static {
        let p = self.p;
        move |_, status| button::Style {
            background: Some(Color::TRANSPARENT.into()),
            text_color: p.text,
            border: Border::default(),
            shadow: Shadow::default(),
            ..match status {
                button::Status::Hovered => button::Style {
                    background: Some(
                        Color::from_rgba(p.danger.r, p.danger.g, p.danger.b, 0.85).into(),
                    ),
                    text_color: Color::WHITE,
                    ..Default::default()
                },
                _ => Default::default(),
            }
        }
    }
}
