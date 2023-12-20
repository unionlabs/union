//! The WebAssembly possible errors
use thiserror::Error;
pub use wasmer_types::{DeserializeError, ImportError, SerializeError};
#[cfg(not(target_arch = "wasm32"))]
use wasmer_vm::Trap;

/// The WebAssembly.LinkError object indicates an error during
/// module instantiation (besides traps from the start function).
///
/// This is based on the [link error][link-error] API.
///
/// [link-error]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/LinkError
#[derive(Error, Debug)]
#[error("Link error: {0}")]
pub enum LinkError {
    /// An error occurred when checking the import types.
    #[error("Error while importing {0:?}.{1:?}: {2}")]
    Import(String, String, ImportError),

    #[cfg(not(target_arch = "wasm32"))]
    /// A trap ocurred during linking.
    #[error("Trap occurred during linking: {0}")]
    Trap(#[source] Trap),

    /// Insufficient resources available for linking.
    #[error("Insufficient resources: {0}")]
    Resource(String),
}

/// An error while instantiating a module.
///
/// This is not a common WebAssembly error, however
/// we need to differentiate from a `LinkError` (an error
/// that happens while linking, on instantiation) and a
/// Trap that occurs when calling the WebAssembly module
/// start function.
#[derive(Error, Debug)]
pub enum InstantiationError {
    /// A linking ocurred during instantiation.
    #[error(transparent)]
    Link(LinkError),

    /// The module was compiled with a CPU feature that is not available on
    /// the current host.
    #[error("module compiled with CPU feature that is missing from host")]
    CpuFeature(String),

    /// A runtime error occured while invoking the start function
    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    Start(Trap),
}
