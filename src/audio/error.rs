use thiserror::Error;

pub type CoreResult<T> = Result<T, CoreError>;
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("I/O error")]
    Io(#[from] std::io::Error),

    #[error("Failed to probe audio format")]
    ProbeFailed,

    #[error("No audio track found in the container")]
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
