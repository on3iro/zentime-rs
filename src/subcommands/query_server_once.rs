use futures::io::BufReader;
use interprocess::local_socket::tokio::LocalSocketStream;
use zentime_rs::ipc::{get_socket_name, ClientToServerMsg};
use zentime_rs::ipc::{InterProcessCommunication, ServerToClientMsg};
use zentime_rs::server::status::{server_status, ServerStatus};

#[tokio::main]
pub async fn query_server_once() {
    // check if server is running -> if not, quit
    if server_status() == ServerStatus::Stopped {
        println!("No zentime server running");
        std::process::exit(0);
    }

    // connect to server
    let socket_name = get_socket_name();
    let connection = match LocalSocketStream::connect(socket_name).await {
        Ok(c) => c,
        Err(error) => panic!("Could not conenct to server: {}", error),
    };

    // receive view once and print it
    let (reader, mut writer) = connection.into_split();
    let mut reader = BufReader::new(reader);

    // TODO handle error
    InterProcessCommunication::send_ipc_message(ClientToServerMsg::Sync, &mut writer)
        .await
        .unwrap();

    // Loop until we receive a timer message from the server and quit afterwards
    // loop {
    let msg_result =
        InterProcessCommunication::recv_ipc_message::<ServerToClientMsg>(&mut reader).await;

    println!("Received result: {:?}", msg_result);

    if let Ok(ServerToClientMsg::Timer(state)) = msg_result {
        println!(
            "{} {} {}",
            state.round,
            state.time,
            if state.is_break { "Break" } else { "Focus" }
        );
        std::process::exit(0);
    }
    // }
}
