use crate::config::create_config;
use crate::input::TerminalInputThread;
use crate::timer::Timer;
use crate::view::TerminalRenderThread;

use std::sync::mpsc;

pub fn run(config_path: &str) {
    let config = create_config(config_path)
        .extract()
        .expect("Could not create config");

    let (terminal_input_sender, terminal_input_receiver) = mpsc::channel();
    let (view_sender, view_receiver) = mpsc::channel();

    TerminalInputThread::spawn(terminal_input_sender);
    let render_thread_handle = TerminalRenderThread::spawn(view_receiver);

    Timer::new(terminal_input_receiver, view_sender, config)
        .init()
        .expect("Could not initialize timer");

    render_thread_handle.join().expect("Could not join threads");
}
