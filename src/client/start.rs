//! Module containing code relevant to starting a zentime terminal client.
//! See [`start`]

use crate::client::terminal_io::output::RawInterface;
use crate::client::terminal_io::output::TerminalOutputTask;
use std::sync::Arc;

use crate::client::terminal_io::input::TerminalInputTask;
use crate::client::terminal_io::output::TerminalOut;
use crate::config::Config;
use futures::future::FutureExt;
use futures::lock::Mutex;
use tokio::sync::mpsc::unbounded_channel;
use tokio::try_join;

use super::connection::ClientConnectionTask;
use crate::client::terminal_io::output::DefaultInterface;
use crate::client::terminal_io::output::MinimalInterface;

/// Start a single zentime client and connect it to the zentime server.
/// This makes sure we have tokio tasks in place to:
/// * listen to incoming input events
/// * render output to the terminal
/// * hanndle communication over a client-server connection via IPC-message passing
///
/// # Example
///
/// ```no_run
/// use zentime_rs::client::start;
/// use zentime_rs::config::create_base_config;
/// use zentime_rs::config::Config;
///
/// #[tokio::main]
/// async fn main() {
///     let config: Config = create_base_config("./some/path/config.toml")
///        .extract()
///        .expect("Could not create config");
///     start(config).await;
/// }
/// ```
pub async fn start(config: Config) {
    let (terminal_in_tx, terminal_in_rx) = unbounded_channel();
    let (terminal_out_tx, terminal_out_rx) = unbounded_channel();

    let interface_type = config.view.interface.clone();

    let terminal_out: Box<dyn TerminalOut + Send> = init_interface(interface_type);

    let thread_safe_terminal_out = Arc::new(Mutex::new(terminal_out));

    let input_handler = TerminalInputTask::spawn(terminal_in_tx);
    let view_handler = TerminalOutputTask::spawn(thread_safe_terminal_out.clone(), terminal_out_rx);
    let connection_handler = ClientConnectionTask::spawn(terminal_in_rx, terminal_out_tx);

    let join_result = try_join! {
        connection_handler.flatten(),
        input_handler.flatten(),
        view_handler.flatten(),
    };

    if let Err(error) = join_result {
        thread_safe_terminal_out
            .lock()
            .await
            .quit(Some(format!("ERROR: {}", error)), true)
    }
}

/// Determine which terminal interface should be used.
fn init_interface(interface_type: String) -> Box<dyn TerminalOut + Send> {
    match interface_type.as_str() {
        "minimal" => match MinimalInterface::new() {
            Ok(interface) => Box::new(interface),
            Err(error) => {
                panic!("Could not initialize interface: {}", error);
            }
        },
        "raw" => match RawInterface::new() {
            Ok(interface) => Box::new(interface),
            Err(error) => {
                panic!("Could not initialize interface: {}", error);
            }
        },
        _ => match DefaultInterface::new() {
            Ok(interface) => Box::new(interface),
            Err(error) => {
                panic!("Could not initialize interface: {}", error);
            }
        },
    }
}
