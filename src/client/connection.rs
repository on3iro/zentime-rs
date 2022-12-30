use crate::client::terminal_io::input::ClientInputAction;
use std::thread::sleep;
use std::time::Duration;

use crate::ipc::ClientToServerMsg;
use crate::ipc::InterProcessCommunication;
use crate::ipc::ServerToClientMsg;
use anyhow::Context;
use interprocess::local_socket::tokio::OwnedWriteHalf;

use crate::ipc::get_socket_name;
use futures::io::BufReader;
use interprocess::local_socket::tokio::LocalSocketStream;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use tokio::{select, task::yield_now};

use super::terminal_io::terminal_event::TerminalEvent;

/// Tokio task handling the connection between the client and the zentime server
pub struct ClientConnectionTask {}

impl ClientConnectionTask {
    pub async fn spawn(
        terminal_in_rx: UnboundedReceiver<ClientInputAction>,
        terminal_out_tx: UnboundedSender<TerminalEvent>,
    ) -> JoinHandle<()> {
        let socket_name = get_socket_name();

        let mut connection_tries = 0;

        // Try to receive a connection to the server (will timeout after the third attempt)
        let connection = loop {
            connection_tries += 1;

            if connection_tries == 4 {
                terminal_out_tx
                    .send(TerminalEvent::Quit {
                        msg: Some(String::from("\nCould not connect to server")),
                        error: true,
                    })
                    .expect("Could not send to terminal out");
            }

            let result = LocalSocketStream::connect(socket_name).await;

            if let Ok(conn) = result {
                break conn;
            } else {
                sleep(Duration::from_millis(200));
            }
        };

        tokio::spawn(async move {
            if let Err(error) =
                handle_connection(connection, terminal_out_tx.clone(), terminal_in_rx).await
            {
                terminal_out_tx
                    .send(TerminalEvent::Quit {
                        msg: Some(format!("{}.\nServer connection closed.", error)),
                        error: true,
                    })
                    .expect("Could not send to terminal out");
            }
        })
    }
}

/// Continously handle the connection to the server by reacting to incoming
/// [ServerToClientMsg] and terminal input events.
async fn handle_connection(
    connection: LocalSocketStream,
    terminal_out_tx: UnboundedSender<TerminalEvent>,
    mut terminal_in_rx: UnboundedReceiver<ClientInputAction>,
) -> anyhow::Result<()> {
    // This consumes our connection and splits it into two halves,
    // so that we could concurrently act on both.
    let (reader, mut writer) = connection.into_split();
    let mut reader = BufReader::new(reader);

    loop {
        select! {
            msg = InterProcessCommunication::recv_ipc_message::<ServerToClientMsg>(&mut reader) => {
                let msg = msg.context("Could not receive message from socket")?;
                handle_server_to_client_msg(msg, &terminal_out_tx).context("Could not handle server to client message")?;
            },
            value = terminal_in_rx.recv() => {
                if let Some(action) = value {
                    handle_client_input_action(action, &terminal_out_tx, &mut writer).await.context("Could not handle input action")?;
                }
            }
        };

        yield_now().await;
    }
}

/// Handle incoming [ClientInputAction]s
async fn handle_client_input_action(
    action: ClientInputAction,
    terminal_out_tx: &UnboundedSender<TerminalEvent>,
    writer: &mut OwnedWriteHalf,
) -> anyhow::Result<()> {
    match action {
        // Command server to shutdown and quit the current client
        ClientInputAction::Quit => {
            let msg = ClientToServerMsg::Quit;
            InterProcessCommunication::send_ipc_message(msg, writer)
                .await
                .context("Could not send IPC message")?;

            // Shutdown current client
            terminal_out_tx
                .send(TerminalEvent::Quit {
                    msg: Some(String::from("Cya!")),
                    error: false,
                })
                .context("Could not send to terminal out")?;
        }

        // Quit the current client (but keep the server running)
        ClientInputAction::Detach => {
            let msg = ClientToServerMsg::Detach;
            InterProcessCommunication::send_ipc_message(msg, writer)
                .await
                .context("Could not send IPC message")?;

            // Shutdown current client, but keep server running
            terminal_out_tx
                .send(TerminalEvent::Quit {
                    msg: None,
                    error: false,
                })
                .context("Could not send to terminal out")?;
        }

        // NoOp
        ClientInputAction::None => return Ok(()),

        // Command the server to pause or play the timer
        ClientInputAction::PlayPause => {
            let msg = ClientToServerMsg::PlayPause;
            InterProcessCommunication::send_ipc_message(msg, writer)
                .await
                .context("Could not send IPC message")?;
        }

        // Command the server to skip to the next interval
        ClientInputAction::Skip => {
            let msg = ClientToServerMsg::Skip;
            InterProcessCommunication::send_ipc_message(msg, writer)
                .await
                .context("Could not send IPC message")?;
        }

        ClientInputAction::Reset => {
            let msg = ClientToServerMsg::Reset;
            InterProcessCommunication::send_ipc_message(msg, writer)
                .await
                .context("Could not send IPC message")?;
        }
    }

    Ok(())
}

/// Handle incoming [ServerToClientMsg]s (e.g. by sending incoming timer state to the
/// [TerminalOutputTask]).
fn handle_server_to_client_msg(
    msg: ServerToClientMsg,
    terminal_out_tx: &UnboundedSender<TerminalEvent>,
) -> anyhow::Result<()> {
    match msg {
        ServerToClientMsg::Timer(state) => {
            terminal_out_tx
                .send(TerminalEvent::View(state))
                .context("Could not send to terminal out")?;
        }
    }

    Ok(())
}
