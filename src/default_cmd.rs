use crate::client::input::TerminalInputThread;

use crate::client::view::TerminalRenderer;
use crate::config::{create_config, Config};

use crate::server;
use std::sync::mpsc;

// TODO
// differentiate between server start and attach

pub fn default_cmd(config_path: &str) {
    let config: Config = create_config(config_path)
        .extract()
        .expect("Could not create config");

    let (terminal_input_sender, terminal_input_receiver) = mpsc::channel();
    let (view_sender, view_receiver) = mpsc::channel();

    TerminalInputThread::spawn(terminal_input_sender);

    let interface_type = config.view.interface.clone();
    let render_thread_handle = TerminalRenderer::spawn(view_receiver, interface_type);

    server::start(config, view_sender, terminal_input_receiver);

    render_thread_handle.join().expect("Could not join threads");
}
