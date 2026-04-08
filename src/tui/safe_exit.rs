use crate::{Model, Popup};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::tui::bar::bar_area;

pub fn render_safe_exit_popup(model: &Model, frame: &mut Frame, area: Rect) {
    let Some(Popup::SafeExit) = model.popup else {
        return;
    };

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .style(Style::default());
    let textual =
        Paragraph::new("    Confirm exit by pressing Ctrl-C or press esc to return to dutar")
            .block(block);

    let popup_area = bar_area(area);
    frame.render_widget(Clear, popup_area);
    frame.render_widget(textual, popup_area);
}
