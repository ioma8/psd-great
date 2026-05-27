use thiserror::Error;

/// Errors that can occur when working with PSD files
#[derive(Error, Debug)]
pub enum PsdError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid PSD format: {0}")]
    InvalidFormat(String),

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("Compression error: {0}")]
    Compression(String),

    #[error("Invalid color mode: {0}")]
    InvalidColorMode(u8),

    #[error("Invalid blend mode: {0}")]
    InvalidBlendMode(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("JPEG error: {0}")]
    JpegError(String),

    #[error("Invalid UTF-8")]
    InvalidUtf8,

    #[error("Invalid engine data: {0}")]
    InvalidEngineData(String),

    #[error("Invalid ASE format: {0}")]
    InvalidAse(String),

    #[error("Invalid ABR format: {0}")]
    InvalidAbr(String),

    #[error("Invalid CSH format: {0}")]
    InvalidCsh(String),
}

pub type Result<T> = std::result::Result<T, PsdError>;
