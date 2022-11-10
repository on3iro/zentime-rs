use std::path::PathBuf;

use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use zentime_rs_timer::config::Config;

pub fn create_config(config_path: &str) -> Figment {
    let mut path_buffer = PathBuf::new();
    path_buffer.push(shellexpand::tilde(config_path).as_ref());
    Figment::from(Serialized::defaults(Config::default())).merge(Toml::file(path_buffer))
}
