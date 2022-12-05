use std::env::current_dir;
use std::process;
use sysinfo::Pid;

use sysinfo::ProcessExt;
use sysinfo::System;
use sysinfo::SystemExt;
use tokio::fs::canonicalize;
use tokio::process::Command;

use crate::config::{create_config, Config};

use crate::client;

// TODO
// * [x] differentiate between server start and attach
// * [x] use IPC between client and server
// * [x] better error handling  (replace unwraps, use results etc.)
// * [ ] docs
// * [ ] refactor and simplify
// * [ ] tests
// * [x] check what happens if too many client connections are opened
// * [ ] add status command (to check if daemon is currently running)
// * [ ] add command to kill server without attaching a client beforehand
// * [ ] handle termination signal gracefully so that we always disable raw mode on termination etc.

#[tokio::main]
pub async fn default_cmd(config_path: &str) {
    let config: Config = create_config(config_path)
        .extract()
        .expect("Could not create config");

    // TODO
    // * check if another zentime process is already running
    // * if not, spawn zentime server start process
    // * start client afterwards
    let system = System::new_all();

    // NOTE: This is a bit brittle during development, because you could
    // technically run another zentime process from another version
    // FIXME - make this more robust (and also the check inside the server::start() method)
    let current_is_only_process_instance = system.processes_by_name("zentime").count() == 1;

    // We need to spawn a server process before we can attach our client
    if current_is_only_process_instance {
        // WHY:
        // We want to get information about the current zentime process, e.g.
        // the path to its executable. That way this does also work in ci or during
        // development, where one might not have added a specific zentime binary to their path.
        let current_process = system
            .process(Pid::from(process::id() as i32))
            .expect("Could not retrieve information for current zentime process");

        let current_dir = current_dir()
            .expect("Could not get current directory")
            .into_os_string();

        if let Err(error) = Command::new(current_process.exe())
            .arg("server")
            .arg("start")
            // NOTE: it's important that the '{' starts
            // immediately after the -c flag, because otherwise the config path would have
            // additional whitespace. (We trim() the path inside [config::create_config()], but
            // still...)
            .arg(format!("-c{}", &config_path))
            .current_dir(current_dir)
            .spawn()
            .expect("Could not start server daemon")
            .wait()
            .await
        {
            panic!("Server exited unexpectedly: {}", error)
        };
    }

    client::start(config).await;
}
