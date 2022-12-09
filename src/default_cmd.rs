use std::env::current_dir;
use std::process;
use sysinfo::Pid;
use zentime_rs::client::start;
use zentime_rs::config::create_config;
use zentime_rs::config::Config;

use sysinfo::ProcessExt;
use sysinfo::System;
use sysinfo::SystemExt;
use tokio::process::Command;

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

    start(config).await;
}
