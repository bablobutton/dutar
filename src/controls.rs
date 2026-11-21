use crate::{Message, Model, SkipDirection};
use color_eyre::{
    Result,
    eyre::{OptionExt, eyre},
};
use log::{debug, error, info};
use rodio::{Sink, decoder::DecoderBuilder, source::EmptyCallback};
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc::Sender;
use std::time::Duration;

pub fn play(model: &mut Model) -> Result<()> {
    if model.audio.sink.empty() {
        load_and_play(model)
    } else {
        model.audio.sink.play();
        Ok(())
    }
}

pub fn pause(model: &Model) {
    model.audio.sink.pause();
}

pub fn load_song(model: &mut Model) -> Result<()> {
    let song = model
        .queue
        .get_current_song()
        .ok_or_eyre("Error getting current song")?;
    debug!("Loading from {}", song.path.display());
    let file = File::open(&song.path)?;
    let source = DecoderBuilder::new()
        .with_seekable(true)
        .with_data(BufReader::new(file))
        .build()?;

    let sink = Sink::connect_new(model.audio.stream.mixer());
    sink.set_volume(model.audio.sink.volume());
    sink.pause();
    model.audio.sink = sink;
    model.audio.sink.append(source);
    model
        .audio
        .sink
        .append(create_callback_source_play_next(model.channel.tx.clone()));

    Ok(())
}

pub fn load_and_play(model: &mut Model) -> Result<()> {
    load_song(model)?;
    model.audio.sink.play();
    Ok(())
}

// rodio has no hook on sound end, so a callback source appended after the song notifies the event loop
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
    if get_current_duration(model) > Duration::from_secs(3) {
        set_current_duration(model, Duration::ZERO);
        model.audio.sink.play();
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

pub fn quadratic_skip(model: &Model) {
    let (Some(skip), Some(total)) = (&model.last_skip, get_current_song_total_duration(model))
    else {
        return;
    };
    let step = Duration::from_secs(skip.streak.pow(2) as u64).min(total / 10);
    let pos = get_current_duration(model);
    let target = match skip.direction {
        SkipDirection::Backwards => pos.saturating_sub(step),
        SkipDirection::Forwards => pos + step,
    };
    set_current_duration(model, target);
}

pub fn get_current_duration(model: &Model) -> Duration {
    model.audio.sink.get_pos()
}

pub fn set_current_duration(model: &Model, duration: Duration) {
    if let Err(e) = model.audio.sink.try_seek(duration) {
        error!("Error seeking to {duration:?}: {e}");
    }
}

pub fn get_current_song_total_duration(model: &Model) -> Option<Duration> {
    let metadata = model.queue.get_current_song()?.metadata.as_ref()?;
    Some(Duration::from_secs(metadata.duration_seconds))
}

pub fn goto_percent_current_song(model: &Model, position: u8) {
    if let Some(total) = get_current_song_total_duration(model) {
        let fraction = (position as f64 / 10.0).min(0.9);
        set_current_duration(model, total.mul_f64(fraction));
    }
}

pub fn change_volume(model: &mut Model, delta: f32) {
    set_volume(model, (get_volume(model) + delta).clamp(0.0, 1.0));
}

pub fn get_volume(model: &Model) -> f32 {
    model.audio.sink.volume()
}

pub fn set_volume(model: &mut Model, val: f32) {
    model.audio.sink.set_volume(val);
    model.saved_state.volume = val;
    debug!("Volume set to {val:.2}");
}
