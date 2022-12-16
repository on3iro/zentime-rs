use crate::ClientConfig;
use figment::providers::Serialized;
use std::env::current_dir;
use std::process;
use sysinfo::Pid;
use zentime_rs::client::start;
use zentime_rs::config::create_base_config;
use zentime_rs::config::Config;
use zentime_rs::config::PartialConfigBuilder;

use sysinfo::ProcessExt;
use sysinfo::System;
use sysinfo::SystemExt;
use tokio::process::Command;

use crate::CommonArgs;

#[tokio::main]
pub async fn default_cmd(common_args: &CommonArgs, client_config: &ClientConfig) {
    let config_path = &common_args.config;
    let config: Config = get_client_config(config_path, client_config);

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

        let server_args = get_server_args(common_args);

        if let Err(error) = Command::new(current_process.exe())
            .arg("server")
            .arg("start")
            .args(server_args)
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

// TODO try to get rid of all that boilerplate
fn get_server_args(common_args: &CommonArgs) -> Vec<String> {
    let mut args: Vec<String> = vec![
        // Config path
        "-c".to_string(),
        common_args.config.to_string(),
    ];

    if let Some(enable_bell) = &common_args.server_config.enable_bell {
        args.push("--enable-bell".to_string());
        args.push(enable_bell.to_string());
    }

    if let Some(volume) = &common_args.server_config.volume {
        args.push("--volume".to_string());
        args.push(volume.to_string());
    }

    if let Some(show_notification) = &common_args.server_config.show_notification {
        args.push("--show-notification".to_string());
        args.push(show_notification.to_string());
    }

    if let Some(timer) = &common_args.server_config.timer {
        args.push("--timer".to_string());
        args.push(timer.to_string());
    }

    if let Some(minor_break) = &common_args.server_config.minor_break {
        args.push("--minor-break".to_string());
        args.push(minor_break.to_string());
    }

    if let Some(major_break) = &common_args.server_config.major_break {
        args.push("--major-break".to_string());
        args.push(major_break.to_string());
    }

    if let Some(intervals) = &common_args.server_config.intervals {
        args.push("--intervals".to_string());
        args.push(intervals.to_string())
    }

    args
}

fn get_client_config(config_path: &str, client_config: &ClientConfig) -> Config {
    let mut client_config_builder = PartialConfigBuilder::new();

    if let Some(interface) = &client_config.interface {
        client_config_builder.interface(interface.to_string());
    }

    let client_config = client_config_builder.build();

    create_base_config(config_path)
        .merge(Serialized::defaults(client_config))
        .extract()
        .expect("Could not create config")
}
