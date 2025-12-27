use crate::Model;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::BorderType;
use ratatui::{
    Frame,
    widgets::{Block, Borders, Row, Table, TableState},
};
use std::ffi::OsStr;

const SCROLL_PADDING: usize = 3;

fn apply_scroll_padding(table_state: &mut TableState, current_idx: Option<usize>, chunk: &Rect) {
    let Some(idx) = current_idx else { return };

    // visible_rows = chunk height - 2 (borders) - 1 (header)
    let visible_rows = chunk.height.saturating_sub(3) as usize;

    // offset is the index of the first song to be rendered
    let offset = table_state.offset_mut();

    // Selection too close to top: idx is within SCROLL_PADDING of the first visible row.
    // Scroll up so that SCROLL_PADDING rows appear above the selection.
    if idx < *offset + SCROLL_PADDING {
        *offset = idx.saturating_sub(SCROLL_PADDING);
    }
    // Selection too close to bottom: idx is within SCROLL_PADDING of the last visible row.
    // Scroll down so that SCROLL_PADDING rows appear below the selection.
    // The +1 accounts for the selected row itself occupying one of the visible rows.
    else if idx >= *offset + visible_rows.saturating_sub(SCROLL_PADDING) {
        *offset = idx.saturating_sub(visible_rows.saturating_sub(SCROLL_PADDING + 1));
    }
}

pub fn render_queue(model: &mut Model, frame: &mut Frame, chunk: Rect) {
    let current_idx = model.queue.get_current_idx();
    model.ui.song_queue_table.select(current_idx);

    let header = Row::new(vec!["Artist", "Title", "Album", "Duration"])
        .style(Style::default().fg(Color::Yellow));

    let rows: Vec<Row> = model
        .queue
        .iter()
        .map(|song| {
            let (artist, title, album, duration) = match &song.metadata {
                Some(meta) => (
                    meta.artist.as_str(),
                    meta.title.as_str(),
                    meta.album.as_str(),
                    format!(
                        "{}:{:02}",
                        meta.duration_seconds / 60,
                        meta.duration_seconds % 60
                    ),
                ),
                None => {
                    let filename = song
                        .path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown");
                    (filename, "", "", String::new())
                }
            };

            let artist = if artist.is_empty() {
                song.path
                    .file_name()
                    .unwrap_or(OsStr::new("Unknown"))
                    .to_str()
                    .unwrap_or("")
            } else {
                artist
            };

            Row::new(vec![
                artist.to_string(),
                title.to_string(),
                album.to_string(),
                duration,
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(25),
        Constraint::Percentage(35),
        Constraint::Percentage(40),
        Constraint::Length(8),
    ];

    let title = match model.queue.get_current_idx() {
        Some(idx) => format!("{}/{}", idx + 1, rows.len()),
        None => format!("_/{}", rows.len()),
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(title),
        )
        .row_highlight_style(Style::default().fg(Color::Black).bg(Color::Yellow));

    apply_scroll_padding(&mut model.ui.song_queue_table, current_idx, &chunk);

    frame.render_stateful_widget(table, chunk, &mut model.ui.song_queue_table);
}
