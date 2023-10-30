//! Result and error types representing the outcome of compiling a function.

use crate::verifier::VerifierErrors;
use std::string::String;

/// A compilation error.
///
/// When Cranelift fails to compile a function, it will return one of these error codes.
#[derive(Debug, PartialEq, Eq)]
pub enum CodegenError {
    /// A list of IR verifier errors.
    ///
    /// This always represents a bug, either in the code that generated IR for Cranelift, or a bug
    /// in Cranelift itself.
    Verifier(VerifierErrors),

    /// An implementation limit was exceeded.
    ///
    /// Cranelift can compile very large and complicated functions, but the [implementation has
    /// limits][limits] that cause compilation to fail when they are exceeded.
    ///
    /// [limits]: https://github.com/bytecodealliance/wasmtime/blob/main/cranelift/docs/ir.md#implementation-limits
    ImplLimitExceeded,

    /// The code size for the function is too large.
    ///
    /// Different target ISAs may impose a limit on the size of a compiled function. If that limit
    /// is exceeded, compilation fails.
    CodeTooLarge,

    /// Something is not supported by the code generator. This might be an indication that a
    /// feature is used without explicitly enabling it, or that something is temporarily
    /// unsupported by a given target backend.
    Unsupported(String),

    /// A failure to map Cranelift register representation to a DWARF register representation.
    #[cfg(feature = "unwind")]
    RegisterMappingError(crate::isa::unwind::systemv::RegisterMappingError),
}

/// A convenient alias for a `Result` that uses `CodegenError` as the error type.
pub type CodegenResult<T> = Result<T, CodegenError>;

// This is manually implementing Error and Display instead of using thiserror to reduce the amount
// of dependencies used by Cranelift.
impl std::error::Error for CodegenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CodegenError::Verifier(source) => Some(source),
            CodegenError::ImplLimitExceeded { .. }
            | CodegenError::CodeTooLarge { .. }
            | CodegenError::Unsupported { .. } => None,
            #[cfg(feature = "unwind")]
            CodegenError::RegisterMappingError { .. } => None,
        }
    }
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CodegenError::Verifier(_) => write!(f, "Verifier errors"),
            CodegenError::ImplLimitExceeded => write!(f, "Implementation limit exceeded"),
            CodegenError::CodeTooLarge => write!(f, "Code for function is too large"),
            CodegenError::Unsupported(feature) => write!(f, "Unsupported feature: {}", feature),
            #[cfg(feature = "unwind")]
            CodegenError::RegisterMappingError(_0) => write!(f, "Register mapping error"),
        }
    }
}

impl From<VerifierErrors> for CodegenError {
    fn from(source: VerifierErrors) -> Self {
        CodegenError::Verifier { 0: source }
    }
}
