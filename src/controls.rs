use crate::{Message, Model, SkipDirection};
use color_eyre::{
    Result,
    eyre::{OptionExt, eyre},
};
use log::{debug, error, info};
use rodio::{Decoder, decoder::DecoderBuilder, source::EmptyCallback};
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc::Sender;
use std::time::Duration;

pub fn play(model: &mut Model) -> Result<()> {
    if model.audio.sink.len() > 0 {
        model.audio.sink.play();
    } else {
        load_and_play(model)?;
    }
    Ok(())
}

pub fn pause(model: &mut Model) {
    model.audio.sink.pause();
}

fn get_source(reader: BufReader<File>) -> Option<Decoder<BufReader<File>>> {
    DecoderBuilder::new()
        .with_seekable(true)
        .with_data(reader)
        .build()
        .ok()
}

fn load_song(model: &mut Model) -> Result<()> {
    let song = model
        .queue
        .get_current_song()
        .ok_or_eyre("Error getting current song")?;
    debug!("Loading from {} and playing", song.path.display());
    let file = File::open(song.path.as_path())?;
    let reader = BufReader::with_capacity(1024 * 1024 * 5, file);
    let source = get_source(reader).ok_or_eyre("Failed to decode")?;
    model.audio.sink.clear(); // clear from current sounds (and callbacks)
    model.audio.sink.append(source);
    model
        .audio
        .sink
        .append(create_callback_source_play_next(model.channel.tx.clone()));
    Ok(())
}

fn load_and_play(model: &mut Model) -> Result<()> {
    load_song(model)?;
    model.audio.sink.play(); // because sink.clear() pauses
    Ok(())
}

pub fn load_and_not_play(model: &mut Model) -> Result<()> {
    load_song(model)
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

pub fn play_next(model: &mut Model) -> Result<()> {
    model.queue.advance()?;
    while let Err(e) = load_and_play(model) {
        error!("Error playing next song: {e}");
        model.queue.remove_current_song();
        if model.queue.is_empty() {
            return Err(eyre!("Queue is empty"));
        }
    }
    Ok(())
}

pub fn play_previous(model: &mut Model) -> Result<()> {
    // if duration is > 3 sec, restart song.
    // otherwise, play previous
    let curr_duration = get_current_duration(model);
    if curr_duration > Duration::from_secs(3) {
        set_current_duration(model, Duration::ZERO);
        return Ok(());
    }

    model.queue.retreat()?;
    while let Err(e) = load_and_play(model) {
        error!("Error playing previous song: {e}");
        model.queue.remove_current_song();
        model.queue.retreat()?;
    }
    Ok(())
}

pub fn quadratic_skip(model: &mut Model) {
    let sink = &model.audio.sink;
    let mut curr_duration = sink.get_pos();

    if let Some(skip) = &model.last_skip {
        match skip.direction {
            SkipDirection::Backwards => {
                curr_duration =
                    curr_duration.saturating_sub(Duration::from_secs(skip.streak.pow(2) as u64));
            }
            SkipDirection::Forwards => {
                curr_duration =
                    curr_duration.saturating_add(Duration::from_secs(skip.streak.pow(2) as u64));
            }
        }
    }

    if let Err(e) = sink.try_seek(curr_duration) {
        error!("{e}");
    }
}

/* pub fn forward_seconds(model: &mut Model, seconds: u64) {
    let sink = &model.audio.sink;
    let curr_duration = sink.get_pos();
    let skip_seconds = Duration::from_secs(seconds);
    let curr_duration = curr_duration.saturating_add(skip_seconds);
    if let Err(e) = sink.try_seek(curr_duration) {
        error!("{e}");
    }
    debug!("Forward {seconds} seconds, current_duration={curr_duration:?}");
}

pub fn backward_seconds(model: &mut Model, seconds: u64) {
    let sink = &model.audio.sink;
    let curr_duration = sink.get_pos();
    let skip_seconds = Duration::from_secs(seconds);
    let curr_duration = curr_duration.saturating_sub(skip_seconds);
    if let Err(e) = sink.try_seek(curr_duration) {
        error!("{e}");
    }
    debug!("Backward {seconds} seconds, current_duration={curr_duration:?}");
} */

pub fn get_current_duration(model: &Model) -> Duration {
    model.audio.sink.get_pos()
}

pub fn set_current_duration(model: &Model, duration: Duration) {
    let sink = &model.audio.sink;
    if let Err(e) = sink.try_seek(duration) {
        error!("{e}");
    }
    debug!("Set current duration to {duration:?}");
}

pub fn get_current_song_total_duration(model: &Model) -> Option<Duration> {
    if let Some(song) = model.queue.get_current_song() {
        if let Some(metadata) = &song.metadata {
            return Some(Duration::from_secs(metadata.duration_seconds));
        }
    }
    None
}

pub fn goto_percent_current_song(model: &Model, position: u8) {
    if let Some(total_duration) = get_current_song_total_duration(model) {
        let percentage_position = (position as f64 / 10.0).clamp(0.0, 0.9);
        let desired_position =
            Duration::from_secs_f64(total_duration.as_secs_f64() * percentage_position);

        let sink = &model.audio.sink;
        if let Err(e) = sink.try_seek(desired_position) {
            error!("{e}");
        }
    }
}

pub fn volume_up(model: &mut Model, val: f32) {
    let sink = &model.audio.sink;
    let curr_vol = sink.volume();
    let new_vol = (curr_vol + val).clamp(0.0, 1.0);
    sink.set_volume(new_vol);
    model.saved_state.volume = new_vol;
    debug!("Volume increased to {new_vol:.2}");
}

pub fn volume_down(model: &mut Model, val: f32) {
    let sink = &model.audio.sink;
    let curr_vol = sink.volume();
    let new_vol = (curr_vol - val).clamp(0.0, 1.0);
    sink.set_volume(new_vol);
    model.saved_state.volume = new_vol;
    debug!("Volume decreased to {new_vol:.2}");
}

pub fn get_volume(model: &Model) -> f32 {
    model.audio.sink.volume()
}

pub fn volume_off(model: &mut Model) {
    model.audio.sink.set_volume(0f32);
}

pub fn set_volume(model: &mut Model, val: f32) {
    model.audio.sink.set_volume(val);
    model.saved_state.volume = val;
}
