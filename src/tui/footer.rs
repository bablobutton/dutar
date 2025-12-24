use std::time::Duration;

use crate::{Model, controls, utils};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{LineGauge, Paragraph, Widget};
use ratatui::{Frame, symbols};

pub fn render_footer(model: &Model, frame: &mut Frame, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Line 1: song info
            Constraint::Length(1), // Line 2: volume and progress
        ])
        .horizontal_margin(1)
        .split(area);

    render_playing_status(model, frame, rows[0]);
    render_bars(model, frame, rows[1]);
}

fn render_playing_status(model: &Model, frame: &mut Frame, area: Rect) {
    let (title, artist, album) = model
        .queue
        .get_current_song()
        .and_then(|song| song.metadata.clone())
        .map(|meta| (meta.title, meta.artist, meta.album))
        .unwrap_or_else(|| {
            (
                String::from("Unknown"),
                String::from("Unknown"),
                String::from("Unknown"),
            )
        });

    let state_text = match &model.app_state {
        crate::AppState::Player(crate::PlayerState::Playing) => "Playing",
        _ => "Paused",
    };

    let line = Line::from(vec![
        Span::raw(state_text),
        Span::raw(" "),
        Span::styled(title, Style::default().fg(Color::Yellow)),
        Span::raw(" by "),
        Span::styled(artist, Style::default().fg(Color::Yellow)),
        Span::raw(" from "),
        Span::styled(album, Style::default().fg(Color::Yellow)),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

fn render_bars(model: &Model, frame: &mut Frame, area: Rect) {
    let bars = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70), // progress bar
            Constraint::Percentage(30), // volume bar
        ])
        .spacing(1)
        .split(area);

    render_progress_bar(model, frame, bars[0]);
    render_volume_bar(model, frame, bars[1]);
}

fn render_progress_bar(model: &Model, frame: &mut Frame, area: Rect) {
    let current_secs = controls::get_current_duration(model).as_secs();

    let total_secs = controls::get_current_song_total_duration(model)
        .unwrap_or(Duration::ZERO)
        .as_secs();

    let ratio = if total_secs > 0 {
        (current_secs as f64) / (total_secs as f64)
    } else {
        0f64
    };

    let minutes_width_current = utils::count_digits(current_secs / 60).max(2);
    let minutes_width_total = utils::count_digits(total_secs / 60).max(2);
    let label = format!(
        "{:0minutes_width_current$}:{:02} - {:0minutes_width_total$}:{:02}",
        current_secs / 60,
        current_secs % 60,
        total_secs / 60,
        total_secs % 60
    );

    LineGauge::default()
        .ratio(ratio)
        .filled_style(Style::default().fg(Color::Yellow))
        .line_set(symbols::line::THICK)
        .label(label)
        .render(area, frame.buffer_mut());
}

fn render_volume_bar(model: &Model, frame: &mut Frame, area: Rect) {
    let ratio = controls::get_volume(model).min(1.0) as f64;

    LineGauge::default()
        .ratio(ratio)
        .filled_style(Style::default().fg(Color::Yellow))
        .line_set(symbols::line::THICK)
        .render(area, frame.buffer_mut());
}
