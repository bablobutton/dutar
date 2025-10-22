use crate::{Message, Model, PlayerState, State};
use color_eyre::{Result, eyre::OptionExt};
use log::{debug, error, info};
use rodio::{Decoder, decoder::DecoderBuilder, source::EmptyCallback};
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc::Sender;
use std::time::Duration;

pub fn toggle_play(model: &mut Model) {
    match model.state {
        State::Init => match load_and_play(model) {
            Ok(()) => model.state = State::Player(PlayerState::Playing),
            Err(e) => {
                error!("Error loading a song: {e}");
            }
        },
        State::Player(PlayerState::Paused) => {
            model.audio.sink.play();
            model.state = State::Player(PlayerState::Playing)
        }
        State::Player(PlayerState::Playing) => {
            model.audio.sink.pause();
            model.state = State::Player(PlayerState::Paused)
        }
        _ => {}
    }
}

fn get_source(reader: BufReader<File>) -> Option<Decoder<BufReader<File>>> {
    DecoderBuilder::new()
        .with_seekable(true)
        .with_data(reader)
        .build()
        .ok()
}

fn load_and_play(model: &mut Model) -> Result<()> {
    let song = model
        .queue
        .get_current_song()
        .ok_or_eyre("No song to play")?;
    debug!("Loading from {} and playing", song.path.display());
    let file = File::open(&song.path.as_path())?;
    let reader = BufReader::with_capacity(1024 * 1024 * 5, file);
    let source = get_source(reader).ok_or_eyre("Failed to decode")?;
    model.audio.sink.clear(); // clear from current sounds (and callbacks)
    model.audio.sink.append(source);
    model
        .audio
        .sink
        .append(create_callback_source_play_next(model.channel.tx.clone()));
    model.audio.sink.play(); // because clear() paused
    Ok(())
}

// rodio does not support hooks on sound end,
// instead they allow to append a "callback" source to sink
// so when the sound has ended, the callback is executed
// to notify the event loop to play next song
// https://github.com/RustAudio/rodio/issues/651
fn create_callback_source_play_next(tx: Sender<Message>) -> EmptyCallback {
    EmptyCallback::new(Box::new(move || {
        info!("Song has ended, playing next one");
        if let Err(e) = tx.send(Message::Next) {
            error!("Error sending message to play next over channel: {e}");
        }
    }))
}

pub fn play_next(model: &mut Model) {
    model.queue.advance();
    if let Err(e) = load_and_play(model) {
        error!("Error playing next song: {e}");
    }
}

pub fn play_previous(model: &mut Model) {
    model.queue.retreat();
    if let Err(e) = load_and_play(model) {
        error!("Error playing previous song: {e}");
    }
}

pub fn forward_seconds(model: &mut Model, seconds: u64) {
    if model.state == State::Player(PlayerState::Playing)
        || model.state == State::Player(PlayerState::Paused)
    {
        let sink = &model.audio.sink;
        let curr_duration = sink.get_pos();
        let skip_seconds = Duration::from_secs(seconds);
        let curr_duration = curr_duration.saturating_add(skip_seconds);
        if let Err(e) = sink.try_seek(curr_duration) {
            error!("{e}");
        }
        debug!("Forward {seconds} seconds, current_duration={curr_duration:?}");
    }
}

pub fn backward_seconds(model: &mut Model, seconds: u64) {
    if model.state == State::Player(PlayerState::Playing)
        || model.state == State::Player(PlayerState::Paused)
    {
        let sink = &model.audio.sink;
        let curr_duration = sink.get_pos();
        let skip_seconds = Duration::from_secs(seconds);
        let curr_duration = curr_duration.saturating_sub(skip_seconds);
        if let Err(e) = sink.try_seek(curr_duration) {
            error!("{e}");
        }
        debug!("Backward {seconds} seconds, current_duration={curr_duration:?}");
    }
}
