use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use zentime_rs_timer::config::TimerConfig;

use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};

#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct NotificationConfig {
    /// Enable/Disable bell
    pub enable_bell: bool,

    /// Notification bell volume
    pub volume: f32,

    /// Show OS-notification
    pub show_notification: bool,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        NotificationConfig {
            volume: 0.5,
            enable_bell: true,
            show_notification: true,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct ViewConfig {
    pub interface: String,
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct Config {
    pub view: ViewConfig,
    pub timers: TimerConfig,
    pub notifications: NotificationConfig,
}

pub fn create_config(config_path: &str) -> Figment {
    let mut path_buffer = PathBuf::new();
    path_buffer.push(shellexpand::tilde(config_path).as_ref());
    Figment::from(Serialized::defaults(Config::default())).merge(Toml::file(path_buffer))
}
