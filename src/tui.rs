use crate::{Model, PlayerState, State};
use ratatui::layout::Rect;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::{
    Frame,
    style::Stylize,
    widgets::{Block, Borders, List, ListDirection, Paragraph},
};
use std::rc::Rc;

pub fn render_state(model: &Model, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(2), Constraint::Fill(1)])
        .split(frame.area());

    render_play_text(model, frame, &chunks);
    render_hints(frame, &chunks);
}

fn render_play_text(model: &Model, frame: &mut Frame, chunks: &Rc<[Rect]>) {
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let playing_state_str = match model.state {
        State::Player(PlayerState::Playing) => "Playing",
        State::Player(PlayerState::Paused) => "Paused",
        State::Init => "Welcome",
        State::Done => "Exiting",
    };
    let text = Paragraph::new(Text::styled(
        playing_state_str,
        Style::default().fg(Color::Green),
    ))
    .block(block);

    frame.render_widget(text, chunks[0]);
}

fn render_hints(frame: &mut Frame, chunks: &Rc<[Rect]>) {
    let block = Block::bordered().title("Hints");

    let items = ["k - play/pause", "q - quit"];
    let list = List::new(items)
        .block(block)
        .style(Style::new().white())
        .highlight_style(Style::new().italic())
        .direction(ListDirection::TopToBottom);

    frame.render_widget(list, chunks[1]);
}
