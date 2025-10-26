use crate::{Model, Popup};
use ratatui::Frame;

pub fn render_hints_popup(model: &Model, frame: &mut Frame) {
    let Some(Popup::Hint) = model.popup else {
        return;
    };
    todo!();
}
