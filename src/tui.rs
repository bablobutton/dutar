use crate::{Model, PlayerState, State};
use ratatui::layout::Rect;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::{Line, Text};
use ratatui::{
    Frame,
    style::Stylize,
    widgets::{Block, Borders, List, ListDirection, Paragraph},
};
use std::rc::Rc;

pub fn render_state(model: &Model, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(2), Constraint::Fill(1)])
        .split(frame.area());

    render_play_text(model, frame, &chunks);
    render_hints(frame, &chunks);
}

fn render_play_text(model: &Model, frame: &mut Frame, chunks: &Rc<[Rect]>) {
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let mut lines = vec![Line::styled(
        format!("State: {:?}", model.state),
        Style::default().fg(Color::Green),
    )];

    let current_song = model.queue.get_current_song();
    let current_song_str = match current_song {
        Some(song) => format!("{:#?}", song),
        None => "None".to_string(),
    };

    lines.extend(
        current_song_str
            .lines()
            .map(|line| Line::styled(line.to_string(), Style::default().fg(Color::Green))),
    );

    let paragraph = Paragraph::new(Text::from(lines)).block(block);
    frame.render_widget(paragraph, chunks[0]);
}

fn render_hints(frame: &mut Frame, chunks: &Rc<[Rect]>) {
    let block = Block::bordered().title("Hints");

    let items = [
        "k - play/pause",
        "l/j - forward/backward 5s",
        "n/p - next/previous",
        "q - quit",
    ];
    let list = List::new(items)
        .block(block)
        .style(Style::new().white())
        .highlight_style(Style::new().italic())
        .direction(ListDirection::TopToBottom);

    frame.render_widget(list, chunks[1]);
}
