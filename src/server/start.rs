use crate::config::Config;
use crate::ipc::{
    get_socket_name, ClientToServerMsg, InterProcessCommunication, ServerToClientMsg,
};
use crate::server::notification::dispatch_notification;
use crate::server::timer_output::TimerOutputAction;
use anyhow::Context;
use crossbeam::channel::{unbounded, Sender};
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

pub async fn start(config: Config) -> anyhow::Result<Option<JoinHandle<()>>> {
    let socket_name = get_socket_name();

    // TODO
    // * handle this more robustly
    // * add a way to connect to a different server during development (e.g. by specifying the
    // socket address)
    let system = System::new_all();

    let socket_file_already_exists = metadata(socket_name).await.is_ok();
    let current_is_only_instance = system.processes_by_name("zentime").count() == 1;

    if socket_file_already_exists && !current_is_only_instance {
        // Apparently a server is already running and we don't need to do anything
        return Ok(None);
    }

    if socket_file_already_exists && current_is_only_instance {
        // We have a dangling socket file without an attached server process.
        // In that case we simply remove the file and start a new server process
        remove_file(socket_name)
            .await
            .context("Could not remove existing socket file")?
    };

    Ok(Some(thread::spawn(move || {
        // TODO
        // * daemonize

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
                match timer_input_receiver.recv_timeout(Duration::from_secs(1)) {
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

        // Spawn new parallel asynchronous tasks onto the Tokio runtime
        // and hand the connection over to them so that multiple clients
        // could be processed simultaneously in a lightweight fashion.
        tokio::spawn(async move {
            if let Err(error) = handle_conn(connection, input_tx, output_rx).await {
                panic!("Could not handle connection: {}", error);
            };
        });
    }
}

// Describe the things we do when we've got a connection ready.
async fn handle_conn(
    conn: LocalSocketStream,
    timer_input_sender: Sender<TimerInputAction>,
    mut timer_output_receiver: BroadcastReceiver<TimerOutputAction>,
) -> anyhow::Result<()> {
    // Split the connection into two halves to process
    // received and sent data concurrently.
    let (reader, mut writer) = conn.into_split();
    let mut reader = BufReader::new(reader);

    loop {
        select! {
            msg = InterProcessCommunication::recv_ipc_message::<ClientToServerMsg>(&mut reader) => {
                let msg = msg.context("Could not receive message from socket")?;

                match msg {
                    ClientToServerMsg::Quit => todo!(),
                    ClientToServerMsg::PlayPause => {
                        timer_input_sender.send(TimerInputAction::PlayPause).context("Could not send Play/Pause to timer")?;
                    },
                    ClientToServerMsg::Skip => {
                        timer_input_sender.send(TimerInputAction::Skip).context("Could not send Skip to timer")?;
                    },
                }
            },
            value = timer_output_receiver.recv() => {
                let TimerOutputAction::Timer(state) = value.context("Could not receive output from timer")?;
                let msg = ServerToClientMsg::Timer(state);
                InterProcessCommunication::send_ipc_message(msg, &mut writer).await.context("Could not send IPC message from server to client")?;
            }
        }

        yield_now().await;
    }
}
