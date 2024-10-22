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
    // #[error("Unknown error")]
    // Unknown,
}
