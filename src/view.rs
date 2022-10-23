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

pub fn render_thread(
    mut terminal: Terminal<CrosstermBackend<Stdout>>,
    view_receiver: Receiver<TerminalEvent>,
) {
    thread::spawn(move || loop {
        match view_receiver.recv() {
            Ok(TerminalEvent::View(string)) => {
                timer_view(&mut terminal, &string);
            }
            Ok(TerminalEvent::Quit) => {
                quit(&mut terminal, Some("Cya!"));
            }
            _ => {}
        }
    });
}
