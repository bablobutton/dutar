// this should render all bars

use crate::{BarType, Model, Popup};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

pub fn render_bar_popup(model: &Model, frame: &mut Frame, area: Rect) {
    let Some(Popup::Bar(bar_state)) = &model.popup else {
        return;
    };

    match &bar_state.bar_type {
        BarType::Command => render_command_bar(model, frame, area),
        // BarType::Search => render_search_bar(model, frame, area),
    };
}

fn render_command_bar(model: &Model, frame: &mut Frame, area: Rect) {
    let Some(Popup::Bar(command_bar)) = &model.popup else {
        unreachable!();
    };

    let input = &command_bar.input;

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .style(Style::default());

    let paragraph = Paragraph::new(Line::from(vec![
        Span::from("   > "),
        Span::styled(input.clone(), Style::default().fg(Color::White)),
    ]))
    .block(block);

    let popup_area = bar_area(area);
    frame.render_widget(Clear, popup_area);
    frame.render_widget(paragraph, popup_area);
}

fn bar_area(r: Rect) -> Rect {
    // Create vertical layout with popup centered between top and bottom margins
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(2),
            Constraint::Fill(3),
        ])
        .split(r);

    // Create horizontal layout with popup centered between left and right margins
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 4),
            Constraint::Fill(1),
            Constraint::Max(70),
            Constraint::Fill(1),
            Constraint::Ratio(1, 4),
        ])
        .split(popup_layout[1])[2] // Take the middle section
}
