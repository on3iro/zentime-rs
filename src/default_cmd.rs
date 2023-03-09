use crate::ClientConfig;
use figment::providers::Serialized;
use std::env::current_dir;
use std::process;
use sysinfo::Pid;
use zentime_rs::client::start;
use zentime_rs::config::create_base_config;
use zentime_rs::config::Config;
use zentime_rs::server::status::server_status;
use zentime_rs::server::status::ServerStatus;

use sysinfo::ProcessExt;
use sysinfo::System;
use sysinfo::SystemExt;
use tokio::process::Command;

use crate::CommonArgs;

#[tokio::main]
pub async fn default_cmd(common_args: &CommonArgs, client_config: &ClientConfig) {
    let config_path = &common_args.config;
    let config: Config = get_client_config(config_path, client_config);

    let system = System::new_all();

    // We need to spawn a server process before we can attach our client
    if server_status() == ServerStatus::Stopped {
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

fn get_server_args(common_args: &CommonArgs) -> Vec<String> {
    let mut args: Vec<String> = vec![
        // Config path
        "-c".to_string(),
        common_args.config.to_string(),
    ];

    if let Some(postpone_limit) = &common_args.server_config.timers.postpone_limit {
        args.push("--postpone-limit".to_string());
        args.push(postpone_limit.to_string());
    }

    if let Some(postpone_timer) = &common_args.server_config.timers.postpone_timer {
        args.push("--postpone-timer".to_string());
        args.push(postpone_timer.to_string());
    }

    if let Some(enable_bell) = &common_args.server_config.notifications.enable_bell {
        args.push("--enable-bell".to_string());
        args.push(enable_bell.to_string());
    }

    if let Some(sound_file) = &common_args.server_config.notifications.sound_file {
        args.push("--sound-file".to_string());
        args.push(sound_file.to_string());
    }

    if let Some(volume) = &common_args.server_config.notifications.volume {
        args.push("--volume".to_string());
        args.push(volume.to_string());
    }

    if let Some(show_notification) = &common_args.server_config.notifications.show_notification {
        args.push("--show-notification".to_string());
        args.push(show_notification.to_string());
    }

    if let Some(timer) = &common_args.server_config.timers.timer {
        args.push("--timer".to_string());
        args.push(timer.to_string());
    }

    if let Some(minor_break) = &common_args.server_config.timers.minor_break {
        args.push("--minor-break".to_string());
        args.push(minor_break.to_string());
    }

    if let Some(major_break) = &common_args.server_config.timers.major_break {
        args.push("--major-break".to_string());
        args.push(major_break.to_string());
    }

    if let Some(intervals) = &common_args.server_config.timers.intervals {
        args.push("--intervals".to_string());
        args.push(intervals.to_string())
    }

    args
}

fn get_client_config(config_path: &str, client_config: &ClientConfig) -> Config {
    create_base_config(config_path)
        .merge(Serialized::defaults(client_config))
        .extract()
        .expect("Could not create config")
}
