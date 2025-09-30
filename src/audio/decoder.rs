use std::fs::File;
use std::path::Path;
use symphonia::core::codecs::{CODEC_TYPE_NULL, Decoder as SymDecoder, DecoderOptions};
use symphonia::core::formats::FormatReader;
use symphonia::core::{
    formats::FormatOptions, io::MediaSourceStream, meta::MetadataOptions, probe::Hint,
};

use super::error::*;

#[derive(Debug)]
pub struct TrackInfo {
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_frames: Option<u64>,
    pub codec: String,
    pub container: String,
}

#[derive(Debug)]
pub struct ReadOutcome {
    pub samples_written: usize,
    pub eof: bool,
}

pub struct Decoder {
    track_info: TrackInfo,
    track_id: u32,
    format: Box<dyn FormatReader>,
    decoder: Box<dyn SymDecoder>,
}

impl Decoder {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, CoreError> {
        let path = path.as_ref();
        let mut hint = Hint::new();
        let container_guess = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_else(|| "unknown".to_string());
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let file = File::open(path)?;
        let source = Box::new(file) as Box<dyn symphonia::core::io::MediaSource>;
        let mss = MediaSourceStream::new(source, Default::default());

        let probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|_| CoreError::ProbeFailed)?;

        let mut format = probed.format;

        let track = format
            .tracks()
            .iter()
            .find(|t| {
                let cp = &t.codec_params;
                cp.codec != CODEC_TYPE_NULL && cp.sample_rate.is_some() && cp.channels.is_some()
            })
            .ok_or(CoreError::NoAudioTrack)?;

        let sample_rate = track
            .codec_params
            .sample_rate
            .ok_or_else(|| CoreError::DecoderInit("missing sample rate".into()))?;

        let ch_map = track
            .codec_params
            .channels
            .ok_or_else(|| CoreError::DecoderInit("missing channels".into()))?;
        let channels = ch_map.count() as u16;
        if channels == 0 {
            return Err(CoreError::DecoderInit("zero channels".into()));
        }
        if channels > 2 {
            return Err(CoreError::UnsupportedChannels(channels));
        }

        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| CoreError::DecoderInit(e.to_string()))?;

        let codec = format!("{:?}", track.codec_params.codec);
        let duration_frames = track.codec_params.n_frames;

        let track_info = TrackInfo {
            sample_rate,
            channels,
            duration_frames,
            codec,
            container: container_guess,
        };

        Ok(Self {
            track_info,
            track_id: track.id,
            format,
            decoder,
        })
    }

    pub fn info(&self) -> &TrackInfo {
        &self.track_info
    }

    pub fn read_into(&mut self, _dst: &mut [f32]) -> Result<ReadOutcome, CoreError> {
        unimplemented!("Coming soon...")
    }
}

#[cfg(test)]
mod tests {
    use super::Decoder;

    #[test]
    fn print_track_info() -> Result<(), Box<dyn std::error::Error>> {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join("resources").join("hydrogen.mp3");

        let dec = Decoder::open(&path)?;
        let info = dec.info();

        println!("container: {}", info.container);
        println!("codec:     {}", info.codec);
        println!("rate:      {} Hz", info.sample_rate);
        println!("channels:  {}", info.channels);
        println!("duration:  {:?}", info.duration_frames);

        Ok(())
    }
}
