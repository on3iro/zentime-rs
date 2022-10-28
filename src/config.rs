use crate::PathBuf;
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TimerConfig {
    /// Timer in seconds
    pub timer: u64,

    /// Minor break time in seconds
    pub minor_break: u64,

    /// Major break time in seconds
    pub major_break: u64,

    /// Intervals before major break
    pub intervals: u64,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub timers: TimerConfig,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            timers: TimerConfig {
                timer: 20,
                minor_break: 10,
                major_break: 900,
                intervals: 4,
            },
        }
    }
}

pub fn create_config(config_path: &str) -> Figment {
    let mut path_buffer = PathBuf::new();
    path_buffer.push(shellexpand::tilde(config_path).as_ref());
    Figment::from(Serialized::defaults(Config::default())).merge(Toml::file(path_buffer))
}
