#[cfg(feature = "js")]
use crate::js::trap::Trap;
#[cfg(feature = "jsc")]
use crate::jsc::trap::Trap;
use std::fmt;
use std::sync::Arc;
use thiserror::Error;
use wasmer_types::{FrameInfo, TrapCode};
#[cfg(feature = "sys")]
use wasmer_vm::Trap;

use wasmer_types::ImportError;

/// The WebAssembly.LinkError object indicates an error during
/// module instantiation (besides traps from the start function).
///
/// This is based on the [link error][link-error] API.
///
/// [link-error]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/LinkError
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(Error))]
#[cfg_attr(feature = "std", error("Link error: {0}"))]
pub enum LinkError {
    /// An error occurred when checking the import types.
    #[cfg_attr(feature = "std", error("Error while importing {0:?}.{1:?}: {2}"))]
    Import(String, String, ImportError),

    /// A trap ocurred during linking.
    #[cfg_attr(feature = "std", error("RuntimeError occurred during linking: {0}"))]
    Trap(#[source] RuntimeError),
    /// Insufficient resources available for linking.
    #[cfg_attr(feature = "std", error("Insufficient resources: {0}"))]
    Resource(String),
}

/// An error while instantiating a module.
///
/// This is not a common WebAssembly error, however
/// we need to differentiate from a `LinkError` (an error
/// that happens while linking, on instantiation), a
/// Trap that occurs when calling the WebAssembly module
/// start function, and an error when initializing the user's
/// host environments.
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(Error))]
pub enum InstantiationError {
    /// A linking ocurred during instantiation.
    #[cfg_attr(feature = "std", error(transparent))]
    Link(LinkError),

    /// A runtime error occured while invoking the start function
    #[cfg_attr(feature = "std", error(transparent))]
    Start(RuntimeError),

    /// The module was compiled with a CPU feature that is not available on
    /// the current host.
    #[cfg_attr(feature = "std", error("missing required CPU features: {0:?}"))]
    CpuFeature(String),

    /// Import from a different [`Store`][super::Store].
    /// This error occurs when an import from a different store is used.
    #[cfg_attr(feature = "std", error("cannot mix imports from different stores"))]
    DifferentStores,

    /// Import from a different Store.
    /// This error occurs when an import from a different store is used.
    #[cfg_attr(feature = "std", error("incorrect OS or architecture"))]
    DifferentArchOS,
}

/// A struct representing an aborted instruction execution, with a message
/// indicating the cause.
#[derive(Clone)]
pub struct RuntimeError {
    pub(crate) inner: Arc<RuntimeErrorInner>,
}

#[derive(Debug)]
struct RuntimeStringError {
    details: String,
}

impl RuntimeStringError {
    fn new(msg: String) -> Self {
        Self { details: msg }
    }
}

impl fmt::Display for RuntimeStringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for RuntimeStringError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub(crate) struct RuntimeErrorInner {
    /// The source error
    pub(crate) source: Trap,
    /// The trap code (if any)
    trap_code: Option<TrapCode>,
    /// The reconstructed Wasm trace (from the native trace and the `GlobalFrameInfo`).
    wasm_trace: Vec<FrameInfo>,
}

impl RuntimeError {
    /// Creates a new generic `RuntimeError` with the given `message`.
    ///
    /// # Example
    /// ```
    /// let trap = wasmer::RuntimeError::new("unexpected error");
    /// assert_eq!("unexpected error", trap.message());
    /// ```
    pub fn new<I: Into<String>>(message: I) -> Self {
        let msg = message.into();
        let source = RuntimeStringError::new(msg);
        Self::user(Box::new(source))
    }

    /// Creates `RuntimeError` from an error and a WasmTrace
    ///
    /// # Example
    /// ```ignore
    /// let wasm_trace = vec![wasmer_types::FrameInfo::new(
    ///   "my_module".to_string(),
    ///   0,
    ///   Some("my_function".to_string()),
    ///   0.into(),
    ///   2.into()
    /// )];
    /// let trap = wasmer::RuntimeError::new_from_source(my_error, wasm_trace, None);
    /// assert_eq!("unexpected error", trap.message());
    /// ```
    pub fn new_from_source(
        source: Trap,
        wasm_trace: Vec<FrameInfo>,
        trap_code: Option<TrapCode>,
    ) -> Self {
        Self {
            inner: Arc::new(RuntimeErrorInner {
                source,
                wasm_trace,
                trap_code,
            }),
        }
    }

    /// Creates a custom user Error.
    ///
    /// This error object can be passed through Wasm frames and later retrieved
    /// using the `downcast` method.
    pub fn user(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        match error.downcast::<Self>() {
            Ok(err) => *err,
            Err(error) => error.into(),
        }
    }

    /// Returns a reference the `message` stored in `Trap`.
    pub fn message(&self) -> String {
        if let Some(trap_code) = self.inner.trap_code {
            trap_code.message().to_string()
        } else {
            self.inner.source.to_string()
        }
    }

    /// Returns a list of function frames in WebAssembly code that led to this
    /// trap happening.
    pub fn trace(&self) -> &[FrameInfo] {
        &self.inner.wasm_trace
    }

    /// Returns trap code, if it's a Trap
    pub fn to_trap(self) -> Option<TrapCode> {
        self.inner.trap_code
    }

    // /// Returns trap code, if it's a Trap
    // pub fn to_source(self) -> &'static Trap {
    //     &self.inner.as_ref().source
    // }

    /// Attempts to downcast the `RuntimeError` to a concrete type.
    pub fn downcast<T: std::error::Error + 'static>(self) -> Result<T, Self> {
        match Arc::try_unwrap(self.inner) {
            Ok(inner) if inner.source.is::<T>() => Ok(inner.source.downcast::<T>().unwrap()),
            Ok(inner) => Err(Self {
                inner: Arc::new(inner),
            }),
            Err(inner) => Err(Self { inner }),
        }
    }

    /// Attempts to downcast the `RuntimeError` to a concrete type.
    pub fn downcast_ref<T: std::error::Error + 'static>(&self) -> Option<&T> {
        self.inner.as_ref().source.downcast_ref::<T>()
    }

    /// Returns true if the `RuntimeError` is the same as T
    pub fn is<T: std::error::Error + 'static>(&self) -> bool {
        self.inner.source.is::<T>()
    }
}

impl fmt::Debug for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RuntimeError")
            .field("source", &self.inner.source)
            .field("wasm_trace", &self.inner.wasm_trace)
            .finish()
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RuntimeError: {}", self.message())?;
        let trace = self.trace();
        if trace.is_empty() {
            return Ok(());
        }
        for frame in self.trace().iter() {
            let name = frame.module_name();
            let func_index = frame.func_index();
            writeln!(f)?;
            write!(f, "    at ")?;
            match frame.function_name() {
                Some(name) => match rustc_demangle::try_demangle(name) {
                    Ok(name) => write!(f, "{}", name)?,
                    Err(_) => write!(f, "{}", name)?,
                },
                None => write!(f, "<unnamed>")?,
            }
            write!(
                f,
                " ({}[{}]:0x{:x})",
                name,
                func_index,
                frame.module_offset()
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for RuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source.source()
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for RuntimeError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        match error.downcast::<Self>() {
            // The error is already a RuntimeError, we return it directly
            Ok(runtime_error) => *runtime_error,
            Err(error) => Trap::user(error).into(),
        }
    }
}
