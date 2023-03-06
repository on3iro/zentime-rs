use anyhow::Context;
use zentime_rs_timer::pomodoro_timer::ViewState;

use std::io::Stdout;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs},
    Terminal as TuiTerminal,
};

/// Default interface
pub fn render(
    terminal: &mut TuiTerminal<CrosstermBackend<Stdout>>,
    timer_state: ViewState,
) -> anyhow::Result<()> {
    terminal
        .draw(|frame| {
            let rect = frame.size();
            let layout = layout(rect);

            // Rendered at the bottom
            let key_tabs = key_binding_info();
            frame.render_widget(key_tabs, layout[1]);

            // Top layou
            let inner_layout = inner_layout(layout[0]);

            // Rendered to the left
            let timer_info = timer_info(&timer_state);
            frame.render_widget(timer_info, inner_layout[0]);

            // Rendered to the right
            let timer = timer(&timer_state.time);
            frame.render_widget(timer, inner_layout[1])
        })
        .context("Could not render to terminal")?;
    Ok(())
}

/// Base layout of the default interface
/// ┌───────────────────────────────────────────────┐
/// │                                               │
/// │                                               │
/// │                    A                          │
/// │                                               │
/// └───────────────────────────────────────────────┘
/// ┌───────────────────────────────────────────────┐
/// │                    B                          │
/// └───────────────────────────────────────────────┘
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

/// Inner layout of the default interface rendered into the base layout part A
/// ┌──────────────────┐ ┌────────────────────────────────┐
/// │                  │ │                                │
/// │        A         │ │                B               │
/// │                  │ │                                │
/// │                  │ │                                │
/// └──────────────────┘ └────────────────────────────────┘
fn inner_layout(rect: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(rect)
}

/// Keyboard shortcuts of the default interface
/// ┌─────────────────────────────────────────────────────────┐
/// │ [Q]uit │ [D]etach │ [S]kip │ Space: Play/Pause          │
/// └─────────────────────────────────────────────────────────┘
fn key_binding_info() -> Tabs<'static> {
    let keybindings = vec!["[Q]uit", "[D]etach", "[S]kip", "Space: Play/Pause"];
    let keybinding_spans = keybindings
        .iter()
        .map(|key| {
            Spans::from(vec![Span::styled(
                *key,
                Style::default().fg(Color::DarkGray),
            )])
        })
        .collect();

    Tabs::new(keybinding_spans).block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::DarkGray)),
    )
}

/// Timer information of the default interface (interval/round number, break/focus)
fn timer_info(timer_state: &ViewState) -> Paragraph {
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

    Paragraph::new(info_text)
        .block(Block::default().title("zentime").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
}

/// Timer of the default interface
fn timer(time: &str) -> Paragraph {
    Paragraph::new(time)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
}
