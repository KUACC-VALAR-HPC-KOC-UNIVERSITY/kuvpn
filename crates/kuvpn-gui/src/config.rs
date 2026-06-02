use crate::theme::ThemeConfig;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GuiSettings {
    pub url: String,
    pub domain: String,
    pub email: String,
    pub escalation_tool: String,
    pub log_level_val: f32,
    pub openconnect_path: String,
    pub login_mode_val: f32,
    pub close_to_tray: bool,
    pub use_client_decorations: bool,
    #[serde(default = "default_auto_hide")]
    pub auto_hide_after_prompt: bool,
    #[serde(default)]
    pub advanced_mode: bool,
    #[serde(default)]
    pub theme: ThemeConfig,
    /// Tunnel mode: 0.0 = Split, 1.0 = Full, 2.0 = Manual.
    /// Manual mode uses `vpnc_script` as the script path.
    #[serde(default = "default_tunnel_mode_val")]
    pub tunnel_mode_val: f32,
    /// Path to a custom vpnc-script passed via --script to openconnect (advanced).
    /// Only used when `tunnel_mode_val` == 2.0 (Manual).
    #[serde(default)]
    pub vpnc_script: String,
    /// Config schema version for migration.
    #[serde(default)]
    pub config_version: u32,
}

fn default_auto_hide() -> bool {
    true
}

fn default_tunnel_mode_val() -> f32 {
    1.0 // Full tunnel
}

impl Default for GuiSettings {
    fn default() -> Self {
        Self {
            url: "https://vpn.ku.edu.tr".to_string(),
            domain: "vpn.ku.edu.tr".to_string(),
            email: String::new(),
            escalation_tool: "sudo".to_string(),
            log_level_val: 3.0, // Default: Info level
            openconnect_path: "openconnect".to_string(),
            login_mode_val: 0.0,
            close_to_tray: true,
            use_client_decorations: true,
            auto_hide_after_prompt: true,
            advanced_mode: false,
            theme: ThemeConfig::default(),
            tunnel_mode_val: 1.0, // Full tunnel
            vpnc_script: String::new(),
            config_version: Self::CURRENT_VERSION,
        }
    }
}

impl GuiSettings {
    const CURRENT_VERSION: u32 = 1;

    /// Runs any pending migrations and stamps the current version.
    /// Add new migration steps here as fields change across releases.
    fn migrate(&mut self) {
        // 0 → 1: first versioned release; no field changes needed.
        self.config_version = Self::CURRENT_VERSION;
    }

    /// Returns `true` when the tunnel mode slider is set to Manual (value 2).
    pub fn is_manual_mode(&self) -> bool {
        self.tunnel_mode_val.round() as i32 == 2
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let dir = kuvpn::utils::get_user_data_dir().map_err(|e| anyhow::anyhow!("{}", e))?;
        let path = dir.join("gui_settings.json");
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load() -> Self {
        if let Ok(dir) = kuvpn::utils::get_user_data_dir() {
            let path = dir.join("gui_settings.json");
            if path.exists() {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(mut settings) = serde_json::from_str::<Self>(&content) {
                        settings.migrate();
                        return settings;
                    }
                }
            }
        }
        Self::default()
    }
}
