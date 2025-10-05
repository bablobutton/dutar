mod audio;
mod controls;
mod logging;
mod tui;

use color_eyre::Result;
use log::{debug, error};
use ratatui::crossterm::event;
use rodio::{OutputStream, Sink};
use std::time::Duration;

struct Model {
    state: State,
    player: Player,
}

impl Model {
    pub fn new() -> Model {
        let stream =
            rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");
        let sink = rodio::Sink::connect_new(stream.mixer());
        Model {
            state: State::Init,
            player: Player { stream, sink },
        }
    }
}

struct Player {
    stream: OutputStream,
    sink: Sink,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum State {
    #[default]
    Init,
    Player(PlayerState), // nested enum for playing screen
    Done,
}

#[derive(Debug, PartialEq, Eq)]
enum PlayerState {
    Playing,
    Paused,
}

#[derive(PartialEq, Debug)]
enum Message {
    TogglePlay,
    SkipForward,
    SkipBack,
    Next,
    Previous,
    Quit,
}

fn main() -> Result<()> {
    logging::init();
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let mut model = Model::new();

    while model.state != State::Done {
        terminal.draw(|frame| tui::render_state(&model, frame))?;
        let mut msg = handle_event(&model)?;

        while msg.is_some() {
            msg = update(&mut model, msg.unwrap());
        }
    }

    ratatui::restore();
    Ok(())
}

fn handle_event(_model: &Model) -> Result<Option<Message>> {
    if event::poll(Duration::from_millis(100))? {
        if let event::Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(key));
            }
        }
    }
    Ok(None)
}

fn handle_key(key: event::KeyEvent) -> Option<Message> {
    match key.code {
        event::KeyCode::Char('k') => Some(Message::TogglePlay),
        event::KeyCode::Char('q') => Some(Message::Quit),
        event::KeyCode::Char('l') => Some(Message::SkipForward),
        event::KeyCode::Char('j') => Some(Message::SkipBack),
        event::KeyCode::Char('n') => Some(Message::Next),
        event::KeyCode::Char('p') => Some(Message::Previous),
        _ => None,
    }
}

fn update(model: &mut Model, msg: Message) -> Option<Message> {
    debug!(
        "Current state [{:?}] <- Message [{:?}]",
        model.state, msg
    );
    let ret = match msg {
        Message::TogglePlay => {
            controls::toggle_play(model);
            None
        }
        Message::Quit => {
            model.state = State::Done;
            None
        }
        Message::SkipForward => {
            controls::forward_seconds(model, 5);
            None
        }
        Message::SkipBack => {
            controls::backward_seconds(model, 5);
            None
        }
        Message::Next => {
            todo!();
        }
        Message::Previous => {
            todo!();
        }
    };
    debug!("Updated state [{:?}]", model.state);
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // check state transitions (not all but the important ones)
    fn test_update() {
        let mut model = Model::new();
        assert_eq!(model.state, State::Init);

        // Init -> Playing
        let result = update(&mut model, Message::TogglePlay);
        assert_eq!(result, None);
        assert_eq!(model.state, State::Player(PlayerState::Playing));

        // Playing -> Paused
        let result = update(&mut model, Message::TogglePlay);
        assert_eq!(result, None);
        assert_eq!(model.state, State::Player(PlayerState::Paused));

        // Paused -> Playing
        let result = update(&mut model, Message::TogglePlay);
        assert_eq!(result, None);
        assert_eq!(model.state, State::Player(PlayerState::Playing));

        // Playing -> Quit
        let result = update(&mut model, Message::Quit);
        assert_eq!(result, None);
        assert_eq!(model.state, State::Done);
    }
}
