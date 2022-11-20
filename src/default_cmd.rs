use crate::config::{create_config, Config};

use crate::client;
use crate::server;

// TODO
// * [ ] differentiate between server start and attach
// * [ ] use IPC between client and server
// * [ ] better error handling  (replace unwraps, use results etc.)
// * [ ] docs
// * [ ] refactor and simplify
// * [ ] tests
// * [ ] check what happens if too many client connections are opened
// * [ ] add status command (to check if daemon is currently running)
// * [ ] add command to kill server without attaching a client beforehand
// * [ ] handle termination signal gracefully so that we always disable raw mode on termination etc.

pub async fn default_cmd(config_path: &str) {
    let config: Config = create_config(config_path)
        .extract()
        .expect("Could not create config");

    let server_config = config.clone();

    server::start(server_config).await.unwrap();
    client::start(config).await;

    // server_handle.join().expect("Could not join server thread");
}
