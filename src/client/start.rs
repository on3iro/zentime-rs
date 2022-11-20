use std::sync::Arc;

// TODO move input into terminal_io mod
use crate::client::input::TerminalInputTask;
use crate::client::terminal_io::output::TerminalOut;
use crate::client::terminal_io::terminal::Terminal;
use crate::config::Config;
use futures::future::FutureExt;
use futures::lock::Mutex;
use tokio::sync::mpsc::unbounded_channel;
use tokio::try_join;

use super::connection::ClientConnectionTask;
use crate::client::terminal_io::output::DefaultInterface;
use crate::client::terminal_io::output::MinimalInterface;

pub async fn start(config: Config) {
    let (terminal_in_tx, terminal_in_rx) = unbounded_channel();
    let (terminal_out_tx, terminal_out_rx) = unbounded_channel();

    let interface_type = config.view.interface.clone();

    let terminal_out: Box<dyn TerminalOut + Send> = if interface_type == "minimal" {
        // TODO maybe try to avoid duplication between both interface types
        match MinimalInterface::new() {
            Ok(interface) => Box::new(interface),
            Err(error) => {
                panic!("Could not initialize interface: {}", error);
            }
        }
    } else {
        match DefaultInterface::new() {
            Ok(interface) => Box::new(interface),
            Err(error) => {
                panic!("Could not initialize interface: {}", error);
            }
        }
    };

    let thread_safe_terminal_out = Arc::new(Mutex::new(terminal_out));

    let input_handler = TerminalInputTask::spawn(terminal_in_tx);
    let view_handler = Terminal::spawn_renderer(thread_safe_terminal_out.clone(), terminal_out_rx);
    let connection_handler = ClientConnectionTask::spawn(terminal_in_rx, terminal_out_tx);

    let join_result = try_join! {
        connection_handler.flatten(),
        input_handler.flatten(),
        view_handler.flatten(),
    };

    match join_result {
        Ok(_) => todo!(),
        Err(error) => thread_safe_terminal_out
            .lock()
            .await
            .quit(Some(&format!("ERROR: {}", error)), true),
    }
}
