use crate::ipc::ClientToServerMsg;
use crate::ipc::InterProcessCommunication;
use crate::ipc::ServerToClientMsg;
use anyhow::Context;

use crate::ipc::get_socket_name;
use futures::io::BufReader;
use interprocess::local_socket::tokio::LocalSocketStream;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use tokio::{fs::metadata, select, task::yield_now};

use super::input::ClientInputAction;
use super::terminal_io::terminal_event::TerminalEvent;

pub struct ClientConnectionTask {}

impl ClientConnectionTask {
    pub async fn spawn(
        terminal_in_rx: UnboundedReceiver<ClientInputAction>,
        terminal_out_tx: UnboundedSender<TerminalEvent>,
    ) -> JoinHandle<()> {
        let socket_name = get_socket_name();

        // Check that socket is ready
        metadata(socket_name).await.unwrap_or_else(|error| {
            panic!("Could not get socket metadata: {}", error);
        });

        let connection = LocalSocketStream::connect(socket_name)
            .await
            .unwrap_or_else(|error| {
                panic!("Could not connect client to socket: {}", error);
            });

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

                match msg {
                    ServerToClientMsg::Timer(state) => {
                        terminal_out_tx.send(TerminalEvent::View(state)).context("Could not send to terminal out")?;
                    },
                    ServerToClientMsg::Quit => {
                        terminal_out_tx.send(TerminalEvent::Quit).context("Could not send to terminal out")?;
                    },
                }
            },
            value = terminal_in_rx.recv() => {
                if let Some(event) = value {
                    match event {
                        ClientInputAction::Quit => {
                            let msg = ClientToServerMsg::Quit;
                            InterProcessCommunication::send_ipc_message(msg, &mut writer).await.context("Could not send IPC message")?;
                        },
                        ClientInputAction::Detach => {
                            // Shutdown current client, but keep server running
                            terminal_out_tx.send(TerminalEvent::Quit).context("Could not send to terminal out")?;
                        },
                        ClientInputAction::None => continue,
                        ClientInputAction::PlayPause => {
                            let msg = ClientToServerMsg::PlayPause;
                            InterProcessCommunication::send_ipc_message(msg, &mut writer).await.context("Could not send IPC message")?;
                        },
                        ClientInputAction::Skip => {
                            let msg = ClientToServerMsg::Skip;
                            InterProcessCommunication::send_ipc_message(msg, &mut writer).await.context("Could not send IPC message")?;
                        },
                    }
                }
            }
        };

        yield_now().await;
    }
}
