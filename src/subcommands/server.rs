use daemonize::Daemonize;
use figment::providers::Serialized;
use interprocess::local_socket::tokio::LocalSocketStream;
use log::{error, info};
use std::env::current_dir;
use std::fs::File;
use std::thread::sleep;
use std::time::Duration;
use zentime_rs::config::create_base_config;
use zentime_rs::config::Config;
use zentime_rs::config::PartialConfigBuilder;
use zentime_rs::ipc::get_socket_name;
use zentime_rs::ipc::ClientToServerMsg;
use zentime_rs::ipc::InterProcessCommunication;
use zentime_rs::server::start;
use zentime_rs::server::status::server_status;

use crate::CommonArgs;

/// Daemonizes the current process and then starts a zentime server instance in it (if there isn't
/// another server already running - otherwise the process terminates).
///
/// NOTE: It's important, that we run this synchronously.
/// [server::start()] will then create a tokio runtime, after the process has been
/// deamonized
pub fn start_daemonized(args: &CommonArgs) {
    let stdout_path = "/tmp/zentime.d.out";
    let stdout = File::create(stdout_path)
        .unwrap_or_else(|error| panic!("Could not create {}: {}", stdout_path, error));
    let stderr_path = "/tmp/zentime.d.err";
    let stderr = File::create(stderr_path)
        .unwrap_or_else(|error| panic!("Could not create {}: {}", stderr_path, error));

    let current_directory = current_dir()
        .expect("Could not get current directory")
        .into_os_string();

    let daemonize = Daemonize::new()
        .working_directory(current_directory)
        .stdout(stdout) // Redirect stdout to `/tmp/daemon.out`.
        .stderr(stderr); // Redirect stderr to `/tmp/daemon.err`.

    if let Err(error) = daemonize.start() {
        panic!("Could not daemonize server process: {}", error);
    };

    info!("Daemonized server process");

    let config = get_server_config(args);

    if let Err(error) = start(config) {
        error!("A server error occured: {}", error);
    };
}

// TODO try to get rid of all that boilerplate
fn get_server_config(args: &CommonArgs) -> Config {
    let config_path = &args.config;
    info!("Creating config from path: {}", config_path);

    let mut server_config_builder = PartialConfigBuilder::new();

    if let Some(enable_bell) = &args.server_config.enable_bell {
        server_config_builder.enable_bell(*enable_bell);
    }

    if let Some(volume) = &args.server_config.volume {
        server_config_builder.volume(*volume);
    }

    if let Some(show_notification) = &args.server_config.show_notification {
        server_config_builder.show_notification(*show_notification);
    }

    if let Some(timer) = &args.server_config.timer {
        server_config_builder.timer(*timer);
    }

    if let Some(minor_break) = &args.server_config.minor_break {
        server_config_builder.minor_break(*minor_break);
    }

    if let Some(major_break) = &args.server_config.major_break {
        server_config_builder.major_break(*major_break);
    }

    if let Some(intervals) = &args.server_config.intervals {
        server_config_builder.intervals(*intervals);
    }

    let server_config = server_config_builder.build();

    create_base_config(config_path)
        .merge(Serialized::defaults(server_config))
        .extract()
        .expect("Could not create config")
}

/// Stops a currently running zentime server (there can only ever be a single instance - all
/// clients will automatically shutdown, when their connection closes).
#[tokio::main]
pub async fn stop() {
    let socket_name = get_socket_name();

    let mut connection_tries = 0;

    info!("Connecting to server...");

    let connection = loop {
        connection_tries += 1;

        if connection_tries == 4 {
            panic!("Could not connect to server");
        }

        let result = LocalSocketStream::connect(socket_name).await;

        if let Ok(conn) = result {
            break conn;
        } else {
            sleep(Duration::from_millis(200));
        }
    };

    info!("Shutting down...");

    let (_, mut writer) = connection.into_split();

    let msg = ClientToServerMsg::Quit;
    InterProcessCommunication::send_ipc_message(msg, &mut writer)
        .await
        .expect("Could not send Quit message");

    info!("Done.");
}

/// Prints the current status of the zentime server
pub fn status() {
    println!("Server is {}", server_status());
}
