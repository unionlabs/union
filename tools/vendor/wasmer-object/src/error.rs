use object::write::Error as ObjectWriteError;
use thiserror::Error;

/// The Object error can occur when creating an object file
/// from a `Compilation`.
#[derive(Error, Debug)]
pub enum ObjectError {
    /// The object was provided a not-supported binary format
    #[error("Binary format {0} not supported")]
    UnsupportedBinaryFormat(String),
    /// The object was provided a not-supported architecture
    #[error("Architecture {0} not supported")]
    UnsupportedArchitecture(String),
    /// The object was provided an unknown endianness
    #[error("Unknown Endianness")]
    UnknownEndianness,
    /// The object was provided a not-supported architecture
    #[error("Error when writing the object: {0}")]
    Write(#[from] ObjectWriteError),
}
