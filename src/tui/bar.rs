// this should render all bars

use crate::{BarType, Model, Popup};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render_bar_popup(model: &Model, frame: &mut Frame) {
    let Some(Popup::Bar(bar_state)) = &model.popup else {
        return;
    };

    match &bar_state.bar_type {
        BarType::Command => render_command_bar(model, frame),
        // BarType::Search => render_search_bar(model, frame),
    };
}

fn render_command_bar(model: &Model, frame: &mut Frame) {
    let block = Block::default()
        .title("Some title")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));

    // let area = centered_bar(frame.area());
}

// fn centered_bar(r: Rect) -> Rect {
//     // Create vertical layout with popup centered between top and bottom margins
//     let popup_layout = Layout::default()
//         .direction(Direction::Vertical)
//         .constraints([
//             Constraint::Percentage((100 - percent_y) / 2),  // Top margin
//             Constraint::Percentage(percent_y),              // Popup height
//             Constraint::Percentage((100 - percent_y) / 2),  // Bottom margin
//         ])
//         .split(r);
//
//     // Create horizontal layout with popup centered between left and right margins
//     Layout::default()
//         .direction(Direction::Horizontal)
//         .constraints([
//             Constraint::Percentage((100 - percent_x) / 2),  // Left margin
//             Constraint::Percentage(percent_x),              // Popup width
//             Constraint::Percentage((100 - percent_x) / 2),  // Right margin
//         ])
//         .split(popup_layout[1])[1]  // Take the middle section (index 1)
// }
