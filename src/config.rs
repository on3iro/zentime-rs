//! Code related to the runtime configuration of zentime

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use zentime_rs_timer::config::PomodoroTimerConfig;

use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};

/// Configuration of notifications which are being send to the OS after each
/// interval/break
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct NotificationConfig {
    /// Enable/Disable bell
    pub enable_bell: bool,

    /// Soundfile to be played back on each interval end.
    /// Will default to a bell sound, if `None`
    pub sound_file: Option<String>,

    /// Notification bell volume
    pub volume: f32,

    /// Show OS-notification
    pub show_notification: bool,

    /// A random suggestion will be picked on each break and shown inside the
    /// notification text.
    pub break_suggestions: Option<Vec<String>>,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        NotificationConfig {
            volume: 0.5,
            sound_file: None,
            enable_bell: true,
            show_notification: true,
            break_suggestions: None,
        }
    }
}

/// Configuration of the interface
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ViewConfig {
    #[doc = include_str!("./ViewConfig.md")]
    pub interface: String,
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self {
            interface: "default".to_string(),
        }
    }
}

/// Zentime configuration
#[derive(Deserialize, Serialize, Clone, Default, Debug)]
pub struct Config {
    /// Interface configuration
    pub view: ViewConfig,

    /// Configuration of the timer itself
    pub timers: PomodoroTimerConfig,

    /// Configuration for OS notifications
    pub notifications: NotificationConfig,
}

/// Creates a base configuration [Figment] by trying to open a configuration file
/// from a given path and merging its configuration with the zentime default configuration.
pub fn create_base_config(config_path: &str) -> Figment {
    let mut path_buffer = PathBuf::new();
    path_buffer.push(shellexpand::tilde(config_path.trim()).as_ref());

    Figment::from(Serialized::defaults(Config::default())).merge(Toml::file(path_buffer))
}
