use crate::{Model, PlayerState, State};
use color_eyre::{Result, eyre::eyre};
use log::{debug, error};
use rodio::{Decoder, decoder::DecoderBuilder};
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

pub fn toggle_play(model: &mut Model) {
    match model.state {
        State::Init => match play_first_time(model) {
            Ok(()) => model.state = State::Player(PlayerState::Playing),
            Err(e) => {
                error!("Error opening first file: {e}");
            }
        },
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

fn get_source(reader: BufReader<File>) -> Option<Decoder<BufReader<File>>> {
    DecoderBuilder::new()
        .with_seekable(true)
        .with_data(reader)
        .build()
        .ok()
}

fn play_first_time(model: &mut Model) -> Result<()> {
    debug!("playing first time, loading from file");
    let file = File::open("resources/hydrogen.mp3").expect("this file should exist");
    let reader = BufReader::with_capacity(1024 * 1024 * 5, file);
    let source = get_source(reader).ok_or_else(|| eyre!("Failed to build a decoder"))?;
    model.player.sink.append(source);
    Ok(())
}

pub fn forward_seconds(model: &mut Model, seconds: u64) {
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

pub fn backward_seconds(model: &mut Model, seconds: u64) {
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
