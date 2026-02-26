use iced::widget::button;
use iced::widget::container;
use iced::widget::scrollable;
use iced::{Border, Color, Shadow, Vector};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use tray_icon::{menu::MenuEvent, TrayIconEvent};

// --- Constants & Styling ---
pub const KU_LOGO_BYTES: &[u8] = include_bytes!("../assets/ku.svg");
pub const WINDOW_ICON: &[u8] = include_bytes!("../assets/icon-512.png");

// Tray Icons
pub const TRAY_ICON_NORMAL: &[u8] = include_bytes!("../assets/vpn-normal.svg");
pub const TRAY_ICON_CONNECTED: &[u8] = include_bytes!("../assets/vpn-connected.svg");
pub const TRAY_ICON_DISCONNECTED: &[u8] = include_bytes!("../assets/vpn-disconnected.svg");

// Colors (Refined Koç University Palette)
pub const COLOR_BG: Color = Color::from_rgb(0.07, 0.07, 0.07);
pub const COLOR_SURFACE: Color = Color::from_rgb(0.12, 0.12, 0.12);
pub const COLOR_ACCENT: Color = Color::from_rgb(0.50, 0.0, 0.125); // #800020 Burgundy
pub const COLOR_SUCCESS: Color = Color::from_rgb(0.42, 0.55, 0.35);
pub const COLOR_WARNING: Color = Color::from_rgb(0.80, 0.60, 0.30);
pub const COLOR_TEXT: Color = Color::from_rgb(0.85, 0.85, 0.85);
pub const COLOR_TEXT_DIM: Color = Color::from_rgb(0.50, 0.50, 0.50);

// Icons (SVG Paths - using simple geometries)
pub const ICON_SETTINGS_SVG: &[u8] = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>"#.as_bytes();
pub const ICON_SHIELD_SVG: &[u8] = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path></svg>"#.as_bytes();
pub const ICON_SHIELD_CHECK_SVG: &[u8] = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path><path d="M9 12l2 2 4-4"></path></svg>"#.as_bytes();
pub const ICON_LOCK_SVG: &[u8] = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg>"#.as_bytes();
pub const ICON_PHONE_SVG: &[u8] = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="5" y="2" width="14" height="20" rx="2" ry="2"></rect><line x1="12" y1="18" x2="12.01" y2="18"></line></svg>"#.as_bytes();
pub const ICON_TERMINAL_SVG: &[u8] = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"></polyline><line x1="12" y1="19" x2="20" y2="19"></line></svg>"#.as_bytes();
pub const ICON_INFO_SVG: &[u8] = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="16" x2="12" y2="12"></line><line x1="12" y1="8" x2="12.01" y2="8"></line></svg>"#.as_bytes();
pub const ICON_REFRESH_SVG: &[u8] = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"></polyline><polyline points="1 20 1 14 7 14"></polyline><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path></svg>"#.as_bytes();
pub const ICON_TRASH_SVG: &[u8] = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path><line x1="10" y1="11" x2="10" y2="17"></line><line x1="14" y1="11" x2="14" y2="17"></line></svg>"#.as_bytes();
pub const ICON_POWER_SVG: &[u8] = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18.36 6.64a9 9 0 1 1-12.73 0"></path><line x1="12" y1="2" x2="12" y2="12"></line></svg>"#.as_bytes();
pub const ICON_EYE_SVG: &[u8] = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path><circle cx="12" cy="12" r="3"></circle></svg>"#.as_bytes();
pub const ICON_EYE_OFF_SVG: &[u8] = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path><line x1="1" y1="1" x2="23" y2="23"></line></svg>"#.as_bytes();
pub const ICON_CLOCK_SVG: &[u8] = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><polyline points="12 6 12 12 16 14"></polyline></svg>"#.as_bytes();
pub const ICON_GLOBE_SVG: &[u8] = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>"#.as_bytes();
pub const ICON_SHIELD_X_SVG: &[u8] = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path><line x1="9" y1="9" x2="15" y2="15"></line><line x1="15" y1="9" x2="9" y2="15"></line></svg>"#.as_bytes();

pub use kuvpn::ConnectionStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Connection,
    Settings,
    Console,
}

#[derive(Debug, Clone, Copy)]
pub enum SegmentPosition {
    Left,
    Middle,
    Right,
    Single,
}

#[derive(Debug, Clone)]
pub enum Message {
    TabChanged(Tab),
    UrlChanged(String),
    DomainChanged(String),
    EscalationToolChanged(String),
    LogLevelSliderChanged(f32),
    #[cfg(not(windows))]
    OpenConnectPathChanged(String),
    EmailChanged(String),
    LoginModeChanged(f32),
    ConnectPressed,
    DisconnectPressed,
    LogAppended(String),
    RequestInput(Arc<InputRequestWrapper>),
    InputChanged(String),
    SubmitInput,
    MfaPushReceived(String),
    MfaCompleteReceived,
    ClearSessionPressed,
    ConnectionFinished(Option<String>, Option<kuvpn::ErrorCategory>),
    StatusChanged(ConnectionStatus),
    Tick,
    TrayEvent(TrayIconEvent),
    MenuEvent(MenuEvent),
    CloseToTrayToggled(bool),
    ClientDecorationsToggled(bool),
    ToggleVisibility { from_close_request: bool },
    WindowOpened(iced::window::Id),
    WindowClosed(iced::window::Id),
    ResetClosePending,
    GtkTick,
    ResetSettings,
    #[cfg(not(windows))]
    TestOpenConnect,
    #[cfg(not(windows))]
    OpenConnectTestResult(Option<String>),
    CopyLogs,
    DragWindow,
    MinimizeWindow,
    QuitRequested,
    QuitAfterCleanup,
    AutoHideAfterPromptToggled(bool),
    AutoHideWindow,
    ShowPasswordHeld(bool),
}

#[derive(Debug)]
pub struct InputRequest {
    pub msg: String,
    pub is_password: bool,
    pub response_tx: oneshot::Sender<String>,
}

#[derive(Debug)]
pub struct InputRequestWrapper(pub Mutex<Option<InputRequest>>);

pub fn log_level_from_slider(val: f32) -> log::LevelFilter {
    match val.round() as i32 {
        0 => log::LevelFilter::Off,
        1 => log::LevelFilter::Error,
        2 => log::LevelFilter::Warn,
        3 => log::LevelFilter::Info,
        4 => log::LevelFilter::Debug,
        5 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Info,
    }
}

pub fn login_mode_flags(val: f32) -> (bool, bool) {
    match val.round() as i32 {
        0 => (true, false),  // Full Automatic
        1 => (false, false), // Visual Automatic
        _ => (false, true),  // Manual
    }
}

// --- Container Styles ---

pub fn card(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(COLOR_SURFACE.into()),
        border: Border {
            color: Color::from_rgb(0.18, 0.18, 0.18),
            width: 1.0,
            radius: 12.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
        ..Default::default()
    }
}

// --- Scrollbar Style ---

pub fn custom_scrollbar(_theme: &iced::Theme, status: scrollable::Status) -> scrollable::Style {
    let scroller_color = if matches!(status, scrollable::Status::Hovered { .. }) {
        Color::from_rgb(0.40, 0.40, 0.40)
    } else {
        Color::from_rgb(0.30, 0.30, 0.30)
    };

    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: scrollable::Rail {
            background: Some(Color::from_rgb(0.10, 0.10, 0.10).into()),
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
        },
        horizontal_rail: scrollable::Rail {
            background: Some(Color::from_rgb(0.10, 0.10, 0.10).into()),
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
        },
        gap: None,
        auto_scroll: scrollable::AutoScroll {
            background: iced::Background::Color(Color::TRANSPARENT),
            border: Border::default(),
            shadow: Shadow::default(),
            icon: Color::TRANSPARENT,
        },
    }
}

// --- Custom Button Styles ---

pub fn btn_primary(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(COLOR_ACCENT.into()),
        text_color: Color::WHITE,
        border: Border {
            radius: 10.0.into(),
            ..Default::default()
        },
        shadow: Shadow {
            color: Color::from_rgba(0.50, 0.0, 0.125, 0.3),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Color::from_rgb(0.60, 0.06, 0.19).into()),
            shadow: Shadow {
                color: Color::from_rgba(0.50, 0.0, 0.125, 0.5),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 16.0,
            },
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Color::from_rgb(0.40, 0.0, 0.10).into()),
            shadow: Shadow {
                color: Color::from_rgba(0.50, 0.0, 0.125, 0.2),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 4.0,
            },
            ..base
        },
        _ => base,
    }
}

pub fn btn_secondary(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Color::TRANSPARENT.into()),
        text_color: COLOR_TEXT,
        border: Border {
            color: Color::from_rgb(0.25, 0.25, 0.25),
            width: 1.0,
            radius: 10.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
            offset: Vector::new(0.0, 1.0),
            blur_radius: 4.0,
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(COLOR_SURFACE.into()),
            border: Border {
                color: Color::from_rgb(0.35, 0.35, 0.35),
                width: 1.0,
                radius: 10.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Color::from_rgb(0.08, 0.08, 0.08).into()),
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
            ..base
        },
        _ => base,
    }
}

pub fn btn_danger(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let base_color = Color::from_rgb(0.8, 0.2, 0.2);
    let base = button::Style {
        background: Some(base_color.into()),
        text_color: Color::WHITE,
        border: Border {
            radius: 10.0.into(),
            ..Default::default()
        },
        shadow: Shadow {
            color: Color::from_rgba(0.8, 0.2, 0.2, 0.3),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Color::from_rgb(0.9, 0.25, 0.25).into()),
            shadow: Shadow {
                color: Color::from_rgba(0.8, 0.2, 0.2, 0.5),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 16.0,
            },
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Color::from_rgb(0.65, 0.15, 0.15).into()),
            shadow: Shadow {
                color: Color::from_rgba(0.8, 0.2, 0.2, 0.2),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 4.0,
            },
            ..base
        },
        _ => base,
    }
}

pub fn btn_segment_selected(
    _theme: &iced::Theme,
    _status: button::Status,
    position: SegmentPosition,
) -> button::Style {
    let radius = match position {
        SegmentPosition::Left => iced::border::Radius {
            top_left: 8.0,
            top_right: 0.0,
            bottom_right: 0.0,
            bottom_left: 8.0,
        },
        SegmentPosition::Middle => iced::border::Radius {
            top_left: 0.0,
            top_right: 0.0,
            bottom_right: 0.0,
            bottom_left: 0.0,
        },
        SegmentPosition::Right => iced::border::Radius {
            top_left: 0.0,
            top_right: 8.0,
            bottom_right: 8.0,
            bottom_left: 0.0,
        },
        SegmentPosition::Single => iced::border::Radius {
            top_left: 8.0,
            top_right: 8.0,
            bottom_right: 8.0,
            bottom_left: 8.0,
        },
    };

    button::Style {
        background: Some(COLOR_ACCENT.into()),
        text_color: Color::WHITE,
        border: Border {
            radius,
            ..Default::default()
        },
        shadow: Shadow::default(),
        ..Default::default()
    }
}

pub fn btn_segment_unselected(
    _theme: &iced::Theme,
    status: button::Status,
    position: SegmentPosition,
) -> button::Style {
    let radius = match position {
        SegmentPosition::Left => iced::border::Radius {
            top_left: 8.0,
            top_right: 0.0,
            bottom_right: 0.0,
            bottom_left: 8.0,
        },
        SegmentPosition::Middle => iced::border::Radius {
            top_left: 0.0,
            top_right: 0.0,
            bottom_right: 0.0,
            bottom_left: 0.0,
        },
        SegmentPosition::Right => iced::border::Radius {
            top_left: 0.0,
            top_right: 8.0,
            bottom_right: 8.0,
            bottom_left: 0.0,
        },
        SegmentPosition::Single => iced::border::Radius {
            top_left: 8.0,
            top_right: 8.0,
            bottom_right: 8.0,
            bottom_left: 8.0,
        },
    };

    let base = button::Style {
        background: Some(Color::TRANSPARENT.into()),
        text_color: COLOR_TEXT,
        border: Border {
            color: Color::from_rgb(0.25, 0.25, 0.25),
            width: 1.0,
            radius,
        },
        shadow: Shadow::default(),
        ..Default::default()
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(Color::from_rgb(0.15, 0.15, 0.15).into()),
            border: Border {
                color: Color::from_rgb(0.35, 0.35, 0.35),
                width: 1.0,
                radius,
            },
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Color::from_rgb(0.10, 0.10, 0.10).into()),
            ..base
        },
        _ => base,
    }
}
