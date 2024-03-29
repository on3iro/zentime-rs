use futures::io::BufReader;
use zentime_rs::client::one_shot_connection::one_shot_connection;
use zentime_rs::ipc::ClientToServerMsg;
use zentime_rs::ipc::InterProcessCommunication;
use zentime_rs::ipc::ServerToClientMsg;

#[tokio::main]
pub async fn reset_timer(silent: bool) {
    let (reader, mut writer) = match one_shot_connection().await {
        Ok(c) => c,
        Err(error) => panic!("Could not conenct to server: {}", error),
    };

    let mut reader = BufReader::new(reader);

    if let Err(err) =
        InterProcessCommunication::send_ipc_message(ClientToServerMsg::Reset, &mut writer).await
    {
        panic!("Could not send to the server: {}", err)
    };

    let msg_result =
        InterProcessCommunication::recv_ipc_message::<ServerToClientMsg>(&mut reader).await;

    if !silent {
        if let Ok(ServerToClientMsg::Timer(state)) = msg_result {
            println!(
                "{} {} {}",
                state.round,
                state.time,
                if state.is_break { "Break" } else { "Focus" }
            );
        }
    }

    InterProcessCommunication::send_ipc_message(ClientToServerMsg::Detach, &mut writer)
        .await
        .ok();
}
