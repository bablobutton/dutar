mod audio;
mod logging;
mod tui;

use color_eyre::Result;
use log::{debug, error};
use ratatui::crossterm::event;
use rodio::{OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
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
    match msg {
        Message::TogglePlay => {
            toggle_play(model);
            None
        }
        Message::Quit => {
            model.state = State::Done;
            None
        }
        Message::SkipForward => {
            forward_seconds(model, 5);
            None
        }
        Message::SkipBack => {
            backward_seconds(model, 5);
            None
        }
        Message::Next => {
            todo!();
        }
        Message::Previous => {
            todo!();
        }
    }
}

fn toggle_play(model: &mut Model) {
    match model.state {
        State::Init => {
            play_first_time(model);
            model.state = State::Player(PlayerState::Playing)
        }
        State::Player(PlayerState::Paused) => {
            model.player.sink.play();
            model.state = State::Player(PlayerState::Playing)
        }
        State::Player(PlayerState::Playing) => {
            model.player.sink.pause();
            model.state = State::Player(PlayerState::Paused)
        }
        _ => {}
    }
}

fn play_first_time(model: &mut Model) {
    debug!("playing first song, loading from file");
    let reader = BufReader::with_capacity(
        1024 * 1024 * 5, // 5 MiB
        File::open("resources/hydrogen.mp3").expect("this file should exist"),
    );
    model.player.sink = rodio::play(model.player.stream.mixer(), reader).unwrap();
}

fn forward_seconds(model: &mut Model, seconds: u64) {
    if model.state == State::Player(PlayerState::Playing)
        || model.state == State::Player(PlayerState::Paused)
    {
        let sink = &model.player.sink;
        let curr_duration = sink.get_pos();
        let skip_seconds = Duration::from_secs(seconds);
        let curr_duration = curr_duration.saturating_add(skip_seconds);
        if let Err(e) = sink.try_seek(curr_duration) {
            error!("{e}");
        }
        debug!("forward {seconds} seconds, current_duration={curr_duration:?}");
    }
}

fn backward_seconds(model: &mut Model, seconds: u64) {
    if model.state == State::Player(PlayerState::Playing)
        || model.state == State::Player(PlayerState::Paused)
    {
        let sink = &model.player.sink;
        let curr_duration = sink.get_pos();
        let skip_seconds = Duration::from_secs(seconds);
        let curr_duration = curr_duration.saturating_sub(skip_seconds);
        if let Err(e) = sink.try_seek(curr_duration) {
            error!("{e}");
        }
        debug!("backward {seconds} seconds, current_duration={curr_duration:?}");
    }
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
