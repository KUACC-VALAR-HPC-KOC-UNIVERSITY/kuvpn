use tray_icon::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    TrayIcon, TrayIconBuilder,
};

use crate::types::{
    TRAY_ICON_CONNECTED, TRAY_ICON_CONNECTING, TRAY_ICON_DISCONNECTED, TRAY_ICON_NORMAL,
};

/// Groups the tray context-menu items that the app needs to update at runtime.
pub struct TrayMenuItems {
    pub status: MenuItem,
    /// Kept alive so the menu item is not dropped; the tray library manages its state.
    _show: MenuItem,
    pub connect: MenuItem,
    pub disconnect: MenuItem,
    pub wipe: MenuItem,
}

pub struct TrayComponents {
    pub tray: TrayIcon,
    pub menu_items: TrayMenuItems,
}

/// Render size for tray icons. Windows system tray uses 16–32 px; supply 32 so
/// the OS has clean source pixels for DPI scaling rather than downsampling from 512.
#[cfg(windows)]
const TRAY_RENDER_SIZE: u32 = 32;
#[cfg(not(windows))]
const TRAY_RENDER_SIZE: u32 = 256;

/// Convert SVG bytes to a tray icon
fn svg_to_tray_icon(svg_bytes: &[u8]) -> Result<tray_icon::Icon, Box<dyn std::error::Error>> {
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(svg_bytes, &opt)?;

    let size = TRAY_RENDER_SIZE;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size).ok_or("Failed to create pixmap")?;

    let transform = resvg::tiny_skia::Transform::from_scale(
        size as f32 / tree.size().width(),
        size as f32 / tree.size().height(),
    );

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Convert RGBA8 to the format expected by tray-icon
    let rgba_data = pixmap.data().to_vec();

    Ok(tray_icon::Icon::from_rgba(rgba_data, size, size)?)
}

/// Return the status label text for the tray menu item.
pub fn status_label(status: kuvpn::ConnectionStatus) -> &'static str {
    match status {
        kuvpn::ConnectionStatus::Connected => "Status: Connected",
        kuvpn::ConnectionStatus::Disconnected => "Status: Disconnected",
        kuvpn::ConnectionStatus::Error => "Status: Error",
        kuvpn::ConnectionStatus::Connecting => "Status: Connecting...",
        kuvpn::ConnectionStatus::Disconnecting => "Status: Disconnecting...",
    }
}

pub fn init_tray() -> TrayComponents {
    let status_item = MenuItem::with_id("status", "Status: Disconnected", false, None);
    let show_item = MenuItem::with_id("show", "Show / Hide", true, None);
    let connect_item = MenuItem::with_id("connect", "Connect", true, None);
    let disconnect_item = MenuItem::with_id("disconnect", "Disconnect", false, None);
    let wipe_item = MenuItem::with_id("wipe", "Wipe Session", true, None);
    let copy_logs_item = MenuItem::with_id("copy_logs", "Copy Logs", true, None);
    let settings_item = MenuItem::with_id("settings", "Settings", true, None);
    let quit_item = MenuItem::with_id("quit", "Quit", true, None);

    let tray_menu = Menu::with_items(&[
        &status_item,
        &PredefinedMenuItem::separator(),
        &show_item,
        &PredefinedMenuItem::separator(),
        &connect_item,
        &disconnect_item,
        &PredefinedMenuItem::separator(),
        &wipe_item,
        &copy_logs_item,
        &PredefinedMenuItem::separator(),
        &settings_item,
        &PredefinedMenuItem::separator(),
        &quit_item,
    ])
    .expect("Failed to create tray menu");

    // Use the normal/idle icon by default
    let icon = svg_to_tray_icon(TRAY_ICON_NORMAL).unwrap_or_else(|e| {
        log::warn!("Failed to load tray icon from SVG: {}, using fallback", e);
        // Fallback to a simple colored square
        let mut rgba = vec![0u8; 32 * 32 * 4];
        for i in 0..32 * 32 {
            rgba[i * 4] = 128; // R
            rgba[i * 4 + 1] = 0; // G
            rgba[i * 4 + 2] = 32; // B
            rgba[i * 4 + 3] = 255; // A
        }
        tray_icon::Icon::from_rgba(rgba, 32, 32).expect("Failed to create fallback icon")
    });

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("KUVPN")
        .with_icon(icon)
        .build()
        .expect("Failed to create tray icon");

    TrayComponents {
        tray,
        menu_items: TrayMenuItems {
            status: status_item,
            _show: show_item,
            connect: connect_item,
            disconnect: disconnect_item,
            wipe: wipe_item,
        },
    }
}

/// Update the tray icon and tooltip based on connection status
pub fn update_tray_icon(tray: &TrayIcon, status: kuvpn::ConnectionStatus) {
    let (svg_bytes, tooltip) = match status {
        kuvpn::ConnectionStatus::Connected => (TRAY_ICON_CONNECTED, "KUVPN — Connected"),
        kuvpn::ConnectionStatus::Disconnected => (TRAY_ICON_NORMAL, "KUVPN — Disconnected"),
        kuvpn::ConnectionStatus::Error => (TRAY_ICON_DISCONNECTED, "KUVPN — Error"),
        kuvpn::ConnectionStatus::Connecting => (TRAY_ICON_CONNECTING, "KUVPN — Connecting…"),
        kuvpn::ConnectionStatus::Disconnecting => (TRAY_ICON_CONNECTING, "KUVPN — Disconnecting…"),
    };

    let _ = tray.set_tooltip(Some(tooltip));

    match svg_to_tray_icon(svg_bytes) {
        Err(_) => log::error!("Failed to convert SVG to tray icon"),
        Ok(icon) => {
            if let Err(e) = tray.set_icon(Some(icon)) {
                log::error!("Failed to update tray icon: {}", e);
            }
        }
    }
}
