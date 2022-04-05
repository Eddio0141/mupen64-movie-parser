//! Error types for the M64 parser.

use std::io;

use thiserror::Error;

/// All possible M64 parsing errors.
#[derive(Debug, Error)]
pub enum M64ParseError {
    /// File signature didn't match.
    #[error("Invalid file signature, expected [4D 36 34 1A], got {0:X?}")]
    InvalidSignature(Vec<u8>),
    /// File version number wasn't 3.
    #[error("Invalid version, expected 3, got {0}")]
    InvalidVersion(u32),
    /// Reserved bytes weren't zero.
    #[error("Reserved data is not all zero at offset 0x{0:X?}")]
    ReservedNotZero(usize),
    /// There was not enough bytes to make up for a field.
    #[error(
        "Not enough bytes to read to make up for the {name} field, requires {requires} more bytes"
    )]
    NotEnoughBytes { name: &'static str, requires: usize },
    /// The input data wasn't 4 bytes aligned.
    #[error("Input data is not 4 bytes aligned, final input data size is {0} bytes")]
    InputNot4BytesAligned(usize),
    /// Invalid movie start type.
    #[error("Invalid movie start type")]
    InvalidMovieStartType,
    /// Invalid UTF-8 string.
    #[error("Invalid UTF-8 string at offset 0x{0:X?}")]
    InvalidString(usize),
    /// Io error.
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// M64 field names used for the error messages.
pub enum FieldName {
    Uid,
    ViFrames,
}
