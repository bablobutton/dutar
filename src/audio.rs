pub mod decoder;
pub mod error;

pub use decoder::{Decoder, TrackInfo, ReadOutcome};
pub use error::{CoreError, CoreResult};