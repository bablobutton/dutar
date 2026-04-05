mod controls;
mod db;
mod logging;
mod queue;
mod tui;
mod utils;

use color_eyre::{Result, eyre::WrapErr};
use db::DB;
use log::{debug, error, warn};
use queue::SongQueue;
use ratatui::crossterm::event;
use ratatui::widgets::TableState;
use rodio::{OutputStream, Sink};
use std::env;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::time::Duration;

// represents a state of the app
struct Model {
    app_state: AppState,     // "main" state of the app for music controls
    popup: Option<Popup>,    // popup state, if any. Ex: command bar, search bar
    ui: UI,                  // TODO: state of UI, will need this when have more than one screen
    saved_state: SavedState, // storing whatever we might need to restore later
    audio: Audio,
    queue: SongQueue,
    channel: Channel,
    db: DB,
}

impl Model {
    pub fn init(args: Vec<String>) -> Result<Model> {
        let stream = rodio::OutputStreamBuilder::open_default_stream()?;
        let sink = rodio::Sink::connect_new(stream.mixer());
        let (tx, rx): (Sender<Message>, Receiver<Message>) = channel();
        let db = DB::new()?;
        let mut saved_state = db.read_saved_state().wrap_err("DB read saved state")?;

        let queue = if args.len() == 2 {
            // When exactly 1 argument is supplied,
            // treat it as a song or a directory of songs to open.
            // If unsuccessful, terminate app with error.
            let q = SongQueue::open_path(&args[1]).wrap_err("Error loading songs")?;
            debug!("Successfully loaded songs from {}", args[1]);
            // reset everything except volume level
            let vol = saved_state.volume;
            saved_state = SavedState::default();
            saved_state.volume = vol;
            q
        } else {
            SongQueue::restore_or_default(db.read_queue_songs_paths())
        };

        let mut model = Model {
            app_state: AppState::Init,
            ui: UI {
                state: UIState::Main,
                song_queue_table: TableState::new(),
            },
            saved_state,
            popup: None,
            audio: Audio {
                _stream: stream, // it's unused, but we can't have it dropped
                sink,
            },
            queue: queue,
            channel: Channel { rx, tx },
            db,
        };

        Self::restore_saved_state(&mut model);
        Self::load_current_song(&mut model);

        Ok(model)
    }

    fn restore_saved_state(model: &mut Model) {
        controls::set_volume(model, model.saved_state.volume);
        model
            .queue
            .set_current_song_idx(model.saved_state.current_song_index);
        controls::set_current_duration(model, model.saved_state.current_duration);
    }

    pub fn update_saved_state(&mut self) {
        self.saved_state.current_song_index = self.queue.get_current_idx().unwrap_or(0);
        self.saved_state.volume = controls::get_volume(self);
        self.saved_state.current_duration = controls::get_current_duration(self);
    }

    fn load_current_song(&mut self) {
        match controls::load_and_not_play(self) {
            Ok(_) => {}
            Err(e) => error!("Error loading song on init: {e}"),
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

#[derive(Debug, Default)]
struct SavedState {
    volume: f32,
    current_song_index: usize,
    current_duration: Duration,
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
    Search,
}

#[derive(Debug, PartialEq)]
enum UIState {
    Main,
}

struct UI {
    state: UIState,
    song_queue_table: TableState,
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
    OpenSearchBar,
    ClearSearch,
    PlayFromSearch(usize),
}

fn main() -> Result<()> {
    logging::init();
    color_eyre::install()?;
    let args: Vec<String> = env::args().collect();
    // app initialization, db creation/migrations, saved state restoration
    let mut model = Model::init(args)?;
    let mut terminal = ratatui::init();

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

    model.update_saved_state();
    model.db.write_saved_state(&model.saved_state)?;
    model.db.write_queue_songs(&model.queue)?;
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
    let key_code = utils::map_key_code(key.code);
    match &model.popup {
        Some(Popup::Bar(_)) => match key_code {
            event::KeyCode::Enter => Some(Message::PopupSubmit),
            event::KeyCode::Esc => Some(Message::ClosePopup),
            event::KeyCode::Char(c) => Some(Message::SendCharToPopup(c)),
            event::KeyCode::Backspace => Some(Message::EraseChar),
            _ => None,
        },
        Some(Popup::Hint) => match key_code {
            event::KeyCode::Enter => Some(Message::ClosePopup),
            event::KeyCode::Esc => Some(Message::ClosePopup),
            event::KeyCode::Char('q') => Some(Message::ClosePopup),
            event::KeyCode::Char(_) => handle_hotkey(key_code),
            _ => None,
        },
        None => match key_code {
            event::KeyCode::Esc => Some(Message::ClearSearch),
            _ => handle_hotkey(key_code),
        },
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
        event::KeyCode::Char('/') => Some(Message::OpenSearchBar),
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
            match controls::play_next(model) {
                Err(e) => error!("Error on message Next: {e}"),
                Ok(()) => model.app_state = AppState::Player(PlayerState::Playing),
            }
            None
        }
        Message::Previous => {
            match controls::play_previous(model) {
                Err(e) => error!("Error on message Previous: {e}"),
                Ok(()) => model.app_state = AppState::Player(PlayerState::Playing),
            }

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
                controls::volume_off(model);
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
                bar_state.input.push(c);

                if bar_state.bar_type == BarType::Search {
                    // Implement search
                    model.queue.set_filter(&bar_state.input);
                }
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

                    if bar_state.bar_type == BarType::Search {
                        model.queue.set_filter(&bar_state.input);
                    }
                }
            }
            None
        }
        Message::OpenHint => {
            model.popup = Some(Popup::Hint);
            None
        }
        Message::OpenSearchBar => {
            model.popup = Some(Popup::Bar(BarState {
                input: String::new(),
                bar_type: BarType::Search,
            }));
            None
        }
        Message::ClearSearch => {
            model.queue.set_filter("");
            let curr_idx = model.queue.get_current_idx();
            model.ui.song_queue_table.select(curr_idx);
            None
        }
        Message::PlayFromSearch(idx) => {
            model.queue.set_current_song_idx(idx);

            model.audio.sink.stop();

            if let Err(e) = controls::load_and_not_play(model) {
                log::error!("Failed to load song: {}", e);
            }

            match controls::play(model) {
                Ok(_) => model.app_state = AppState::Player(PlayerState::Playing),
                Err(e) => log::error!("Failed to play: {}", e),
            }

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
        return None;
    };

    let ret = match popup {
        Popup::Bar(bar) => match bar.bar_type {
            BarType::Command => handle_command(model),
            BarType::Search => {
                let target_index = model
                    .queue
                    .get_display_songs()
                    .first()
                    .map(|(real_idx, _)| *real_idx);

                model.popup = None;

                if let Some(idx) = target_index {
                    Some(Message::PlayFromSearch(idx))
                } else {
                    None
                }
            }
        },
        Popup::Hint => None,
    };

    if model.popup.is_some() {
        model.popup = None;
    }

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

fn handle_set_volume(argv: &[&str]) -> Option<Message> {
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
    fn test_state_transitions() {
        let mut model = Model::init(vec![]).unwrap();
        assert_eq!(model.app_state, AppState::Init);

        // Test TogglePlay dispatches to Play message when not playing
        let result = update(&mut model, Message::TogglePlay);
        assert_eq!(result, Some(Message::Play));
        assert_eq!(model.app_state, AppState::Init); // state unchanged, Play message needs to be handled

        // Test Pause message (transitions to Paused without needing files)
        let result = update(&mut model, Message::Pause);
        assert_eq!(result, None);
        assert_eq!(model.app_state, AppState::Player(PlayerState::Paused));

        // Test TogglePlay dispatches to Play when paused
        let result = update(&mut model, Message::TogglePlay);
        assert_eq!(result, Some(Message::Play));
        assert_eq!(model.app_state, AppState::Player(PlayerState::Paused)); // unchanged until Play handled

        // Manually transition to playing for subsequent tests
        model.app_state = AppState::Player(PlayerState::Playing);

        // Test TogglePlay dispatches to Pause when playing
        let result = update(&mut model, Message::TogglePlay);
        assert_eq!(result, Some(Message::Pause));

        // can't test Message::Play because no music file can be loaded during tests

        // open command bar
        let result = update(&mut model, Message::OpenCommandBar);
        assert_eq!(result, None);
        assert_eq!(model.app_state, AppState::Player(PlayerState::Playing)); // didn't change
        assert!(matches!(model.popup, Some(Popup::Bar(_))));
        if let Some(Popup::Bar(bar)) = &model.popup {
            assert_eq!(bar.bar_type, BarType::Command);
        } else {
            assert!(false);
        }

        // send char to popup
        let result = update(&mut model, Message::SendCharToPopup('x'));
        assert_eq!(result, None);
        assert_eq!(model.app_state, AppState::Player(PlayerState::Playing)); // didn't change
        assert!(matches!(model.popup, Some(Popup::Bar(_))));
        if let Some(Popup::Bar(bar)) = &model.popup {
            assert_eq!(bar.bar_type, BarType::Command);
            assert_eq!(bar.input, "x");
        } else {
            assert!(false);
        }

        // erase that char
        let result = update(&mut model, Message::EraseChar);
        assert_eq!(result, None);
        assert_eq!(model.app_state, AppState::Player(PlayerState::Playing)); // didn't change
        assert!(matches!(model.popup, Some(Popup::Bar(_))));
        if let Some(Popup::Bar(bar)) = &model.popup {
            assert_eq!(bar.bar_type, BarType::Command);
            assert!(bar.input.is_empty());
        } else {
            assert!(false);
        }

        // close the popup
        let result = update(&mut model, Message::ClosePopup);
        assert_eq!(result, None);
        assert_eq!(model.app_state, AppState::Player(PlayerState::Playing)); // didn't change
        assert!(model.popup.is_none());

        // open hints
        let result = update(&mut model, Message::OpenHint);
        assert_eq!(result, None);
        assert_eq!(model.app_state, AppState::Player(PlayerState::Playing)); // didn't change
        assert!(matches!(model.popup, Some(Popup::Hint)));

        // Playing -> Quit
        let result = update(&mut model, Message::Quit);
        assert_eq!(result, None);
        assert_eq!(model.app_state, AppState::Done);
    }
}
