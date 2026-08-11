use thiserror::Error;

#[derive(Error, Debug)]
pub enum FitError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),

    #[error("Compression failed: {0}")]
    CompressionFailed(String),

    #[error("Checksum mismatch: expected {expected:x}, got {actual:x}")]
    ChecksumMismatch { expected: u64, actual: u64 },

    #[error("Hash integrity check failed: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("Invalid magic header: expected {expected:?}, got {actual:?}")]
    InvalidMagic { expected: Vec<u8>, actual: Vec<u8> },

    #[error("Unsupported version: {0}")]
    UnsupportedVersion(u16),

    #[error("Cryptographic error: {0}")]
    Crypto(String),

    #[error("Invalid password or corrupted header")]
    InvalidPassword,

    #[error("Recovery failed: {0}")]
    RecoveryFailed(String),

    #[error("Security violation: {0}")]
    SecurityViolation(String),

    #[error("Archive format error: {0}")]
    ArchiveFormat(String),

    #[error("Plugin error: {0}")]
    PluginError(String),

    #[error("Operation cancelled")]
    Cancelled,
}

pub type FitResult<T> = Result<T, FitError>;
