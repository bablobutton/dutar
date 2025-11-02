mod bar;
mod hints;

use crate::Model;
use bar::render_bar_popup;
use hints::render_hints_popup;
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

pub fn render(model: &Model, frame: &mut Frame) {
    // chunks divide main UI into regions into which we'll put things
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(2), Constraint::Fill(1)])
        .split(frame.area());

    render_play_text(model, frame, chunks[0]);
    render_hints(frame, chunks[1]);

    // render popus (they don't need chunks)
    render_bar_popup(model, frame);
    render_hints_popup(model, frame);
}

fn render_play_text(model: &Model, frame: &mut Frame, chunk: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let mut lines = vec![Line::styled(
        format!("State: {:?}", model.app_state),
        Style::default().fg(Color::Green),
    )];

    let current_song = model.queue.get_current_song();
    let current_song_str = match current_song {
        Some(song) => format!("{song:#?}"),
        None => "None".to_string(),
    };

    lines.extend(
        current_song_str
            .lines()
            .map(|line| Line::styled(line.to_string(), Style::default().fg(Color::Green))),
    );

    let paragraph = Paragraph::new(Text::from(lines)).block(block);
    frame.render_widget(paragraph, chunk);
}

fn render_hints(frame: &mut Frame, chunk: Rect) {
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

    frame.render_widget(list, chunk);
}
