use crate::config::create_config;
use crate::config::Config;
use crate::server;

pub async fn start(config_path: &str) {
    let config: Config = create_config(config_path)
        .extract()
        .expect("Could not create config");

    server::start(config).await.unwrap();
}

pub fn stop() {
    todo!();
}

pub fn status() {
    todo!();
}
