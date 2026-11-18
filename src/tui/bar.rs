use crate::{BarType, Model, Popup};
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

pub fn render_bar_popup(model: &Model, frame: &mut Frame, area: Rect) {
    let Some(Popup::Bar(bar)) = &model.popup else {
        return;
    };

    let prompt = match bar.bar_type {
        BarType::Command => "   > ",
        BarType::Search => "   ?> ",
    };

    let paragraph = Paragraph::new(Line::from(vec![
        Span::from(prompt),
        Span::styled(bar.input.as_str(), Style::default().fg(Color::Yellow)),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));

    let popup_area = bar_area(area);
    frame.render_widget(Clear, popup_area);
    frame.render_widget(paragraph, popup_area);
}

fn bar_area(r: Rect) -> Rect {
    const BAR_HEIGHT: u16 = 2;
    const BAR_WIDTH: u16 = 70;

    let vertical_offset = r.height / 4;
    let centered_horizontally = r.centered_horizontally(Constraint::Length(BAR_WIDTH.min(r.width)));

    Rect {
        x: centered_horizontally.x,
        y: r.y + vertical_offset,
        width: centered_horizontally.width,
        height: BAR_HEIGHT.min(r.height.saturating_sub(vertical_offset)),
    }
}
