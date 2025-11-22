mod controls;
mod logging;
mod queue;
mod tui;
mod utils;

use color_eyre::Result;
use log::{debug, error, warn};
use queue::SongQueue;
use ratatui::crossterm::event;
use ratatui::widgets::ScrollbarState;
use rodio::{OutputStream, Sink};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::time::Duration;

const ITEM_HEIGHT: usize = 4;
// represents a state of the app
struct Model {
    app_state: AppState,     // "main" state of the app for music controls
    popup: Option<Popup>,    // popup state, if any. Ex: command bar, search bar
    ui_state: UIState,       // TODO: state of UI, will need this when have more than one screen
    saved_state: SavedState, // storing whatever we might need to restore later
    scroll_state: ScrollbarState,
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
        let queue = SongQueue::new();
        let scroll_state = ScrollbarState::new(queue.get_current_queue().len());
        Model {
            app_state: AppState::Init,
            ui_state: UIState::Main,
            saved_state: SavedState { volume: 1.0f32 },
            scroll_state,
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

struct SavedState {
    volume: f32,
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
    Play,
    Pause,
    SkipForward,
    SkipBack,
    Next,
    Previous,
    Quit,
    OpenCommandBar,
    ClosePopup,
    PopupSubmit,
    SendCharToPopup(char),
    VolumeUp,
    VolumeDown,
    SetVolume(u8),
    ToggleMute,
    Mute,
    Unmute,
    EraseChar,
    OpenHint,
}

fn main() -> Result<()> {
    logging::init();
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let mut model = Model::new();

    // main event loop, see ELM https://ratatui.rs/concepts/application-patterns/the-elm-architecture/
    while model.app_state != AppState::Done {
        terminal.draw(|frame| tui::render(&mut model, frame))?;
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
        Some(Popup::Bar(_)) => match key.code {
            event::KeyCode::Enter => Some(Message::PopupSubmit),
            event::KeyCode::Esc => Some(Message::ClosePopup),
            event::KeyCode::Char(c) => Some(Message::SendCharToPopup(c)),
            event::KeyCode::Backspace => Some(Message::EraseChar),
            _ => None,
        },
        Some(Popup::Hint) => match key.code {
            event::KeyCode::Enter => Some(Message::ClosePopup),
            event::KeyCode::Esc => Some(Message::ClosePopup),
            event::KeyCode::Char('q') => Some(Message::ClosePopup),
            event::KeyCode::Char(_) => handle_hotkey(key.code),
            _ => None,
        },
        None => handle_hotkey(key.code),
    }
}

fn handle_hotkey(keycode: event::KeyCode) -> Option<Message> {
    match keycode {
        event::KeyCode::Char('k') => Some(Message::TogglePlay),
        event::KeyCode::Char('q') => Some(Message::Quit),
        event::KeyCode::Char('l') => Some(Message::SkipForward),
        event::KeyCode::Char('j') => Some(Message::SkipBack),
        event::KeyCode::Char('n') => Some(Message::Next),
        event::KeyCode::Char('p') => Some(Message::Previous),
        event::KeyCode::Char('m') => Some(Message::ToggleMute),
        event::KeyCode::Char(':') => Some(Message::OpenCommandBar),
        event::KeyCode::Char('=') | event::KeyCode::Char('+') => Some(Message::VolumeUp),
        event::KeyCode::Char('-') | event::KeyCode::Char('_') => Some(Message::VolumeDown),
        event::KeyCode::Char('?') => Some(Message::OpenHint),
        _ => None,
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
            if model.app_state == AppState::Player(PlayerState::Playing) {
                Some(Message::Pause)
            } else {
                Some(Message::Play)
            }
        }
        Message::Play => {
            match controls::play(model) {
                Ok(()) => model.app_state = AppState::Player(PlayerState::Playing),
                Err(err) => error!("Error trying to play: {err}"),
            }
            None
        }
        Message::Pause => {
            controls::pause(model);
            model.app_state = AppState::Player(PlayerState::Paused);
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
        Message::VolumeUp => {
            controls::volume_up(model, 0.05);
            None
        }
        Message::VolumeDown => {
            controls::volume_down(model, 0.05);
            None
        }
        Message::SetVolume(vol) => {
            let volf32 = vol.clamp(0, 100) as f32 / 100.0f32;
            controls::set_volume(model, volf32);
            None
        }
        Message::ToggleMute => {
            if controls::get_volume(model) == 0f32 {
                Some(Message::Unmute)
            } else {
                Some(Message::Mute)
            }
        }
        Message::Mute => {
            let vol = controls::get_volume(model);
            if vol != 0f32 {
                model.saved_state.volume = vol;
                controls::set_volume(model, 0f32);
            }
            None
        }
        Message::Unmute => {
            let vol = controls::get_volume(model);
            if vol == 0f32 {
                controls::set_volume(model, model.saved_state.volume);
            }
            None
        }
        Message::OpenCommandBar => {
            model.popup = Some(Popup::Bar(BarState {
                input: String::with_capacity(64),
                bar_type: BarType::Command,
            }));
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
            debug_assert!(
                model.popup.is_some(),
                "PopupSubmit shouldn't be sent if there's no popup"
            );
            handle_popup_submit(model)
        }
        Message::SendCharToPopup(c) => {
            debug_assert!(
                model.popup.is_some(),
                "SendCharToPopup shouldn't be sent if there's no popup open"
            );
            if let Some(Popup::Bar(bar_state)) = &mut model.popup {
                bar_state.input.push(c)
            };
            None
        }
        Message::EraseChar => {
            debug_assert!(
                matches!(model.popup, Some(Popup::Bar(_))),
                "EraseChar shouldn't be sent if there's no bar open"
            );
            if let Some(Popup::Bar(bar_state)) = &mut model.popup {
                if !bar_state.input.is_empty() {
                    bar_state.input.pop();
                }
            }
            None
        }
        Message::OpenHint => {
            model.popup = Some(Popup::Hint);
            None
        }
    };
    debug!(
        "Updated state [{:?}], popup [{:?}], Message [{:?}]",
        model.app_state, model.popup, ret
    );
    ret
}

fn handle_popup_submit(model: &mut Model) -> Option<Message> {
    let Some(popup) = &model.popup else {
        unreachable!();
    };

    let ret = match popup {
        Popup::Bar(bar) => match bar.bar_type {
            BarType::Command => handle_command(model),
            // BarType::Search => handle_search(model),
        },
        Popup::Hint => None,
    };

    model.popup = None;
    ret
}

fn handle_command(model: &mut Model) -> Option<Message> {
    let Some(Popup::Bar(bar)) = &model.popup else {
        unreachable!();
    };
    if bar.bar_type != BarType::Command {
        unreachable!();
    }

    let argv: Vec<&str> = bar.input.split_whitespace().collect();
    if argv.is_empty() {
        return None;
    }

    match argv[0] {
        "next" => Some(Message::Next),
        "prev" => Some(Message::Previous),
        "toggleplay" => Some(Message::TogglePlay),
        "play" => Some(Message::Play),
        "pause" => Some(Message::Pause),
        "setv" => handle_set_volume(&argv),
        "quit" | "q" => Some(Message::Quit),
        "mute" => Some(Message::Mute),
        "unmute" => Some(Message::Unmute),
        "togglemute" => Some(Message::ToggleMute),
        _ => None,
    }
}

fn handle_set_volume(argv: &Vec<&str>) -> Option<Message> {
    if argv.len() >= 2 {
        let volume: std::result::Result<u8, std::num::ParseIntError> = argv[1].parse();
        if let Ok(v) = volume {
            return Some(Message::SetVolume(v));
        }
        return None;
    }
    warn!("No argument supplied to set volume");
    None
}

fn handle_search(model: &mut Model) -> Option<Message> {
    todo!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // state transitions (not all but the important ones)
    // TODO: tests can't load music. need mocks.
    fn test_update() {
        // let mut model = Model::new();
        // assert_eq!(model.app_state, AppState::Init);
        //
        // // Init -> Playing
        // let result = update(&mut model, Message::TogglePlay);
        // assert_eq!(result, None);
        // assert_eq!(model.app_state, AppState::Player(PlayerState::Playing));
        //
        // // Playing -> Paused
        // let result = update(&mut model, Message::TogglePlay);
        // assert_eq!(result, None);
        // assert_eq!(model.app_state, AppState::Player(PlayerState::Paused));
        //
        // // Paused -> Playing
        // let result = update(&mut model, Message::TogglePlay);
        // assert_eq!(result, None);
        // assert_eq!(model.app_state, AppState::Player(PlayerState::Playing));
        //
        // // open command bar
        // let result = update(&mut model, Message::OpenCommandBar);
        // assert_eq!(result, None);
        // assert_eq!(model.app_state, AppState::Player(PlayerState::Playing)); // didn't change
        // assert!(matches!(model.popup, Some(Popup::Bar(_))));
        // if let Some(Popup::Bar(bar)) = &model.popup {
        //     assert_eq!(bar.bar_type, BarType::Command);
        // } else {
        //     assert!(false);
        // }
        //
        // // send char to popup
        // let result = update(&mut model, Message::SendCharToPopup('x'));
        // assert_eq!(result, None);
        // assert_eq!(model.app_state, AppState::Player(PlayerState::Playing)); // didn't change
        // assert!(matches!(model.popup, Some(Popup::Bar(_))));
        // if let Some(Popup::Bar(bar)) = &model.popup {
        //     assert_eq!(bar.bar_type, BarType::Command);
        //     assert_eq!(bar.input, "x");
        // } else {
        //     assert!(false);
        // }
        //
        // // erase that char
        // let result = update(&mut model, Message::EraseChar);
        // assert_eq!(result, None);
        // assert_eq!(model.app_state, AppState::Player(PlayerState::Playing)); // didn't change
        // assert!(matches!(model.popup, Some(Popup::Bar(_))));
        // if let Some(Popup::Bar(bar)) = &model.popup {
        //     assert_eq!(bar.bar_type, BarType::Command);
        //     assert!(bar.input.is_empty());
        // } else {
        //     assert!(false);
        // }
        //
        // // close the popup
        // let result = update(&mut model, Message::ClosePopup);
        // assert_eq!(result, None);
        // assert_eq!(model.app_state, AppState::Player(PlayerState::Playing)); // didn't change
        // assert!(model.popup.is_none());
        //
        // // Playing -> Quit
        // let result = update(&mut model, Message::Quit);
        // assert_eq!(result, None);
        // assert_eq!(model.app_state, AppState::Done);
    }
}
