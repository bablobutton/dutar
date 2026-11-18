use crate::{AppState, Model, PlayerState, controls};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{LineGauge, Paragraph, Widget};
use ratatui::{Frame, symbols};

pub fn render_footer(model: &Model, frame: &mut Frame, area: Rect) {
    let [safe_exit_area, status_area, bars_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Line 1: Gap, would be replaced with Safe Exit
            Constraint::Length(1), // Line 2: song info
            Constraint::Length(1), // Line 3: volume and progress
        ])
        .horizontal_margin(1)
        .areas(area);

    if model.safe_exit_active() {
        render_safe_exit(frame, safe_exit_area);
    }

    render_playing_status(model, frame, status_area);
    render_bars(model, frame, bars_area);
}

fn render_safe_exit(frame: &mut Frame, area: Rect) {
    let styled_text = Span::styled(
        "Press Ctrl-C again to exit",
        Style::default().fg(Color::DarkGray),
    );
    let textual = Paragraph::new(styled_text);
    frame.render_widget(textual, area);
}

fn render_playing_status(model: &Model, frame: &mut Frame, area: Rect) {
    let yellow = |s: &str| {
        Span::styled(
            if s.is_empty() { "Unknown" } else { s }.to_string(),
            Style::default().fg(Color::Yellow),
        )
    };

    let state_text = match model.app_state {
        AppState::Player(PlayerState::Playing) => "Playing ",
        _ => "Paused ",
    };

    let spans = match model.queue.get_current_song() {
        None => vec![Span::raw("Queue is empty")],
        Some(song) => match &song.metadata {
            Some(m) if !(m.title.is_empty() && m.artist.is_empty() && m.album.is_empty()) => vec![
                Span::raw(state_text),
                yellow(&m.title),
                Span::raw(" by "),
                yellow(&m.artist),
                Span::raw(" from "),
                yellow(&m.album),
            ],
            _ => vec![Span::raw(state_text), yellow("Unknown song")],
        },
    };

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn render_bars(model: &Model, frame: &mut Frame, area: Rect) {
    let [progress_area, volume_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70), // progress bar
            Constraint::Percentage(30), // volume bar
        ])
        .spacing(1)
        .areas(area);

    render_progress_bar(model, frame, progress_area);
    render_volume_bar(model, frame, volume_area);
}

fn render_progress_bar(model: &Model, frame: &mut Frame, area: Rect) {
    let current_secs = controls::get_current_duration(model).as_secs();

    let total_secs = controls::get_current_song_total_duration(model)
        .unwrap_or_default()
        .as_secs();

    let ratio = if total_secs > 0 {
        ((current_secs as f64) / (total_secs as f64)).clamp(0.0, 1.0)
    } else {
        0f64
    };

    let label = format!(
        "{:02}:{:02} - {:02}:{:02}",
        current_secs / 60,
        current_secs % 60,
        total_secs / 60,
        total_secs % 60
    );

    LineGauge::default()
        .ratio(ratio)
        .filled_symbol(symbols::line::HEAVY_QUADRUPLE_DASH_HORIZONTAL)
        .filled_style(Style::default().fg(Color::Yellow))
        .label(label)
        .render(area, frame.buffer_mut());
}

fn render_volume_bar(model: &Model, frame: &mut Frame, area: Rect) {
    let ratio = controls::get_volume(model).clamp(0.0, 1.0) as f64;

    LineGauge::default()
        .ratio(ratio)
        .filled_symbol(symbols::line::HEAVY_QUADRUPLE_DASH_HORIZONTAL)
        .filled_style(Style::default().fg(Color::Yellow))
        .render(area, frame.buffer_mut());
}
