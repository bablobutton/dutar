mod audio;
mod tui;

use color_eyre::Result;
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

#[derive(PartialEq)]
enum Message {
    TogglePlay,
    Quit,
}

fn main() -> Result<()> {
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
    let reader = BufReader::with_capacity(
        1024 * 1024 * 5,
        File::open("resources/hydrogen.mp3").expect("this file should exist"),
    );
    model.player.sink = rodio::play(model.player.stream.mixer(), reader).unwrap();
}
