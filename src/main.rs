mod audio;
mod controls;
mod logging;
mod queue;
mod tui;
mod utils;

use color_eyre::Result;
use log::debug;
use queue::SongQueue;
use ratatui::crossterm::event;
use rodio::{OutputStream, Sink};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

struct Model {
    state: State,
    audio: Audio,
    queue: SongQueue,
    channel: Channel,
}

impl Model {
    pub fn new() -> Model {
        let stream =
            rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");
        let sink = rodio::Sink::connect_new(stream.mixer());
        let (tx, rx): (Sender<Message>, Receiver<Message>) = channel();
        Model {
            state: State::Init,
            audio: Audio {
                _stream: stream, // it's unused, but we can't have it dropped
                sink: sink,
            },
            queue: SongQueue::new(),
            channel: Channel { rx, tx },
        }
    }
}

// channel for sending and receiving events
struct Channel {
    rx: Receiver<Message>,
    tx: Sender<Message>,
}

struct Audio {
    _stream: OutputStream,
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

fn handle_event(model: &Model) -> Result<Option<Message>> {
    let channel_msg = model.channel.rx.recv_timeout(Duration::from_millis(10));
    match channel_msg {
        Ok(msg) => {
            debug!("Received message over channel: {msg:?}");
            return Ok(Some(msg));
        }
        Err(_) => {}
    }

    if event::poll(Duration::from_millis(90))? {
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
    debug!("Current state [{:?}] <- Message [{:?}]", model.state, msg);
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
            controls::play_next(model);
            None
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
    // state transitions (not all but the important ones)
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
