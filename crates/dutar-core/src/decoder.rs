
#[derive(Debug)]
pub struct TrackInfo {
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_frames: Option<u64>,
    pub code: String,
    pub container: String,
}
#[derive(Debug)]
pub struct ReadOutcome {
    pub samples_written: usize,
    pub eof: bool,
}

pub struct Decoder {

}

impl Decoder {
    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self, CoreError> {
        unimplemented!("Coming soon...")
    }
    pub fn inf(&self) -> &TrackInfo {
        unimplemented!("Coming soon...")
    }

    pub fn read_into(dst: &mut [f32]) -> Result<ReadOutcome, CoreError> {
        unimplemented!("Coming soon...")
    }

}