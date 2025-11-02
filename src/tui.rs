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
use ratatui::widgets::Gauge;
use ratatui::{
    Frame,
    style::Stylize,
    widgets::{Block, Borders, List, ListDirection, Paragraph},
};

pub fn render(model: &Model, frame: &mut Frame) {
    // chunks divide main UI into regions into which we'll put things
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(2),
            Constraint::Fill(1),
            Constraint::Percentage(10), // Progress bar
        ])
        .split(frame.area());

    render_play_text(model, frame, chunks[0]);
    render_hints(frame, chunks[1]);

    // render popus (they don't need chunks)
    render_bar_popup(model, frame);
    render_hints_popup(model, frame);
    render_progress(model, frame, chunks[2]);
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

fn render_progress(model: &Model, frame: &mut Frame, chunk: Rect) {
    let sink = &model.audio.sink;
    let current_pos = sink.get_pos();
    let secs = current_pos.as_secs();

    let total_duration = model
        .queue
        .get_current_song()
        .and_then(|song| song.metadata.as_ref())
        .map(|meta| meta.duration)
        .unwrap_or(0);

    let ratio = if total_duration > 0 {
        (secs as f64) / (total_duration as f64)
    } else {
        0.0
    };
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Progress"))
        .ratio(ratio)
        .style(Style::default().fg(Color::Green))
        .label(format!(
            "{:02}:{:02} / {:02}:{:02}",
            secs / 60,
            secs % 60,
            total_duration / 60,
            total_duration % 60
        ));

    frame.render_widget(gauge, chunk);
}
