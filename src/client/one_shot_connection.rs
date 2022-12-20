//! Creates a connection for single reads/writes from/to the server
use crate::ipc::get_socket_name;
use crate::server::status::server_status;
use crate::server::status::ServerStatus;
use interprocess::local_socket::tokio::LocalSocketStream;
use interprocess::local_socket::tokio::OwnedReadHalf;
use interprocess::local_socket::tokio::OwnedWriteHalf;

/// Creates a connection to the zentime server (if one is running) and returns
/// a tuple of [OwnedReadHalf] and an [OwnedWriteHalf].
/// If no server is running the current process is terminated.
///
/// NOTE:
/// If you just want to read from the server, you still need to write [ClientToServerMsg::Sync]
/// first and make sure that the writer isn't being dropped before you read. Otherwise you will
/// encount EOF on the socket!
///
/// NOTE:
/// Also make sure to send a detach message to the server as well
pub async fn one_shot_connection() -> anyhow::Result<(OwnedReadHalf, OwnedWriteHalf)> {
    // check if server is running -> if not, quit
    if server_status() == ServerStatus::Stopped {
        println!("No zentime server running");
        std::process::exit(0);
    }

    // connect to server
    let socket_name = get_socket_name();
    let connection = LocalSocketStream::connect(socket_name).await?;

    Ok(connection.into_split())
}
