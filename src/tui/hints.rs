use crate::{Model, Popup};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListItem};

pub fn render_hints_popup(model: &Model, frame: &mut Frame, area: Rect) {
    let Some(Popup::Hint) = model.popup else {
        return;
    };

    let hotkey_pairs = [
        ["k", "Toggle play/pause"],
        ["Ctrl+C", "Quit with confirmation"],
        ["l", "Skip forward"],
        ["j", "Skip back"],
        ["n", "Next track"],
        ["p", "Previous track"],
        ["m", "Toggle mute"],
        [":", "Open command bar"],
        ["=/+", "Volume up"],
        ["-/_", "Volume down"],
        ["?", "Show hints"],
        ["0..=9", "Jump within track"],
    ];

    let command_pairs = [
        ["next", "Next track"],
        ["prev", "Previous track"],
        ["toggleplay", "Toggle play/pause"],
        ["play", "Play"],
        ["pause", "Pause"],
        ["setv <0-100>", "Set volume level"],
        ["quit, q", "Quit"],
        ["mute", "Mute"],
        ["unmute", "Unmute"],
        ["togglemute", "Toggle mute"],
        ["jump <0-9>", "Jump within track"],
    ];

    // Calculate popup area
    let popup_area = hints_area(&hotkey_pairs, &command_pairs, area);
    frame.render_widget(Clear, popup_area); // Clear the background first

    // Split area into left and right halves
    let [hotkey_area, command_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .areas(popup_area);

    // Render hotkeys on the left
    let mut hotkey_items = Vec::<ListItem>::new();
    for pair in hotkey_pairs {
        let hint = pair[0];
        let desc = pair[1];
        hotkey_items.push(ListItem::new(Line::from(vec![
            Span::styled(hint, Style::default()),
            Span::styled(" - ", Style::default().fg(Color::DarkGray)),
            Span::styled(desc, Style::default().fg(Color::DarkGray)),
        ])));
    }

    let hotkey_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Hotkeys");

    let hotkey_list = List::new(hotkey_items).block(hotkey_block);
    frame.render_widget(hotkey_list, hotkey_area);

    // Render commands on the right
    let mut command_items = Vec::<ListItem>::new();
    for pair in command_pairs {
        let cmd = pair[0];
        let desc = pair[1];
        command_items.push(ListItem::new(Line::from(vec![
            Span::styled(format!(":{cmd}"), Style::default()),
            Span::styled(" - ", Style::default().fg(Color::DarkGray)),
            Span::styled(desc, Style::default().fg(Color::DarkGray)),
        ])));
    }

    let command_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Commands");

    let command_list = List::new(command_items).block(command_block);
    frame.render_widget(command_list, command_area);
}

fn hints_area(hotkey_pairs: &[[&str; 2]], command_pairs: &[[&str; 2]], r: Rect) -> Rect {
    const BORDER_WIDTH: u16 = 2; // 1 cell on each side for borders

    // Calculate the maximum content width for hotkeys
    let max_hotkey_width = hotkey_pairs
        .iter()
        .map(|pair| format!("{} - {}", pair[0], pair[1]).len())
        .max()
        .unwrap_or(0) as u16;

    // Calculate the maximum content width for commands (with : prefix)
    let max_command_width = command_pairs
        .iter()
        .map(|pair| format!(":{} - {}", pair[0], pair[1]).len())
        .max()
        .unwrap_or(0) as u16;

    // Total width needed for both columns
    let needed_width = 2 * (max_hotkey_width.max(max_command_width) + BORDER_WIDTH);

    // Width: 2/3 of available area by default, but expand if content needs more
    let default_width = (r.width * 2) / 3;
    let popup_width = needed_width.max(default_width).min(r.width);

    // Height: 1/3 of available area by default, but expand if content needs more
    let default_height = r.height / 3;
    let max_items = hotkey_pairs.len().max(command_pairs.len()) as u16;
    let needed_height = max_items + BORDER_WIDTH; // +2 for borders
    let popup_height = needed_height.max(default_height).min(r.height);

    r.centered(
        Constraint::Length(popup_width),
        Constraint::Length(popup_height),
    )
}
