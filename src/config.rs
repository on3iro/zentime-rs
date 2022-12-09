//! Code related to the runtime configuration of zentime

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use zentime_rs_timer::config::TimerConfig;

use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};

/// Configuration of notifications which are being send to the OS after each
/// interval/break
#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
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

/// Configuration of the interface
#[derive(Deserialize, Serialize, Clone, Default, Debug)]
pub struct ViewConfig {
    /// Type of terminal interface
    /// Can be one of:
    /// * 'default'
    /// * 'minimal'
    pub interface: String,
}

/// Zentime configuration
#[derive(Deserialize, Serialize, Clone, Default, Debug)]
pub struct Config {
    /// Interface configuration
    pub view: ViewConfig,

    /// Configuration of the timer itself
    pub timers: TimerConfig,

    /// Configuration for OS notifications
    pub notifications: NotificationConfig,
}

/// Creates a configuration [Figment] by trying to open a configuration file
/// from a given path and merging its configuration with the zentime default configuration.
pub fn create_config(config_path: &str) -> Figment {
    let mut path_buffer = PathBuf::new();
    path_buffer.push(shellexpand::tilde(config_path.trim()).as_ref());

    Figment::from(Serialized::defaults(Config::default())).merge(Toml::file(path_buffer))
}
