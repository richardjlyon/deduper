//! Application error definitions.

use thiserror::Error;

// Define custom errors using the `thiserror::Error` derive macro
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Unsupported image type: {0}")]
    UnsupportedType(String),

    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("Different aspect ratios")]
    DifferentAspectRatio,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialise error: {0}")]
    SerialiseError(#[from] serde_json::Error),

    #[error("Invalid hash chunk size {0}: Should be between 0.0 and 1.0")]
    InvalidHashChunkSize(f32),

    #[error("Fjall error: {0}")]
    FjallError(#[from] fjall::Error),

    #[error("Exif error: {0}")]
    ExifError(#[from] exif::Error),

    #[error("Deserialise error")]
    DeserialiseError(serde_json::Error),
}
