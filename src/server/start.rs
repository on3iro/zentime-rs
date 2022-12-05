use crate::config::Config;
use crate::ipc::{
    get_socket_name, ClientToServerMsg, InterProcessCommunication, ServerToClientMsg,
};
use crate::server::notification::dispatch_notification;
use crate::server::timer_output::TimerOutputAction;
use anyhow::Context;
use crossbeam::channel::{unbounded, Sender};
use daemonize::Daemonize;
use interprocess::local_socket::tokio::OwnedWriteHalf;
use std::fs::File;
use std::process;
use sysinfo::ProcessExt;
use tokio::task::yield_now;

use std::sync::Arc;
use std::thread::{self, JoinHandle};
use sysinfo::{System, SystemExt};
use tokio::select;
use tokio::sync::{self, broadcast::Receiver as BroadcastReceiver};

use futures::io::BufReader;
use interprocess::local_socket::tokio::{LocalSocketListener, LocalSocketStream};

use std::time::Duration;
use tokio::fs::{metadata, remove_file};

use zentime_rs_timer::{Timer, TimerInputAction};

// TODO
// add logging

pub async fn start(config: Config) -> anyhow::Result<Option<JoinHandle<()>>> {
    let socket_name = get_socket_name();

    // TODO
    // * add a way to connect to a different server during development (e.g. by specifying the
    // socket address - or making use of the executable path)
    let system = System::new_all();

    let socket_file_already_exists = metadata(socket_name).await.is_ok();
    let zentime_process_instances = system.processes_by_name("zentime");

    // WHY:
    // We identify a server process by its comman (e.g. "zentime server start").
    // This process itself will be one instance, so if we have two instances there is already
    // another server process running and we don't have to start this one and can exit early.
    let server_is_already_running = zentime_process_instances
        .filter(|p| p.cmd().contains(&String::from("server")))
        .count()
        == 2;

    if socket_file_already_exists && server_is_already_running {
        // Apparently a server is already running and we don't need to do anything
        return Ok(None);
    }

    if socket_file_already_exists {
        // We have a dangling socket file without an attached server process.
        // In that case we simply remove the file and start a new server process
        remove_file(socket_name)
            .await
            .context("Could not remove existing socket file")?
    };

    Ok(Some(thread::spawn(move || {
        let stdout = File::create("/tmp/zentime.d.out").unwrap();
        let stderr = File::create("/tmp/zentime.d.err").unwrap();

        let daemonize = Daemonize::new()
            .stdout(stdout) // Redirect stdout to `/tmp/daemon.out`.
            .stderr(stderr); // Redirect stderr to `/tmp/daemon.err`.

        if let Err(error) = daemonize.start() {
            panic!("Could not daemonize server process: {}", error);
        };

        if let Err(error) = listen(config, socket_name) {
            panic!("Could not start server listener: {}", error);
        };
    })))
}

#[tokio::main]
async fn listen(config: Config, socket_name: &str) -> anyhow::Result<()> {
    let listener =
        LocalSocketListener::bind(socket_name).context("Could not bind to local socket")?;

    let (timer_input_sender, timer_input_receiver) = unbounded();
    let (timer_output_sender, _timer_output_receiver) = sync::broadcast::channel(24);

    let timer_output_sender = Arc::new(timer_output_sender.clone());
    // Arc clone to create a reference to our sender which can be consumed by the
    // timer thread. This is necessary because we need a reference to this sender later on
    // to continuously subscribe to it on incoming client connections
    let timer_out_tx = timer_output_sender.clone();

    let (connection_synchronization_sender, _connection_synchronization_receiver) =
        sync::broadcast::channel(24);
    let connection_synchronization_sender = Arc::new(connection_synchronization_sender.clone());

    // Timer running on its own thread so that it does not block our async runtime
    thread::spawn(move || {
        Timer::new(
            config.timers,
            Box::new(move |_, msg| {
                // We simply discard errors here for now...
                dispatch_notification(config.notifications, msg).ok();
            }),
            Box::new(move |view_state| {
                // Update the view
                timer_out_tx.send(TimerOutputAction::Timer(view_state)).ok();

                // Handle app actions and hand them to the timer caller
                match timer_input_receiver.recv_timeout(Duration::from_millis(100)) {
                    Ok(action) => Some(action),
                    _ => Some(TimerInputAction::None),
                }
            }),
        )
        .init()
    });

    // Set up our loop boilerplate that processes our incoming connections.
    loop {
        let connection = listener
            .accept()
            .await
            .context("There was an error with an incoming connection")?;

        let input_tx = timer_input_sender.clone();
        let output_rx = timer_output_sender.subscribe();
        let connection_sync_tx = connection_synchronization_sender.clone();
        let connection_sync_rx = connection_synchronization_sender.subscribe();

        // Spawn new parallel asynchronous tasks onto the Tokio runtime
        // and hand the connection over to them so that multiple clients
        // could be processed simultaneously in a lightweight fashion.
        tokio::spawn(async move {
            if let Err(error) = handle_conn(
                connection,
                input_tx,
                output_rx,
                connection_sync_tx,
                connection_sync_rx,
            )
            .await
            {
                panic!("Could not handle connection: {}", error);
            };
        });
    }
}

#[derive(Clone)]
enum ConnectionSynchronizationAction {
    Quit,
}

// Describe the things we do when we've got a connection ready.
async fn handle_conn(
    conn: LocalSocketStream,
    timer_input_sender: Sender<TimerInputAction>,
    mut timer_output_receiver: BroadcastReceiver<TimerOutputAction>,
    connection_sync_tx: std::sync::Arc<
        tokio::sync::broadcast::Sender<ConnectionSynchronizationAction>,
    >,
    mut connection_sync_rx: BroadcastReceiver<ConnectionSynchronizationAction>,
) -> anyhow::Result<()> {
    // Split the connection into two halves to process
    // received and sent data concurrently.
    let (reader, mut writer) = conn.into_split();
    let mut reader = BufReader::new(reader);

    loop {
        select! {
            // Actions which are received by all connections to act upon
            value = connection_sync_rx.recv() => {
                let action = value.context("Could not receive synchronization action")?;
                handle_connection_synchronization(action, &mut writer).await.context("Could not handle connection synchronization action")?;
            },
            msg = InterProcessCommunication::recv_ipc_message::<ClientToServerMsg>(&mut reader) => {
                let msg = msg.context("Could not receive message from socket")?;
                handle_client_to_server_msg(msg, &connection_sync_tx, &timer_input_sender).await.context("Could not handle client to server message")?;
            },
            value = timer_output_receiver.recv() => {
                let action = value.context("Could not receive output from timer")?;
                handle_timer_output_action(action, &mut writer).await.context("Couuld not handle timer output action")?;
            }
        }

        yield_now().await;
    }
}

async fn handle_connection_synchronization(
    action: ConnectionSynchronizationAction,
    writer: &mut OwnedWriteHalf,
) -> anyhow::Result<()> {
    match action {
        ConnectionSynchronizationAction::Quit => {
            let msg = ServerToClientMsg::Quit;
            InterProcessCommunication::send_ipc_message(msg, writer)
                .await
                .context("Could not send IPC message from server to client")?;
            process::exit(0);
        }
    }
}

async fn handle_client_to_server_msg(
    msg: ClientToServerMsg,
    connection_sync_tx: &std::sync::Arc<
        tokio::sync::broadcast::Sender<ConnectionSynchronizationAction>,
    >,
    timer_input_sender: &Sender<TimerInputAction>,
) -> anyhow::Result<()> {
    match msg {
        ClientToServerMsg::Quit => {
            // TODO handle error
            connection_sync_tx
                .send(ConnectionSynchronizationAction::Quit)
                .ok();
        }
        ClientToServerMsg::PlayPause => {
            timer_input_sender
                .send(TimerInputAction::PlayPause)
                .context("Could not send Play/Pause to timer")?;
        }
        ClientToServerMsg::Skip => {
            timer_input_sender
                .send(TimerInputAction::Skip)
                .context("Could not send Skip to timer")?;
        }
    }

    Ok(())
}

async fn handle_timer_output_action(
    action: TimerOutputAction,
    writer: &mut OwnedWriteHalf,
) -> anyhow::Result<()> {
    let TimerOutputAction::Timer(state) = action;
    let msg = ServerToClientMsg::Timer(state);
    InterProcessCommunication::send_ipc_message(msg, writer)
        .await
        .context("Could not send IPC message from server to client")?;

    Ok(())
}
