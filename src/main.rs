mod audio;
mod tui;

use color_eyre::Result;
use ratatui::Frame;
use ratatui::widgets::{Paragraph};
use ratatui::crossterm::event;
use std::time::Duration;

#[derive(Debug, Default)]
struct Model {
    state: State,
}

#[derive(Debug, PartialEq, Eq)]
enum Player {
    Playing,
    Paused,
}

// state tracker of the app
#[derive(Debug, Default, PartialEq, Eq)]
enum State {
    #[default]
    Init,
    Player(Player), // nested enum for playing screen
    Done,
}

#[derive(PartialEq)]
enum Message {
    TogglePlay,
    Quit,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let mut model = Model::default();

    while model.state != State::Done {
        terminal.draw(|frame| view(&mut model, frame))?;
        let mut msg = handle_event(&model)?;

        while msg.is_some() {
            msg = update(&mut model, msg.unwrap());
        }
    }

    ratatui::restore();
    Ok(())
}

fn handle_event(model: &Model) -> Result<Option<Message>> {
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
        _ => {
            println!("NOIMPL");
            None
        }
    }
}

fn view(model: &mut Model, frame: &mut Frame) {
    let current_state_string = match model.state {
        State::Init => "init",
        State::Player(Player::Playing) => "playing",
        State::Player(Player::Paused) => "paused",
        State::Done => "done",
    };
    frame.render_widget(Paragraph::new(current_state_string), frame.area());
}

fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::TogglePlay => {
            toggle_play(model);
            None
        },
        Message::Quit => {
            model.state = State::Done;
            None
        }
    }
}

fn toggle_play(model: &mut Model) {
    match model.state {
        State::Init => {
            model.state = State::Player(Player::Playing)
        },
        State::Player(Player::Playing) => {
            model.state = State::Player(Player::Paused)
        }, 
        State::Player(Player::Paused) => {
            model.state = State::Player(Player::Playing)
        }, 
        _ => {}
    }
}
