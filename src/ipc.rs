use anyhow::{bail, Context};
use futures::io::BufReader;
use futures::{AsyncReadExt, AsyncWriteExt};
use interprocess::local_socket::tokio::{OwnedReadHalf, OwnedWriteHalf};
use interprocess::local_socket::NameTypeSupport;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use zentime_rs_timer::timer::ViewState;

const DEFAULT_SOCKET_PATH: &str = "/tmp/zentime.sock";
const DEFAULT_SOCKET_NAMESPACE: &str = "@zentime.sock";

pub fn get_socket_name() -> &'static str {
    // This scoping trick allows us to nicely contain the import inside the `match`, so that if
    // any imports of variants named `Both` happen down the line, they won't collide with the
    // enum we're working with here. Maybe someone should make a macro for this.
    use NameTypeSupport::*;
    match NameTypeSupport::query() {
        OnlyPaths => DEFAULT_SOCKET_PATH,
        OnlyNamespaced | Both => DEFAULT_SOCKET_NAMESPACE,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerToClientMsg {
    Timer(ViewState),
    Quit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientToServerMsg {
    Quit,
    PlayPause,
    Skip,
}

pub struct InterProcessCommunication {}

impl InterProcessCommunication {
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