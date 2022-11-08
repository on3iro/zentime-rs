use crate::config::create_config;
use crate::config::Config;
use crate::events::InputEvent;
use crate::events::TerminalEvent;
use crate::input::poll_input_thread;
use crate::state::PomodoroTimer;
use crate::view::render_thread;
use crossterm::event::Event;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

pub fn run(config_path: &str) {
    let config: Config = create_config(config_path)
        .extract()
        .expect("Could not create config");

    let (input_worker_sender, input_worker_receiver): (
        Sender<InputEvent<Event>>,
        Receiver<InputEvent<Event>>,
    ) = mpsc::channel();
    let (view_sender, view_receiver): (Sender<TerminalEvent>, Receiver<TerminalEvent>) =
        mpsc::channel();

    poll_input_thread(input_worker_sender);
    let render_thread_handle = render_thread(view_receiver);
    PomodoroTimer::new(input_worker_receiver, view_sender, config)
        .init()
        .expect("Could not initialize timer");

    render_thread_handle.join().expect("Could not join threads");
}
