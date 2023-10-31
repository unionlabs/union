//! The WebAssembly possible errors
use std::io;
use thiserror::Error;
use wasmer_compiler::CompileError;
use wasmer_types::ExternType;

/// The Serialize error can occur when serializing a
/// compiled Module into a binary.
#[derive(Error, Debug)]
pub enum SerializeError {
    /// An IO error
    #[error(transparent)]
    Io(#[from] io::Error),
    /// A generic serialization error
    #[error("{0}")]
    Generic(String),
}

/// The Deserialize error can occur when loading a
/// compiled Module from a binary.
#[derive(Error, Debug)]
pub enum DeserializeError {
    /// An IO error
    #[error(transparent)]
    Io(#[from] io::Error),
    /// A generic deserialization error
    #[error("{0}")]
    Generic(String),
    /// Incompatible serialized binary
    #[error("incompatible binary: {0}")]
    Incompatible(String),
    /// The provided binary is corrupted
    #[error("corrupted binary: {0}")]
    CorruptedBinary(String),
    /// The binary was valid, but we got an error when
    /// trying to allocate the required resources.
    #[error(transparent)]
    Compiler(CompileError),
}

/// An ImportError.
///
/// Note: this error is not standard to WebAssembly, but it's
/// useful to determine the import issue on the API side.
#[derive(Error, Debug)]
pub enum ImportError {
    /// Incompatible Import Type.
    /// This error occurs when the import types mismatch.
    #[error("incompatible import type. Expected {0:?} but received {1:?}")]
    IncompatibleType(ExternType, ExternType),

    /// Unknown Import.
    /// This error occurs when an import was expected but not provided.
    #[error("unknown import. Expected {0:?}")]
    UnknownImport(ExternType),
}

/// An error while preinstantiating a module.
///
#[derive(Error, Debug)]
pub enum PreInstantiationError {
    /// The module was compiled with a CPU feature that is not available on
    /// the current host.
    #[error("module compiled with CPU feature that is missing from host")]
    CpuFeature(String),
}
