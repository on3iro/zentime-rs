use crate::events::ViewState;
use crossterm::terminal::enable_raw_mode;
use std::io::Stdout;
use std::sync::mpsc::Receiver;
use std::thread;
use tui::backend::CrosstermBackend;
use tui::layout::Alignment;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::Color;
use tui::style::Style;
use tui::text::Span;
use tui::text::Spans;
use tui::widgets::Borders;
use tui::widgets::Paragraph;
use tui::widgets::{Block, Tabs};
use tui::Terminal;

use crate::events::TerminalEvent;
use crate::util::quit;

/// Base layout of the program
fn layout(rect: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Max(4),
                Constraint::Max(3),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(rect)
}

fn inner_layout(rect: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(rect)
}

// TODO
// refactor

fn timer_view(terminal: &mut Terminal<CrosstermBackend<Stdout>>, timer_state: ViewState) {
    terminal
        .draw(|frame| {
            let rect = frame.size();
            let layout = layout(rect);
            let inner_layout = inner_layout(layout[0]);

            let keybindings = vec!["[Q]uit", "Space: Play/Pause"];
            let keybinding_spans = keybindings
                .iter()
                .map(|key| {
                    Spans::from(vec![Span::styled(
                        *key,
                        Style::default().fg(Color::DarkGray),
                    )])
                })
                .collect();
            let key_tabs = Tabs::new(keybinding_spans).block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::DarkGray)),
            );

            frame.render_widget(key_tabs, layout[1]);

            let rounds = format!("Round: {}", timer_state.round);
            let work_or_break = if timer_state.is_break {
                "Break"
            } else {
                "Focus"
            };

            let info_text = vec![
                Spans::from(Span::styled(work_or_break, Style::default().fg(Color::Red))),
                Spans::from(vec![Span::styled(rounds, Style::default().fg(Color::Gray))]),
            ];

            let info = Paragraph::new(info_text)
                .block(Block::default().title("zentime").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Left);

            let timer = Paragraph::new(timer_state.time)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::Cyan))
                .alignment(Alignment::Center);

            frame.render_widget(info, inner_layout[0]);
            frame.render_widget(timer, inner_layout[1])
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
                timer_view(&mut terminal, state);
            }
            Ok(TerminalEvent::Quit) => {
                quit(&mut terminal, Some("Cya!"));
            }
            _ => {}
        }
    })
}
