use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to probe audio format")]
    ProbeFailed,
    #[error("No audio track found")]
    NoAudioTrack,
    #[error("Unsupported number of channels: {0}")]
    UnsupportedChannels(u16),
    #[error("Unsupported sample format: {0}")]
    UnsupportedSampleFormat(&'static str),
    #[error("Decoder initialization failed: {0}")]
    DecoderInit(String),
    #[error("Decode error: {0}")]
    DecodePacket(String),
}

pub type CoreResult<T> = Result<T, CoreError>;
