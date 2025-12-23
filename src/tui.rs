mod bar;
mod hints;
mod queue;

use crate::Model;
use bar::render_bar_popup;
use hints::render_hints_popup;
use queue::render_queue;
use ratatui::layout::Rect;
use ratatui::layout::{Constraint, Direction, Flex, Layout};
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::{BorderType, Gauge};
use ratatui::{
    Frame,
    widgets::{Block, Borders},
};

const MAX_APP_WIDTH: u16 = 100;
const MAX_APP_HEIGHT: u16 = 30;

fn center_area(area: Rect, width: u16, height: u16) -> Rect {
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    area
}

pub fn render(model: &mut Model, frame: &mut Frame) {
    // Limit the app area to MAX_APP_WIDTH x MAX_APP_HEIGHT (centered)
    let terminal_area = frame.area();
    let app_width = MAX_APP_WIDTH.min(terminal_area.width);
    let app_height = MAX_APP_HEIGHT.min(terminal_area.height);

    let app_area = center_area(terminal_area, app_width, app_height);

    // chunks divide main UI into regions into which we'll put things
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(2),        // Play text
            Constraint::Percentage(10), // Volume
            Constraint::Percentage(10), // Progress bar
        ])
        .split(app_area);

    render_queue(model, frame, chunks[0]);
    render_volume(model, frame, chunks[1]);
    render_progress(model, frame, chunks[2]);

    // render popups
    render_bar_popup(model, frame, app_area);
    render_hints_popup(model, frame, app_area);
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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Progress"),
        )
        .ratio(ratio)
        .style(Style::default().fg(Color::Yellow))
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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Volume"),
        )
        .ratio(ratio)
        .style(Style::default().fg(Color::Yellow))
        .label(format!("{curr_vol:.2}",));

    frame.render_widget(gauge, chunk);
}
