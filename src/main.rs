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
use ratatui::crossterm::event::{self, KeyModifiers};
use ratatui::widgets::TableState;
use rodio::{OutputStream, Sink};
use std::env;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::time::{Duration, Instant};

struct Model {
    app_state: AppState,
    popup: Option<Popup>,
    queue_table: TableState,
    saved_state: SavedState,
    audio: Audio,
    queue: SongQueue,
    channel: Channel,
    db: DB,
    safe_exit_expires_at: Option<Instant>,
    last_skip: Option<Skip>,
}

impl Model {
    pub fn init(args: Vec<String>) -> Result<Model> {
        let stream = rodio::OutputStreamBuilder::open_default_stream()?;
        let sink = rodio::Sink::connect_new(stream.mixer());
        let (tx, rx): (Sender<Message>, Receiver<Message>) = channel();
        let db = DB::new()?;
        let mut saved_state = db.read_saved_state().wrap_err("DB read saved state")?;

        let queue = if let [_, path] = args.as_slice() {
            let q = SongQueue::open_path(path).wrap_err("Error loading songs")?;
            debug!("Successfully loaded songs from {path}");
            saved_state = SavedState {
                volume: saved_state.volume,
                ..Default::default()
            };
            q
        } else {
            SongQueue::restore_or_default(db.read_queue_songs_paths())
        };

        let mut model = Model {
            app_state: AppState::Init,
            queue_table: TableState::new(),
            saved_state,
            popup: None,
            audio: Audio {
                _stream: stream,
                sink,
            },
            queue,
            channel: Channel { rx, tx },
            db,
            safe_exit_expires_at: None,
            last_skip: None,
        };

        model.restore_saved_state();
        Ok(model)
    }

    fn restore_saved_state(&mut self) {
        controls::set_volume(self, self.saved_state.volume);
        self.queue
            .set_current_song_idx(self.saved_state.current_song_index);
        if let Err(e) = controls::load_song(self) {
            error!("Error loading song on init: {e}");
            return;
        }
        controls::set_current_duration(self, self.saved_state.current_duration);
    }

    fn update_saved_state(&mut self) {
        self.saved_state.current_song_index = self.queue.get_current_idx().unwrap_or(0);
        self.saved_state.current_duration = controls::get_current_duration(self);
    }

    fn safe_exit_active(&self) -> bool {
        self.safe_exit_expires_at
            .is_some_and(|t| Instant::now() <= t)
    }
}

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
    Player(PlayerState),
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

#[derive(Debug)]
struct Skip {
    direction: SkipDirection,
    last_fired: Instant,
    last_streak_updated: Instant,
    streak: u32,
}

#[derive(Debug, PartialEq)]
enum SkipDirection {
    Backwards,
    Forwards,
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
    OpenSafeExit,
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
    JumpTo(u8),
}

fn main() -> Result<()> {
    logging::init();
    color_eyre::install()?;
    let mut model = Model::init(env::args().collect())?;
    let result = run(&mut model);
    ratatui::restore();
    result?;

    model.update_saved_state();
    model.db.write_saved_state(&model.saved_state)?;
    model.db.write_queue_songs(&model.queue)?;
    Ok(())
}

fn run(model: &mut Model) -> Result<()> {
    let mut terminal = ratatui::init();
    while model.app_state != AppState::Done {
        terminal.draw(|frame| tui::render(model, frame))?;
        let mut msg = handle_event(model)?;
        while let Some(m) = msg {
            msg = update(model, m);
        }
    }
    Ok(())
}

fn handle_event(model: &Model) -> Result<Option<Message>> {
    match model.channel.rx.recv_timeout(Duration::from_millis(10)) {
        Ok(msg) => {
            debug!("Received message over channel: {msg:?}");
            return Ok(Some(msg));
        }
        Err(RecvTimeoutError::Timeout) => {}
        Err(err) => error!("Error receiving channel message: {err}"),
    }

    if event::poll(Duration::from_millis(90))?
        && let event::Event::Key(key) = event::read()?
        && key.kind == event::KeyEventKind::Press
    {
        return Ok(handle_key(key, model));
    }

    Ok(None)
}

fn handle_key(key: event::KeyEvent, model: &Model) -> Option<Message> {
    let ctrl_c =
        key.code == event::KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL);
    match &model.popup {
        Some(Popup::Bar(_)) => match key.code {
            _ if ctrl_c => Some(Message::ClosePopup),
            event::KeyCode::Enter => Some(Message::PopupSubmit),
            event::KeyCode::Esc => Some(Message::ClosePopup),
            event::KeyCode::Char(c) => Some(Message::SendCharToPopup(c)),
            event::KeyCode::Backspace => Some(Message::EraseChar),
            _ => None,
        },
        Some(Popup::Hint) => match key.code {
            event::KeyCode::Enter | event::KeyCode::Esc => Some(Message::ClosePopup),
            _ => handle_hotkey(key.code),
        },
        None if ctrl_c && model.safe_exit_active() => Some(Message::Quit),
        None if ctrl_c => Some(Message::OpenSafeExit),
        None if key.code == event::KeyCode::Esc => Some(Message::ClearSearch),
        None => handle_hotkey(key.code),
    }
}

fn handle_hotkey(keycode: event::KeyCode) -> Option<Message> {
    match keycode {
        event::KeyCode::Char('k') => Some(Message::TogglePlay),
        event::KeyCode::Char('l') => Some(Message::SkipForward),
        event::KeyCode::Char('j') => Some(Message::SkipBack),
        event::KeyCode::Char('n') => Some(Message::Next),
        event::KeyCode::Char('p') => Some(Message::Previous),
        event::KeyCode::Char('m') => Some(Message::ToggleMute),
        event::KeyCode::Char(':') => Some(Message::OpenCommandBar),
        event::KeyCode::Char('=' | '+') => Some(Message::VolumeUp),
        event::KeyCode::Char('-' | '_') => Some(Message::VolumeDown),
        event::KeyCode::Char('?') => Some(Message::OpenHint),
        event::KeyCode::Char('/') => Some(Message::OpenSearchBar),
        event::KeyCode::Char(ch @ '0'..='9') => Some(Message::JumpTo(ch as u8 - b'0')),
        _ => None,
    }
}

fn open_bar(model: &mut Model, bar_type: BarType) {
    model.popup = Some(Popup::Bar(BarState {
        input: String::new(),
        bar_type,
    }));
}

fn set_playing(model: &mut Model, result: Result<()>) {
    match result {
        Ok(()) => model.app_state = AppState::Player(PlayerState::Playing),
        Err(e) => error!("Error starting playback: {e}"),
    }
}

fn update(model: &mut Model, msg: Message) -> Option<Message> {
    let ret = match msg {
        Message::TogglePlay => {
            if model.app_state == AppState::Player(PlayerState::Playing) {
                Some(Message::Pause)
            } else {
                Some(Message::Play)
            }
        }
        Message::Play => {
            let result = controls::play(model);
            set_playing(model, result);
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
            handle_skip(model, SkipDirection::Forwards);
            None
        }
        Message::SkipBack => {
            handle_skip(model, SkipDirection::Backwards);
            None
        }
        Message::Next => {
            let result = controls::play_next(model);
            set_playing(model, result);
            None
        }
        Message::Previous => {
            let result = controls::play_previous(model);
            set_playing(model, result);
            None
        }
        Message::VolumeUp => {
            controls::change_volume(model, 0.05);
            None
        }
        Message::VolumeDown => {
            controls::change_volume(model, -0.05);
            None
        }
        Message::SetVolume(vol) => {
            controls::set_volume(model, vol.min(100) as f32 / 100.0);
            None
        }
        Message::ToggleMute => {
            if controls::get_volume(model) == 0.0 {
                Some(Message::Unmute)
            } else {
                Some(Message::Mute)
            }
        }
        Message::Mute => {
            model.audio.sink.set_volume(0.0);
            None
        }
        Message::Unmute => {
            controls::set_volume(model, model.saved_state.volume);
            None
        }
        Message::OpenCommandBar => {
            open_bar(model, BarType::Command);
            None
        }
        Message::OpenSearchBar => {
            open_bar(model, BarType::Search);
            None
        }
        Message::OpenHint => {
            model.popup = Some(Popup::Hint);
            None
        }
        Message::ClosePopup => {
            model.popup = None;
            None
        }
        Message::PopupSubmit => handle_popup_submit(model),
        Message::SendCharToPopup(c) => {
            if let Some(Popup::Bar(bar)) = &mut model.popup {
                bar.input.push(c);
                if bar.bar_type == BarType::Search {
                    model.queue.set_filter(&bar.input);
                }
            }
            None
        }
        Message::EraseChar => {
            if let Some(Popup::Bar(bar)) = &mut model.popup
                && bar.input.pop().is_some()
                && bar.bar_type == BarType::Search
            {
                model.queue.set_filter(&bar.input);
            }
            None
        }
        Message::OpenSafeExit => {
            model.safe_exit_expires_at = Some(Instant::now() + Duration::from_millis(1250));
            None
        }
        Message::ClearSearch => {
            model.queue.set_filter("");
            None
        }
        Message::PlayFromSearch(idx) => {
            model.queue.set_current_song_idx(idx);
            let result = controls::load_and_play(model);
            set_playing(model, result);
            None
        }
        Message::JumpTo(place) => {
            controls::goto_percent_current_song(model, place);
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
    let Some(Popup::Bar(bar)) = model.popup.take() else {
        return None;
    };
    match bar.bar_type {
        BarType::Command => parse_command(&bar.input),
        BarType::Search => model
            .queue
            .get_display_songs()
            .first()
            .map(|(idx, _)| Message::PlayFromSearch(*idx)),
    }
}

fn parse_command(input: &str) -> Option<Message> {
    let mut argv = input.split_whitespace();
    let cmd = argv.next()?;
    let mut arg = || argv.next().and_then(|a| a.parse().ok());
    match cmd {
        "next" => Some(Message::Next),
        "prev" => Some(Message::Previous),
        "toggleplay" => Some(Message::TogglePlay),
        "play" => Some(Message::Play),
        "pause" => Some(Message::Pause),
        "setv" => arg().map(Message::SetVolume),
        "quit" | "q" => Some(Message::Quit),
        "mute" => Some(Message::Mute),
        "unmute" => Some(Message::Unmute),
        "togglemute" => Some(Message::ToggleMute),
        "jump" => arg().map(Message::JumpTo),
        _ => {
            warn!("Unknown command: {cmd}");
            None
        }
    }
}

fn handle_skip(model: &mut Model, direction: SkipDirection) {
    const PRESS_GRACE: Duration = Duration::from_millis(250);
    const STREAK_LEVELUP_COOLDOWN: Duration = Duration::from_secs(1);
    const MAX_STREAK: u32 = 16;

    let now = Instant::now();
    match &mut model.last_skip {
        Some(skip)
            if skip.direction == direction && now.duration_since(skip.last_fired) < PRESS_GRACE =>
        {
            if now.duration_since(skip.last_streak_updated) >= STREAK_LEVELUP_COOLDOWN {
                skip.streak = (skip.streak + 1).min(MAX_STREAK);
                skip.last_streak_updated = now;
            }
            skip.last_fired = now;
        }
        _ => {
            model.last_skip = Some(Skip {
                direction,
                last_fired: now,
                last_streak_updated: now,
                streak: 1,
            })
        }
    }

    controls::quadratic_skip(model);
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
        let Some(Popup::Bar(bar)) = &model.popup else {
            panic!("expected bar popup");
        };
        assert_eq!(bar.bar_type, BarType::Command);

        // send char to popup
        let result = update(&mut model, Message::SendCharToPopup('x'));
        assert_eq!(result, None);
        assert_eq!(model.app_state, AppState::Player(PlayerState::Playing)); // didn't change
        let Some(Popup::Bar(bar)) = &model.popup else {
            panic!("expected bar popup");
        };
        assert_eq!(bar.bar_type, BarType::Command);
        assert_eq!(bar.input, "x");

        // erase that char
        let result = update(&mut model, Message::EraseChar);
        assert_eq!(result, None);
        assert_eq!(model.app_state, AppState::Player(PlayerState::Playing)); // didn't change
        let Some(Popup::Bar(bar)) = &model.popup else {
            panic!("expected bar popup");
        };
        assert_eq!(bar.bar_type, BarType::Command);
        assert!(bar.input.is_empty());

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
