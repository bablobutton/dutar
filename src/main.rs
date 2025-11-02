mod controls;
mod logging;
mod queue;
mod tui;
mod utils;

use color_eyre::Result;
use log::{debug, error};
use queue::SongQueue;
use ratatui::crossterm::event;
use rodio::{OutputStream, Sink};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::time::Duration;

// represents a state of the app
struct Model {
    app_state: AppState,  // "main" state of the app for music controls
    popup: Option<Popup>, // popup state, if any. Ex: command bar, search bar
    ui_state: UIState,    // TODO: state of UI, will need this when have more than one screen
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
            app_state: AppState::Init,
            ui_state: UIState::Main,
            popup: None,
            audio: Audio {
                _stream: stream, // it's unused, but we can't have it dropped
                sink,
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
enum AppState {
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

#[derive(Debug, PartialEq)]
struct BarState {
    input: String,
    bar_type: BarType,
}

#[derive(Debug, PartialEq)]
enum Popup {
    Bar(BarState),
    Hint,
}

#[derive(Debug, PartialEq)]
enum BarType {
    Command,
    // Search,
}

#[derive(Debug, PartialEq)]
enum UIState {
    Main,
}

#[derive(PartialEq, Debug)]
enum Message {
    TogglePlay,
    SkipForward,
    SkipBack,
    Next,
    Previous,
    Quit,
    OpenCommandBar,
    ClosePopup,
    PopupSubmit,
    SendCharToPopup(char),
}

fn main() -> Result<()> {
    logging::init();
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let mut model = Model::new();

    // main event loop, see ELM https://ratatui.rs/concepts/application-patterns/the-elm-architecture/
    while model.app_state != AppState::Done {
        terminal.draw(|frame| tui::render(&model, frame))?;
        // event handler, converts event -> message
        let mut msg = handle_event(&model)?;

        while msg.is_some() {
            // acts on received message and updates the state.
            // can emit another message and process it too.
            msg = update(&mut model, msg.unwrap());
        }
    }

    ratatui::restore();
    Ok(())
}

// this is the main event loop handler, all events including thread communication
// and I/O should arrive as a message here
fn handle_event(model: &Model) -> Result<Option<Message>> {
    let channel_msg = model.channel.rx.recv_timeout(Duration::from_millis(10));
    match channel_msg {
        Ok(msg) => {
            debug!("Received message over channel: {msg:?}");
            return Ok(Some(msg));
        }
        Err(RecvTimeoutError::Timeout) => {}
        Err(err) => error!("Error receiving channel message: {err}"),
    }

    if event::poll(Duration::from_millis(90))? {
        if let event::Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(key, model));
            }
        }
    }
    Ok(None)
}

fn handle_key(key: event::KeyEvent, model: &Model) -> Option<Message> {
    match &model.popup {
        Some(_) => match key.code {
            event::KeyCode::Enter => Some(Message::PopupSubmit),
            event::KeyCode::Esc => Some(Message::ClosePopup),
            event::KeyCode::Char(c) => Some(Message::SendCharToPopup(c)),
            _ => None,
        },
        None => match key.code {
            event::KeyCode::Char('k') => Some(Message::TogglePlay),
            event::KeyCode::Char('q') => Some(Message::Quit),
            event::KeyCode::Char('l') => Some(Message::SkipForward),
            event::KeyCode::Char('j') => Some(Message::SkipBack),
            event::KeyCode::Char('n') => Some(Message::Next),
            event::KeyCode::Char('p') => Some(Message::Previous),
            event::KeyCode::Char(':') => Some(Message::OpenCommandBar),
            _ => None,
        },
    }
}

// this is where state gets updated
// when adding new state transitions, consider adding tests to solidify them
fn update(model: &mut Model, msg: Message) -> Option<Message> {
    debug!(
        "Current playback state [{:?}], popup [{:?}] <- Message [{:?}]",
        model.app_state, model.popup, msg
    );
    let ret = match msg {
        Message::TogglePlay => {
            controls::toggle_play(model);
            None
        }
        Message::Quit => {
            model.app_state = AppState::Done;
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
            controls::play_previous(model);
            None
        }
        Message::OpenCommandBar => {
            debug_assert!(
                model.popup.is_none(),
                "OpenCommandBar shouldn't be sent if there's a popup open"
            );
            if model.popup == None {
                model.popup = Some(Popup::Bar(BarState {
                    input: String::with_capacity(32),
                    bar_type: BarType::Command,
                }));
            }
            None
        }
        Message::ClosePopup => {
            debug_assert!(
                model.popup.is_some(),
                "ClosePopup shouldn't be sent if there's no popup"
            );
            model.popup = None;
            None
        }
        Message::PopupSubmit => {
            todo!();
        }
        Message::SendCharToPopup(c) => {
            debug_assert!(
                matches!(model.popup, Some(Popup::Bar(_))),
                "SendCharToPopup shouldn't be sent if there's no bar open"
            );
            if let Some(Popup::Bar(bar_state)) = &mut model.popup {
                bar_state.input.push(c)
            };
            None
        }
    };
    debug!(
        "Updated state [{:?}], popup [{:?}]",
        model.app_state, model.popup
    );
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // state transitions (not all but the important ones)
    fn test_update() {
        let mut model = Model::new();
        assert_eq!(model.app_state, AppState::Init);

        // Init -> Playing
        let result = update(&mut model, Message::TogglePlay);
        assert_eq!(result, None);
        assert_eq!(model.app_state, AppState::Player(PlayerState::Playing));

        // Playing -> Paused
        let result = update(&mut model, Message::TogglePlay);
        assert_eq!(result, None);
        assert_eq!(model.app_state, AppState::Player(PlayerState::Paused));

        // Paused -> Playing
        let result = update(&mut model, Message::TogglePlay);
        assert_eq!(result, None);
        assert_eq!(model.app_state, AppState::Player(PlayerState::Playing));

        // Playing -> Quit
        let result = update(&mut model, Message::Quit);
        assert_eq!(result, None);
        assert_eq!(model.app_state, AppState::Done);
    }
}
