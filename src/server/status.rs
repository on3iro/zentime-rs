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
    // TODO
    // * add a way to connect to a different server during development (e.g. by specifying the
    // socket address - or making use of the executable path)
    let system = System::new_all();

    let zentime_process_instances = system.processes_by_name("zentime");

    // WHY:
    // We identify a server process by its command (e.g. "zentime server start").
    // This process itself will be one instance, so if we have two instances there is already
    // another server process running and we don't have to start this one and can exit early.
    let server_is_running = zentime_process_instances
        .filter(|p| p.cmd().contains(&String::from("server")))
        .count()
        == 2;

    if server_is_running {
        ServerStatus::Running
    } else {
        ServerStatus::Stopped
    }
}
