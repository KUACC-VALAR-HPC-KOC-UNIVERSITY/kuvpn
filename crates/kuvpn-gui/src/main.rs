#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod logger;
mod provider;
mod styles;
mod theme;
mod tray;
mod types;
mod view;

use crate::app::KuVpnGui;
use crate::tray::init_tray;
use crate::types::Message;
use iced::Task;
use std::sync::{Arc, Mutex};

pub fn load_window_icon() -> Option<iced::window::Icon> {
    iced::window::icon::from_file_data(crate::types::WINDOW_ICON, Some(image::ImageFormat::Png))
        .ok()
}

fn get_title(_: &KuVpnGui, _: iced::window::Id) -> String {
    "KUVPN".to_string()
}

fn get_theme(gui: &KuVpnGui, id: iced::window::Id) -> iced::Theme {
    gui.theme(id)
}

fn get_subscription(gui: &KuVpnGui) -> iced::Subscription<Message> {
    gui.subscription()
}

pub fn main() -> iced::Result {
    // VPN helper mode: invoked by the app itself under elevation to manage
    // OpenConnect's lifecycle (single UAC prompt per connection).
    // Must run before any GUI initialisation.
    #[cfg(windows)]
    if let Some(code) = kuvpn::run_vpn_helper_if_requested() {
        std::process::exit(code);
    }

    // Ensure only one instance is running
    if let Err(e) = kuvpn::utils::ensure_single_instance() {
        eprintln!("{}", e);
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        // tray-icon on Linux requires GTK to be initialized first
        let _ = gtk::init();
    }

    #[allow(clippy::arc_with_non_send_sync)]
    let components = Arc::new(Mutex::new(Some(init_tray())));

    iced::daemon(
        move || {
            let mut gui = KuVpnGui::default();

            if let Ok(mut guard) = components.lock() {
                if let Some(c) = guard.take() {
                    gui.tray_icon = Some(c.tray);
                    gui.tray_menu = Some(c.menu_items);
                }
            }

            let use_csd = gui.settings.use_client_decorations;
            let (id, task) = iced::window::open(app::window_settings(use_csd));

            gui.window_id = Some(id);
            gui.is_visible = true;

            (
                gui,
                Task::batch(vec![
                    task.map(Message::WindowOpened),
                    Task::done(Message::TestOpenConnect),
                    Task::perform(
                        async { kuvpn::load_events().unwrap_or_default() },
                        Message::HistoryLoaded,
                    ),
                ]),
            )
        },
        KuVpnGui::update,
        KuVpnGui::view,
    )
    .title(get_title)
    .subscription(get_subscription)
    .theme(get_theme)
    .run()
}
