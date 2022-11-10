use crate::config::create_config;
use crate::config::Config;

use crate::events::TerminalEvent;
use crate::input::TerminalInputThread;
use crate::timer::PomodoroTimer;
use crate::view::render_thread;
use crate::AppAction;

use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

pub fn run(config_path: &str) {
    let config: Config = create_config(config_path)
        .extract()
        .expect("Could not create config");

    let (terminal_input_sender, terminal_input_receiver): (Sender<AppAction>, Receiver<AppAction>) =
        mpsc::channel();
    let (view_sender, view_receiver): (Sender<TerminalEvent>, Receiver<TerminalEvent>) =
        mpsc::channel();

    TerminalInputThread::spawn(terminal_input_sender);
    let render_thread_handle = render_thread(view_receiver);

    PomodoroTimer::new(terminal_input_receiver, view_sender, config)
        .init()
        .expect("Could not initialize timer");

    render_thread_handle.join().expect("Could not join threads");
}
