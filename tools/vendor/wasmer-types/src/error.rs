//! The WebAssembly possible errors
use crate::{ExternType, Pages};
use std::io;
use thiserror::Error;

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
    Compiler(#[from] CompileError),
    /// Input artifact bytes have an invalid length
    #[error("invalid input bytes: expected {expected} bytes, got {got}")]
    InvalidByteLength {
        /// How many bytes were expected
        expected: usize,
        /// How many bytes the artifact contained
        got: usize,
    },
}

/// Error type describing things that can go wrong when operating on Wasm Memories.
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MemoryError {
    /// Low level error with mmap.
    #[error("Error when allocating memory: {0}")]
    Region(String),
    /// The operation would cause the size of the memory to exceed the maximum or would cause
    /// an overflow leading to unindexable memory.
    #[error("The memory could not grow: current size {} pages, requested increase: {} pages", current.0, attempted_delta.0)]
    CouldNotGrow {
        /// The current size in pages.
        current: Pages,
        /// The attempted amount to grow by in pages.
        attempted_delta: Pages,
    },
    /// Invalid memory was provided.
    #[error("The memory is invalid because {}", reason)]
    InvalidMemory {
        /// The reason why the provided memory is invalid.
        reason: String,
    },
    /// Caller asked for more minimum memory than we can give them.
    #[error("The minimum requested ({} pages) memory is greater than the maximum allowed memory ({} pages)", min_requested.0, max_allowed.0)]
    MinimumMemoryTooLarge {
        /// The number of pages requested as the minimum amount of memory.
        min_requested: Pages,
        /// The maximum amount of memory we can allocate.
        max_allowed: Pages,
    },
    /// Caller asked for a maximum memory greater than we can give them.
    #[error("The maximum requested memory ({} pages) is greater than the maximum allowed memory ({} pages)", max_requested.0, max_allowed.0)]
    MaximumMemoryTooLarge {
        /// The number of pages requested as the maximum amount of memory.
        max_requested: Pages,
        /// The number of pages requested as the maximum amount of memory.
        max_allowed: Pages,
    },
    /// Returned when a shared memory is required, but the given memory is not shared.
    #[error("The memory is not shared")]
    MemoryNotShared,
    /// A user defined error value, used for error cases not listed above.
    #[error("A user-defined error occurred: {0}")]
    Generic(String),
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

    /// Memory Error
    #[error("memory error. {0}")]
    MemoryError(String),
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

use crate::lib::std::string::String;

// Compilation Errors
//
// If `std` feature is enable, we can't use `thiserror` until
// https://github.com/dtolnay/thiserror/pull/64 is merged.

/// The WebAssembly.CompileError object indicates an error during
/// WebAssembly decoding or validation.
///
/// This is based on the [Wasm Compile Error][compile-error] API.
///
/// [compiler-error]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/CompileError
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(Error))]
pub enum CompileError {
    /// A Wasm translation error occured.
    #[cfg_attr(feature = "std", error("WebAssembly translation error: {0}"))]
    Wasm(WasmError),

    /// A compilation error occured.
    #[cfg_attr(feature = "std", error("Compilation error: {0}"))]
    Codegen(String),

    /// The module did not pass validation.
    #[cfg_attr(feature = "std", error("Validation error: {0}"))]
    Validate(String),

    /// The compiler doesn't support a Wasm feature
    #[cfg_attr(feature = "std", error("Feature {0} is not yet supported"))]
    UnsupportedFeature(String),

    /// The compiler cannot compile for the given target.
    /// This can refer to the OS, the chipset or any other aspect of the target system.
    #[cfg_attr(
        feature = "std",
        error("The target {0} is not yet supported (see https://docs.wasmer.io/runtime/features)")
    )]
    UnsupportedTarget(String),

    /// Insufficient resources available for execution.
    #[cfg_attr(feature = "std", error("Insufficient resources: {0}"))]
    Resource(String),
}

impl From<WasmError> for CompileError {
    fn from(original: WasmError) -> Self {
        Self::Wasm(original)
    }
}

/// A error in the middleware.
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(Error))]
#[cfg_attr(feature = "std", error("Error in middleware {name}: {message}"))]
pub struct MiddlewareError {
    /// The name of the middleware where the error was created
    pub name: String,
    /// The error message
    pub message: String,
}

impl MiddlewareError {
    /// Create a new `MiddlewareError`
    pub fn new<A: Into<String>, B: Into<String>>(name: A, message: B) -> Self {
        Self {
            name: name.into(),
            message: message.into(),
        }
    }
}

/// A WebAssembly translation error.
///
/// When a WebAssembly function can't be translated, one of these error codes will be returned
/// to describe the failure.
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(Error))]
pub enum WasmError {
    /// The input WebAssembly code is invalid.
    ///
    /// This error code is used by a WebAssembly translator when it encounters invalid WebAssembly
    /// code. This should never happen for validated WebAssembly code.
    #[cfg_attr(
        feature = "std",
        error("Invalid input WebAssembly code at offset {offset}: {message}")
    )]
    InvalidWebAssembly {
        /// A string describing the validation error.
        message: String,
        /// The bytecode offset where the error occurred.
        offset: usize,
    },

    /// A feature used by the WebAssembly code is not supported by the embedding environment.
    ///
    /// Embedding environments may have their own limitations and feature restrictions.
    #[cfg_attr(feature = "std", error("Unsupported feature: {0}"))]
    Unsupported(String),

    /// An implementation limit was exceeded.
    #[cfg_attr(feature = "std", error("Implementation limit exceeded"))]
    ImplLimitExceeded,

    /// An error from the middleware error.
    #[cfg_attr(feature = "std", error("{0}"))]
    Middleware(MiddlewareError),

    /// A generic error.
    #[cfg_attr(feature = "std", error("{0}"))]
    Generic(String),
}

impl From<MiddlewareError> for WasmError {
    fn from(original: MiddlewareError) -> Self {
        Self::Middleware(original)
    }
}

/// The error that can happen while parsing a `str`
/// to retrieve a [`CpuFeature`](crate::CpuFeature).
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(Error))]
pub enum ParseCpuFeatureError {
    /// The provided string feature doesn't exist
    #[cfg_attr(feature = "std", error("CpuFeature {0} not recognized"))]
    Missing(String),
}

/// A convenient alias for a `Result` that uses `WasmError` as the error type.
pub type WasmResult<T> = Result<T, WasmError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn middleware_error_can_be_created() {
        let msg = String::from("Something went wrong");
        let error = MiddlewareError::new("manipulator3000", msg);
        assert_eq!(error.name, "manipulator3000");
        assert_eq!(error.message, "Something went wrong");
    }

    #[test]
    fn middleware_error_be_converted_to_wasm_error() {
        let error = WasmError::from(MiddlewareError::new("manipulator3000", "foo"));
        match error {
            WasmError::Middleware(MiddlewareError { name, message }) => {
                assert_eq!(name, "manipulator3000");
                assert_eq!(message, "foo");
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }
}
