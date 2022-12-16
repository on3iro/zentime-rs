//! Code related to the runtime configuration of zentime

use serde::{ser::SerializeMap, Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use zentime_rs_timer::config::TimerConfig;

use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};

// NOTE:
// How to enhance the config with options also available as cli args:
//
// Currently the configuration merging for configuration options which should also be available
// as CLI arguments is a bit more involved than I would like.
// If you want to add an option that should be available as a CLI argument as well, these are the
// steps you need to take:
//
// 1. Add the configuration option to the config struct(s) below (you might add another nesting
//    layer if necessary or add it to an existing one)
// 2. If your option has a data type not yet supported by [ConfigValue] add a wrapping type to
//    [ConfigValue] and also make sure to add a new match arm to the [Serialize]-implementation for
//    [ConfigValue].
// 3. Add a new method to the [PartialConfigBuilder]
// 4. Add an optional arg to the [ServerConfig] or [ClientConfig] inside main.rs
// 5. If you enhanced the [ServerConfig] you need to also add the option to
//    [get_server_config] inside server.rs and [get_server_args] inside default_cmd.rs.
//    If you enhanced the [ClientConfig] you need to add the option to [get_client_config] inside
//    default_cmd.rs
//
// I hope that we'll be able to come up with a less brittle and boilerplatey solution in the
// future, but for now that's the way it is ;)

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
    pub timers: TimerConfig,

    /// Configuration for OS notifications
    pub notifications: NotificationConfig,
}

/// Config value used for the creation of a config hashmap via the [PartialConfigBuilder]
#[derive(Debug, Deserialize, Clone)]
pub enum ConfigValue {
    /// Config value of type [String]
    String(String),

    /// Config value of type [bool]
    Bool(bool),

    /// Config value of type [f32]
    Float(f32),

    /// Config value of type [u64]
    Int(u64),

    /// Config value of type [HashMap]
    HashMap(HashMap<String, Self>),
}

impl Serialize for ConfigValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ConfigValue::String(v) => serializer.serialize_str(v),
            ConfigValue::Bool(v) => serializer.serialize_bool(v.to_owned()),
            ConfigValue::Float(v) => serializer.serialize_f32(v.to_owned()),
            ConfigValue::Int(v) => serializer.serialize_u64(v.to_owned()),
            ConfigValue::HashMap(v) => {
                let mut map = serializer.serialize_map(Some(v.len()))?;
                for (k, value) in v {
                    map.serialize_entry(k, value)?;
                }
                map.end()
            }
        }
    }
}

/// Builder that allows the creation of partial configuration hashmap at runtime
#[derive(Debug)]
pub struct PartialConfigBuilder {
    partial_config: HashMap<String, ConfigValue>,
}

// TODO try to get rid of all that boilerplate
/// Configbuilder that allows to build a partial configuration at runtime which can than be merged into
/// a [figment::Figment]. This can be useful if there are configuration sources which only provide
/// a few configurations e.g. via clap arguments.
///
/// # Example
///
/// ```no_run
/// # struct Args {
/// #   enable_bell: Option<bool>,
/// #   volume: Option<f32>
/// # }
///
/// use figment::providers::Serialized;
/// use zentime_rs::config::{Config, create_base_config, PartialConfigBuilder};
///
/// // Just some example args (in a real application these would probably be derived by something
/// // like clap)
/// let args = Args { enable_bell: Some(true), volume: None };
///
/// let mut server_config_builder = PartialConfigBuilder::new();
///
/// if let Some(enable_bell) = &args.enable_bell {
///     server_config_builder.enable_bell(*enable_bell);
/// }
///
/// if let Some(volume) = &args.volume {
///     server_config_builder.volume(*volume);
/// }
///
/// let server_config = server_config_builder.build();
///
/// let config_path = "./some/path";
/// let config: Config = create_base_config(config_path)
///     .merge(Serialized::defaults(server_config))
///     .extract()
///     .expect("Could not create config");
/// ```
impl PartialConfigBuilder {
    /// Instantiates a new [Self]
    pub fn new() -> Self {
        Self {
            partial_config: HashMap::from([
                ("view".to_string(), ConfigValue::HashMap(HashMap::from([]))),
                (
                    "timers".to_string(),
                    ConfigValue::HashMap(HashMap::from([])),
                ),
                (
                    "notifications".to_string(),
                    ConfigValue::HashMap(HashMap::from([])),
                ),
            ]),
        }
    }

    /// Adds the interface type to the "view" part of the configuration
    pub fn interface(&mut self, value: String) -> &mut Self {
        if let Some(ConfigValue::HashMap(map)) = self.partial_config.get_mut("view") {
            map.insert("interface".to_string(), ConfigValue::String(value));
        }

        self
    }

    /// Adds the enable_bell flag to the "notifications" part of the configuration
    pub fn enable_bell(&mut self, value: bool) -> &mut Self {
        if let Some(ConfigValue::HashMap(map)) = self.partial_config.get_mut("notifications") {
            map.insert("enable_bell".to_string(), ConfigValue::Bool(value));
        }

        self
    }

    /// Adds the show_notification flag to the "notifications" part of the configuration
    pub fn show_notification(&mut self, value: bool) -> &mut Self {
        if let Some(ConfigValue::HashMap(map)) = self.partial_config.get_mut("notifications") {
            map.insert("show_notification".to_string(), ConfigValue::Bool(value));
        }

        self
    }

    /// Adds the volume value to the "notifications" part of the configuration
    pub fn volume(&mut self, value: f32) -> &mut Self {
        if let Some(ConfigValue::HashMap(map)) = self.partial_config.get_mut("notifications") {
            map.insert("volume".to_string(), ConfigValue::Float(value));
        }

        self
    }

    /// Adds the timer value to the "timers" part of the configuration
    pub fn timer(&mut self, value: u64) -> &mut Self {
        if let Some(ConfigValue::HashMap(map)) = self.partial_config.get_mut("timers") {
            map.insert("timer".to_string(), ConfigValue::Int(value));
        }

        self
    }

    /// Adds the minor_break value to the "timers" part of the configuration
    pub fn minor_break(&mut self, value: u64) -> &mut Self {
        if let Some(ConfigValue::HashMap(map)) = self.partial_config.get_mut("timers") {
            map.insert("minor_break".to_string(), ConfigValue::Int(value));
        }

        self
    }

    /// Adds the major_break value to the "timers" part of the configuration
    pub fn major_break(&mut self, value: u64) -> &mut Self {
        if let Some(ConfigValue::HashMap(map)) = self.partial_config.get_mut("timers") {
            map.insert("major_break".to_string(), ConfigValue::Int(value));
        }

        self
    }

    /// Adds the intervals value to the "timers" part of the configuration
    pub fn intervals(&mut self, value: u64) -> &mut Self {
        if let Some(ConfigValue::HashMap(map)) = self.partial_config.get_mut("timers") {
            map.insert("intervals".to_string(), ConfigValue::Int(value));
        }

        self
    }

    /// Consumes the builder and returns the config hashmap
    pub fn build(self) -> HashMap<String, ConfigValue> {
        self.partial_config
    }
}

impl Default for PartialConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a base configuration [Figment] by trying to open a configuration file
/// from a given path and merging its configuration with the zentime default configuration.
pub fn create_base_config(config_path: &str) -> Figment {
    let mut path_buffer = PathBuf::new();
    path_buffer.push(shellexpand::tilde(config_path.trim()).as_ref());

    Figment::from(Serialized::defaults(Config::default())).merge(Toml::file(path_buffer))
}
