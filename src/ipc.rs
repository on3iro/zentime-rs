//! Utilities to handle zentime inter-process-communication

use anyhow::{bail, Context};
use futures::io::BufReader;
use futures::{AsyncReadExt, AsyncWriteExt};
use interprocess::local_socket::tokio::{OwnedReadHalf, OwnedWriteHalf};
use interprocess::local_socket::NameTypeSupport;
use serde::{Deserialize, Serialize};
use zentime_rs_timer::pomodoro_timer::ViewState;
use std::fmt::Debug;

const DEFAULT_SOCKET_PATH: &str = "/tmp/zentime.sock";
const DEFAULT_SOCKET_NAMESPACE: &str = "@zentime.sock";
const DEBUG_SOCKET_PATH: &str = "/tmp/zentime_debug.sock";
const DEBUG_SOCKET_NAMESPACE: &str = "@zentime_debug.sock";

/// Get zentime socket name over which server and clients may connect
pub fn get_socket_name() -> &'static str {
    // This scoping trick allows us to nicely contain the import inside the `match`, so that if
    // any imports of variants named `Both` happen down the line, they won't collide with the
    // enum we're working with here. Maybe someone should make a macro for this.
    use NameTypeSupport::*;

    if cfg!(debug_assertions) {
        match NameTypeSupport::query() {
            OnlyPaths => DEBUG_SOCKET_PATH,
            OnlyNamespaced | Both => DEBUG_SOCKET_NAMESPACE,
        }
    } else {
        match NameTypeSupport::query() {
            OnlyPaths => DEFAULT_SOCKET_PATH,
            OnlyNamespaced | Both => DEFAULT_SOCKET_NAMESPACE,
        }
    }
}

/// A message from the zentime server to the client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerToClientMsg {
    /// Aggregated state of the timer which a client can display
    Timer(ViewState),
}

/// A message from a client to the zentime server
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ClientToServerMsg {
    /// Command the server to shutdown and close all connections
    Quit,

    /// Detach from the server
    Detach,

    /// Command the server to Play/Pause the timer
    PlayPause,

    /// Command the server to skip to the next interval
    Skip,

    /// Command the server to reset the timer back to interval 1
    Reset,

    /// Currently it's necessary for a client to write at least once to a socket
    /// connection to synchronize with the server.
    /// For one-shot zentime commands we therefore use this sync msg to synchronize
    /// with the server.
    Sync,
}

/// Service handling communication between processes over the zentime socket.
/// Multiple clients may exist alongside a single (usually daemonized) zentime server instance.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct InterProcessCommunication {}

impl InterProcessCommunication {
    /// Writes a message to the zentime socket.
    /// The message is encoded via [rmp_serde::encode] which uses [Messagepack](https://msgpack.org/) to encode type information.
    pub async fn send_ipc_message<M>(msg: M, writer: &mut OwnedWriteHalf) -> anyhow::Result<()>
    where
        M: Serialize + for<'a> Deserialize<'a> + Debug,
    {
        let encoded_msg =
            rmp_serde::encode::to_vec::<M>(&msg).context(format!("Could not encode {:?}", msg))?;
        let msg_length =
            u32::try_from(encoded_msg.len()).context("Could not cast msg length to u32")?;
        let msg_length = msg_length.to_le_bytes();

        // Write msg with length to the stream (see [Self::recv_ipc_message] for why this is needed)
        writer
            .write_all(&msg_length)
            .await
            .context("Could not write message length to stream")?;

        // Write actual msg to the stream
        writer
            .write_all(&encoded_msg)
            .await
            .context(format!("Could not write {:?} to stream", msg))?;

        Ok(())
    }

    /// Writes a message to the zentime socket.
    /// The message is decoded via [rmp_serde::decode] which uses [Messagepack](https://msgpack.org/) to decode type information.
    pub async fn recv_ipc_message<M>(reader: &mut BufReader<OwnedReadHalf>) -> anyhow::Result<M>
    where
        M: Serialize + for<'a> Deserialize<'a> + Debug,
    {
        // Read message length, so that we can make an exact read of the actual message afterwards
        let mut buffer = [0_u8; 4];

        reader
            .read_exact(&mut buffer)
            .await
            .context("Could not read msg length")?;
        let msg_length = u32::from_le_bytes(buffer);

        let mut buffer = [0_u8; 1024];

        // Read message of previously determined length, decode and return it
        if let Err(error) = reader
            .read_exact(
                &mut buffer[0..usize::try_from(msg_length)
                    .context("Could not convert msg length to usize")?],
            )
            .await
        {
            match error.kind() {
                std::io::ErrorKind::UnexpectedEof => {
                    bail!("Buffer slice has not been filled entirely: {:?}", error)
                }
                _ => bail!("Could not read into buffer: {:?}", error),
            }
        };

        match rmp_serde::from_slice::<M>(
            &buffer[0..usize::try_from(msg_length)
                .context("Could not convert msg length to usize")?],
        ) {
            Ok(msg) => Ok(msg),
            Err(error) => bail!("Could not decode msg: {:?}", error),
        }
    }
}
