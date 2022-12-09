use crate::config::create_config;
use crate::config::Config;
use crate::server;
use crate::server::util::server_status;
use daemonize::Daemonize;
use std::env::current_dir;
use std::fs::File;

pub fn start(config_path: &str) {
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

    let config: Config = create_config(config_path)
        .extract()
        .expect("Could not create config");

    server::start(config).unwrap();
}

pub fn stop() {
    todo!();
}

pub fn status() {
    println!("Server is {}", server_status());
}
