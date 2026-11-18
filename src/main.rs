mod controls;
mod db;
mod logging;
mod queue;
mod tui;
mod utils;

use color_eyre::{Result, eyre::WrapErr};
use db::DB;
use log::{debug, error};
use queue::SongQueue;
use ratatui::crossterm::event::{self, KeyModifiers};
use ratatui::widgets::TableState;
use rodio::{OutputStream, Sink};
use std::env;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::time::{Duration, Instant};

struct Model {
    app_state: AppState,
    search: Option<String>,
    show_hints: bool,
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
            search: None,
            show_hints: false,
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
    OpenSafeExit,
    CloseSearch,
    SubmitSearch,
    SendCharToSearch(char),
    VolumeUp,
    VolumeDown,
    ToggleMute,
    Mute,
    Unmute,
    EraseChar,
    OpenSearch,
    OpenHints,
    CloseHints,
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
    match &model.search {
        Some(_) => match key.code {
            _ if ctrl_c => Some(Message::CloseSearch),
            event::KeyCode::Enter => Some(Message::SubmitSearch),
            event::KeyCode::Esc => Some(Message::CloseSearch),
            event::KeyCode::Char(c) => Some(Message::SendCharToSearch(c)),
            event::KeyCode::Backspace => Some(Message::EraseChar),
            _ => None,
        },
        None if model.show_hints
            && matches!(key.code, event::KeyCode::Enter | event::KeyCode::Esc) =>
        {
            Some(Message::CloseHints)
        }
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
        event::KeyCode::Char('=' | '+') => Some(Message::VolumeUp),
        event::KeyCode::Char('-' | '_') => Some(Message::VolumeDown),
        event::KeyCode::Char('?') => Some(Message::OpenHints),
        event::KeyCode::Char('/') => Some(Message::OpenSearch),
        event::KeyCode::Char(ch @ '0'..='9') => Some(Message::JumpTo(ch as u8 - b'0')),
        _ => None,
    }
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
        Message::OpenHints => {
            model.show_hints = true;
            None
        }
        Message::CloseHints => {
            model.show_hints = false;
            None
        }
        Message::OpenSearch => {
            model.search = Some(String::new());
            None
        }
        Message::CloseSearch => {
            model.search = None;
            None
        }
        Message::SubmitSearch => {
            model.search = None;
            model
                .queue
                .get_display_songs()
                .first()
                .map(|(idx, _)| Message::PlayFromSearch(*idx))
        }
        Message::SendCharToSearch(c) => {
            if let Some(input) = &mut model.search {
                input.push(c);
                model.queue.set_filter(input);
            }
            None
        }
        Message::EraseChar => {
            if let Some(input) = &mut model.search
                && input.pop().is_some()
            {
                model.queue.set_filter(input);
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
        "Updated state [{:?}], search [{:?}], Message [{:?}]",
        model.app_state, model.search, ret
    );
    ret
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

        let result = update(&mut model, Message::OpenSearch);
        assert_eq!(result, None);
        assert_eq!(model.search.as_deref(), Some(""));

        let result = update(&mut model, Message::SendCharToSearch('x'));
        assert_eq!(result, None);
        assert_eq!(model.search.as_deref(), Some("x"));

        let result = update(&mut model, Message::EraseChar);
        assert_eq!(result, None);
        assert_eq!(model.search.as_deref(), Some(""));

        let result = update(&mut model, Message::CloseSearch);
        assert_eq!(result, None);
        assert!(model.search.is_none());

        let result = update(&mut model, Message::OpenHints);
        assert_eq!(result, None);
        assert!(model.show_hints);

        let result = update(&mut model, Message::CloseHints);
        assert_eq!(result, None);
        assert!(!model.show_hints);
        assert_eq!(model.app_state, AppState::Player(PlayerState::Playing));

        // Playing -> Quit
        let result = update(&mut model, Message::Quit);
        assert_eq!(result, None);
        assert_eq!(model.app_state, AppState::Done);
    }
}
