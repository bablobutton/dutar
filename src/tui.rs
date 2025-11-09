mod bar;
mod hints;

use crate::Model;
use bar::render_bar_popup;
use hints::render_hints_popup;
use ratatui::layout::Rect;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Gauge;
use ratatui::{
    Frame,
    style::Stylize,
    text::Span,
    widgets::{Block, Borders, Cell, List, ListDirection, Row, Table},
};

pub fn render(model: &Model, frame: &mut Frame) {
    // chunks divide main UI into regions into which we'll put things
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(2),        // Play text
            Constraint::Fill(1),        // Hints
            Constraint::Percentage(7),  // Volume
            Constraint::Percentage(10), // Progress bar
        ])
        .split(frame.area());

    render_play_text(model, frame, chunks[0]);
    render_hints(frame, chunks[1]);
    render_volume(model, frame, chunks[2]);
    render_progress(model, frame, chunks[3]);

    // render popus (they don't need chunks)
    render_bar_popup(model, frame);
    render_hints_popup(model, frame);
}

fn render_play_text(model: &Model, frame: &mut Frame, chunk: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Now Playing")
        .style(Style::default());

    let current_song = model.queue.get_current_song();

    let rows: Vec<Row> = if let Some(song) = current_song {
        if let Some(meta) = &song.metadata {
            vec![Row::new(vec![
                Cell::from(meta.artist.clone()),
                Cell::from(meta.title.clone()),
                Cell::from(meta.album.clone()),
                Cell::from(format!(
                    "{:02}:{:02}",
                    meta.duration / 60,
                    meta.duration % 60
                )),
            ])]
        } else {
            vec![Row::new(vec![Cell::from("No metadata found")])]
        }
    } else {
        vec![Row::new(vec![Cell::from("No song playing")])]
    };

    let header = Row::new(vec![
        Cell::from(Span::styled("Artist", Style::default().fg(Color::Yellow))),
        Cell::from(Span::styled("Title", Style::default().fg(Color::Yellow))),
        Cell::from(Span::styled("Album", Style::default().fg(Color::Yellow))),
        Cell::from(Span::styled("Duration", Style::default().fg(Color::Yellow))),
    ])
    .style(Style::default().fg(Color::White));

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(35),
            Constraint::Percentage(30),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(block)
    .style(Style::default().fg(Color::Green));

    frame.render_widget(table, chunk);
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

fn render_volume(model: &Model, frame: &mut Frame, chunk: Rect) {
    let sink = &model.audio.sink;
    let max_vol = 1.0;
    let curr_vol = sink.volume();

    let ratio = (curr_vol as f64) / (max_vol as f64);
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Volume"))
        .ratio(ratio)
        .style(Style::default().fg(Color::Green))
        .label(format!("{curr_vol:.2}",));

    frame.render_widget(gauge, chunk);
}
