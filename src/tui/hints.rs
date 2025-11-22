use crate::{Model, Popup};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListItem};

pub fn render_hints_popup(model: &Model, frame: &mut Frame) {
    let Some(Popup::Hint) = model.popup else {
        return;
    };

    // Hotkeys
    let hotkey_pairs = [
        ["k", "Toggle play/pause"],
        ["q", "Quit"],
        ["l", "Skip forward"],
        ["j", "Skip back"],
        ["n", "Next track"],
        ["p", "Previous track"],
        ["m", "Toggle mute"],
        [":", "Open command bar"],
        ["=/+", "Volume up"],
        ["-/_", "Volume down"],
        ["?", "Show hints"],
    ];

    // Commands
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
    ];

    // Calculate popup area
    let area = hints_area(&hotkey_pairs, &command_pairs, frame.area());
    frame.render_widget(Clear, area); // Clear the background first

    // Split area into left and right halves
    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Render hotkeys on the left
    let mut hotkey_items = Vec::<ListItem>::new();
    for pair in hotkey_pairs {
        let hint = pair[0];
        let desc = pair[1];
        hotkey_items.push(ListItem::new(Line::from(Span::styled(
            format!("{hint} - {desc}"),
            Style::default(),
        ))));
    }

    let hotkey_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Hotkeys");

    let hotkey_list = List::new(hotkey_items).block(hotkey_block);
    frame.render_widget(hotkey_list, halves[0]);

    // Render commands on the right
    let mut command_items = Vec::<ListItem>::new();
    for pair in command_pairs {
        let cmd = pair[0];
        let desc = pair[1];
        command_items.push(ListItem::new(Line::from(Span::styled(
            format!(":{cmd} - {desc}"),
            Style::default(),
        ))));
    }

    let command_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Commands");

    let command_list = List::new(command_items).block(command_block);
    frame.render_widget(command_list, halves[1]);
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
    let popup_width = needed_width.max(default_width);

    // Height: 1/3 of available area by default, but expand if content needs more
    let default_height = r.height / 3;
    let max_items = hotkey_pairs.len().max(command_pairs.len()) as u16;
    let needed_height = max_items + BORDER_WIDTH; // +2 for borders
    let popup_height = needed_height.max(default_height);

    // Create vertical layout with popup centered between top and bottom margins
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),              // Top margin
            Constraint::Length(popup_height), // Popup height
            Constraint::Fill(1),              // Bottom margin
        ])
        .split(r);

    // Create horizontal layout with popup centered between left and right margins
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),             // Left margin
            Constraint::Length(popup_width), // Popup width
            Constraint::Fill(1),             // Right margin
        ])
        .split(popup_layout[1])[1] // Take the middle section (index 1)
}
