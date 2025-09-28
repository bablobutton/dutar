use super::{Model, PlayerState, State};
use ratatui::{Frame, widgets::Paragraph};

pub fn render_state(model: &Model, frame: &mut Frame) {
    let current_state_string = match model.state {
        State::Init => "init",
        State::Player(PlayerState::Playing) => "playing",
        State::Player(PlayerState::Paused) => "paused",
        State::Done => "done",
    };
    frame.render_widget(Paragraph::new(current_state_string), frame.area());
}
