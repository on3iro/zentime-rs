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

pub struct ClientConnectionTask {}

impl ClientConnectionTask {
    pub async fn spawn(
        terminal_in_rx: UnboundedReceiver<ClientInputAction>,
        terminal_out_tx: UnboundedSender<TerminalEvent>,
    ) -> JoinHandle<()> {
        let socket_name = get_socket_name();

        let mut connection_tries = 0;

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

        tokio::spawn(async move {
            if let Err(error) = handle_connection(connection, terminal_out_tx, terminal_in_rx).await
            {
                panic!("Could not handle client connection: {}", error);
            }
        })
    }
}

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

async fn handle_client_input_action(
    action: ClientInputAction,
    terminal_out_tx: &UnboundedSender<TerminalEvent>,
    writer: &mut OwnedWriteHalf,
) -> anyhow::Result<()> {
    match action {
        ClientInputAction::Quit => {
            let msg = ClientToServerMsg::Quit;
            InterProcessCommunication::send_ipc_message(msg, writer)
                .await
                .context("Could not send IPC message")?;
        }
        ClientInputAction::Detach => {
            // Shutdown current client, but keep server running
            terminal_out_tx
                .send(TerminalEvent::Quit)
                .context("Could not send to terminal out")?;
        }
        ClientInputAction::None => return Ok(()),
        ClientInputAction::PlayPause => {
            let msg = ClientToServerMsg::PlayPause;
            InterProcessCommunication::send_ipc_message(msg, writer)
                .await
                .context("Could not send IPC message")?;
        }
        ClientInputAction::Skip => {
            let msg = ClientToServerMsg::Skip;
            InterProcessCommunication::send_ipc_message(msg, writer)
                .await
                .context("Could not send IPC message")?;
        }
    }

    Ok(())
}

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
        ServerToClientMsg::Quit => {
            terminal_out_tx
                .send(TerminalEvent::Quit)
                .context("Could not send to terminal out")?;
        }
    }

    Ok(())
}
