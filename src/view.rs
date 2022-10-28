use crossterm::terminal::enable_raw_mode;
use std::io::Stdout;
use std::sync::mpsc::Receiver;
use std::thread;
use tui::backend::CrosstermBackend;
use tui::layout::Alignment;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::Color;
use tui::style::Style;
use tui::widgets::Block;
use tui::widgets::Borders;
use tui::widgets::Paragraph;
use tui::Terminal;

use crate::events::TerminalEvent;
use crate::util::quit;

/// Base layout of the program
fn layout(rect: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
        .split(rect)
}

fn timer_view(terminal: &mut Terminal<CrosstermBackend<Stdout>>, timer: &str) {
    terminal
        .draw(|frame| {
            let rect = frame.size();
            let layout = layout(rect);
            let timer = Paragraph::new(timer)
                .block(Block::default().title("zentime").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center);
            frame.render_widget(timer, layout[0])
        })
        .unwrap();
}

pub fn render_thread(view_receiver: Receiver<TerminalEvent>) -> thread::JoinHandle<()> {
    enable_raw_mode().expect("Can run in raw mode");
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("Terminal could be created");
    terminal.clear().expect("Terminal could be cleared");

    thread::spawn(move || loop {
        match view_receiver.recv() {
            Ok(TerminalEvent::View(state)) => {
                timer_view(&mut terminal, &state.time);
            }
            Ok(TerminalEvent::Quit) => {
                quit(&mut terminal, Some("Cya!"));
            }
            _ => {}
        }
    })
}
