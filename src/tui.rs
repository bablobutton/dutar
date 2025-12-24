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
use ratatui::layout::Rect;
use ratatui::layout::{Constraint, Direction, Flex, Layout};

const MAX_APP_WIDTH: u16 = 100;
const MAX_APP_HEIGHT: u16 = 30;

fn center_area(area: Rect, width: u16, height: u16) -> Rect {
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    area
}

pub fn render(model: &mut Model, frame: &mut Frame) {
    // Limit the app area to MAX_APP_WIDTH x MAX_APP_HEIGHT (centered)
    let terminal_area = frame.area();
    let app_width = MAX_APP_WIDTH.min(terminal_area.width);
    let app_height = MAX_APP_HEIGHT.min(terminal_area.height);

    let app_area = center_area(terminal_area, app_width, app_height);

    // chunks divide main UI into regions into which we'll put things
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),   // song queue
            Constraint::Length(2), // footer
        ])
        .split(app_area);

    render_queue(model, frame, chunks[0]);
    render_footer(model, frame, chunks[1]);

    // render popups
    render_bar_popup(model, frame, app_area);
    render_hints_popup(model, frame, app_area);
}
