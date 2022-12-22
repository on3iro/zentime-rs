//! Code related to server status information
use std::fmt::Display;

use sysinfo::{ProcessExt, System, SystemExt};

/// Current status of the zentime server
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServerStatus {
    /// The server is active and running
    Running,

    /// No server process is currently running
    Stopped,
}

impl Display for ServerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerStatus::Running => write!(f, "running"),
            ServerStatus::Stopped => write!(f, "not running"),
        }
    }
}

/// Gets the current status of the zentime server, by checking if a process is running
/// which was started by a `zentime server`-command.
pub fn server_status() -> ServerStatus {
    let system = System::new_all();

    let mut zentime_process_instances = system.processes_by_name("zentime");

    // WHY:
    // We identify a server process by its command (e.g. "zentime server start") and assume that
    // there is no other way, that the word "start" is part of a server command
    //
    // NOTE: During debug builds we use a different socket and therefore the server is not
    // shared with the production one
    let server_is_running = if cfg!(debug_assertions) {
        zentime_process_instances.any(|p| {
            p.cmd()[0].contains(&String::from("target/debug"))
                && p.cmd().contains(&String::from("server"))
                && p.cmd().contains(&String::from("start"))
        })
    } else {
        zentime_process_instances.any(|p| {
            !p.cmd()[0].contains(&String::from("target/debug"))
                && p.cmd().contains(&String::from("server"))
                && p.cmd().contains(&String::from("start"))
        })
    };

    if server_is_running {
        ServerStatus::Running
    } else {
        ServerStatus::Stopped
    }
}
