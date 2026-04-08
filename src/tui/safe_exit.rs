use crate::{Model, Popup};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

pub fn render_safe_exit_popup(model: &Model, frame: &mut Frame, area: Rect) {
    let Some(Popup::SafeExit) = model.popup else {
        return;
    };
    let popup_width = area.width * 3 / 4;
    let popup_height = 4;
    let popup_x = area.x + (area.width - popup_width) / 2;
    let popup_y = area.y + (area.height - popup_height) / 2;

    let popup_area = Rect { x: popup_x, y: popup_y, width: popup_width, height: popup_height };

    let block = Block::default()
        .title(" Safe Exit")
        .borders(Borders::ALL)
        .style(Style::default());
    let textual =
        Paragraph::new("!! Confirm exit by pressing Ctrl-C or press esc to return to dutar !!")
            .block(block);

    
    frame.render_widget(Clear, popup_area);
    frame.render_widget(textual, popup_area);
}
