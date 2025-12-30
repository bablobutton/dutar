mod bar;
mod footer;
mod hints;
mod queue;

use crate::Model;
use bar::render_bar_popup;
use footer::render_footer;
use hints::render_hints_popup;
use queue::render_queue;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};

const MAX_APP_WIDTH: u16 = 100;
const MAX_APP_HEIGHT: u16 = 30;

pub fn render(model: &mut Model, frame: &mut Frame) {
    // Limit the app area to MAX_APP_WIDTH x MAX_APP_HEIGHT (centered)
    let terminal_area = frame.area();
    let app_width = MAX_APP_WIDTH.min(terminal_area.width);
    let app_height = MAX_APP_HEIGHT.min(terminal_area.height);

    let app_area = terminal_area.centered(
        Constraint::Length(app_width),
        Constraint::Length(app_height),
    );

    // Divide main UI into regions for queue and footer
    let [queue_area, footer_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),   // song queue
            Constraint::Length(2), // footer
        ])
        .areas(app_area);

    render_queue(model, frame, queue_area);
    render_footer(model, frame, footer_area);

    // render popups
    render_bar_popup(model, frame, app_area);
    render_hints_popup(model, frame, app_area);
}
