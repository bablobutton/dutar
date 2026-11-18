use crate::Model;
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListItem};

const HOTKEYS: [[&str; 2]; 13] = [
    ["k", "Toggle play/pause"],
    ["Ctrl+C", "Quit with confirmation"],
    ["l", "Skip forward"],
    ["j", "Skip back"],
    ["n", "Next track"],
    ["p", "Previous track"],
    ["m", "Toggle mute"],
    ["=/+", "Volume up"],
    ["-/_", "Volume down"],
    ["?", "Show hints"],
    ["/", "Search"],
    ["Esc", "Clear search"],
    ["0..=9", "Jump within track"],
];

pub fn render_hints_popup(model: &Model, frame: &mut Frame, area: Rect) {
    if !model.show_hints {
        return;
    }

    let items = HOTKEYS.iter().map(|[key, desc]| {
        ListItem::new(Line::from(vec![
            Span::raw(*key),
            Span::styled(" - ", Style::default().fg(Color::DarkGray)),
            Span::styled(*desc, Style::default().fg(Color::DarkGray)),
        ]))
    });
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Hotkeys"),
    );

    let width = HOTKEYS
        .iter()
        .map(|[key, desc]| key.len() + desc.len() + 3)
        .max()
        .unwrap_or(0) as u16
        + 2;
    let height = HOTKEYS.len() as u16 + 2;
    let popup_area = area.centered(
        Constraint::Length(width.min(area.width)),
        Constraint::Length(height.min(area.height)),
    );

    frame.render_widget(Clear, popup_area);
    frame.render_widget(list, popup_area);
}
