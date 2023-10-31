use crate::lib::std::string::String;
#[cfg(feature = "std")]
use thiserror::Error;

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
    #[cfg_attr(feature = "std", error("The target {0} is not yet supported (see https://docs.wasmer.io/ecosystem/wasmer/wasmer-features)"))]
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
/// to retrieve a [`CpuFeature`](crate::target::CpuFeature).
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
