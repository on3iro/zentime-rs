use crate::events::InputEvent;
use crate::events::TerminalEvent;
use crate::input::poll_input_thread;
use crate::state::PomodoroTimer;
use crate::view::render_thread;
use crossterm::event::Event;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::time::Duration;

pub fn run() {
    let (input_worker_tx, input_worker_rx): (
        Sender<InputEvent<Event>>,
        Receiver<InputEvent<Event>>,
    ) = mpsc::channel();
    let (view_sender, view_receiver): (Sender<TerminalEvent>, Receiver<TerminalEvent>) =
        mpsc::channel();

    poll_input_thread(input_worker_tx);
    render_thread(view_receiver);
    PomodoroTimer::new(input_worker_rx, view_sender, Duration::from_secs(20)).run();
}
