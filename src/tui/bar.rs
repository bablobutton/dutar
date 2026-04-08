// this should render all bars

use crate::{BarType, Model, Popup};
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

pub fn render_bar_popup(model: &Model, frame: &mut Frame, area: Rect) {
    let Some(Popup::Bar(bar_state)) = &model.popup else {
        return;
    };

    match &bar_state.bar_type {
        BarType::Command => render_command_bar(model, frame, area),
        BarType::Search => render_search_bar(model, frame, area),
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
        Span::styled(input.clone(), Style::default().fg(Color::Yellow)),
    ]))
    .block(block);

    let popup_area = bar_area(area);
    frame.render_widget(Clear, popup_area);
    frame.render_widget(paragraph, popup_area);
}

fn render_search_bar(model: &Model, frame: &mut Frame, area: Rect) {
    let Some(Popup::Bar(search_bar)) = &model.popup else {
        unreachable!();
    };

    let input = &search_bar.input;

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .style(Style::default());

    let paragraph = Paragraph::new(Line::from(vec![
        Span::from("   ?> "),
        Span::styled(input.clone(), Style::default().fg(Color::Yellow)),
    ]))
    .block(block);

    let popup_area = bar_area(area);
    frame.render_widget(Clear, popup_area);
    frame.render_widget(paragraph, popup_area);
}

pub fn bar_area(r: Rect) -> Rect {
    const BAR_HEIGHT: u16 = 2;
    const BAR_WIDTH: u16 = 70;

    // Position the bar 1/4 from the top (not center)
    let vertical_offset = r.height / 4;
    let centered_horizontally = r.centered_horizontally(Constraint::Length(BAR_WIDTH.min(r.width)));

    Rect {
        x: centered_horizontally.x,
        y: r.y + vertical_offset,
        width: centered_horizontally.width,
        height: BAR_HEIGHT.min(r.height.saturating_sub(vertical_offset)),
    }
}
