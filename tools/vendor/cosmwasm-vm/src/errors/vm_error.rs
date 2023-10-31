#[cfg(feature = "backtraces")]
use std::backtrace::Backtrace;
use std::fmt::{Debug, Display};
use thiserror::Error;

use cosmwasm_crypto::CryptoError;

use super::communication_error::CommunicationError;
use crate::backend::BackendError;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum VmError {
    #[error("Aborted: {}", msg)]
    Aborted {
        msg: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Error calling into the VM's backend: {}", source)]
    BackendErr {
        source: BackendError,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Cache error: {msg}")]
    CacheErr {
        msg: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Error in guest/host communication: {source}")]
    CommunicationErr {
        #[from]
        source: CommunicationError,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Error compiling Wasm: {msg}")]
    CompileErr {
        msg: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Couldn't convert from {} to {}. Input: {}", from_type, to_type, input)]
    ConversionErr {
        from_type: String,
        to_type: String,
        input: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Crypto error: {}", source)]
    CryptoErr {
        source: CryptoError,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Ran out of gas during contract execution")]
    GasDepletion {
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    /// Whenever there is no specific error type available
    #[error("Generic error: {msg}")]
    GenericErr {
        msg: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Error instantiating a Wasm module: {msg}")]
    InstantiationErr {
        msg: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Hash doesn't match stored data")]
    IntegrityErr {
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Error parsing into type {target_type}: {msg}")]
    ParseErr {
        /// the target type that was attempted
        target_type: String,
        msg: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Data too long for deserialization. Got: {length} bytes; limit: {max_length} bytes")]
    DeserializationLimitExceeded {
        /// the target type that was attempted
        length: usize,
        max_length: usize,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Error serializing type {source_type}: {msg}")]
    SerializeErr {
        /// the source type that was attempted
        source_type: String,
        msg: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Error resolving Wasm function: {}", msg)]
    ResolveErr {
        msg: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error(
        "Unexpected number of result values when calling '{}'. Expected: {}, actual: {}.",
        function_name,
        expected,
        actual
    )]
    ResultMismatch {
        function_name: String,
        expected: usize,
        actual: usize,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Error executing Wasm: {}", msg)]
    RuntimeErr {
        msg: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Error during static Wasm validation: {}", msg)]
    StaticValidationErr {
        msg: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Uninitialized Context Data: {}", kind)]
    UninitializedContextData {
        kind: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Must not call a writing storage function in this context.")]
    WriteAccessDenied {
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Maximum call depth exceeded.")]
    MaxCallDepthExceeded {
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
}

impl VmError {
    pub(crate) fn aborted(msg: impl Into<String>) -> Self {
        VmError::Aborted {
            msg: msg.into(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub(crate) fn backend_err(original: BackendError) -> Self {
        VmError::BackendErr {
            source: original,
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub(crate) fn cache_err(msg: impl Into<String>) -> Self {
        VmError::CacheErr {
            msg: msg.into(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub(crate) fn compile_err(msg: impl Into<String>) -> Self {
        VmError::CompileErr {
            msg: msg.into(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub(crate) fn conversion_err(
        from_type: impl Into<String>,
        to_type: impl Into<String>,
        input: impl Into<String>,
    ) -> Self {
        VmError::ConversionErr {
            from_type: from_type.into(),
            to_type: to_type.into(),
            input: input.into(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub(crate) fn crypto_err(original: CryptoError) -> Self {
        VmError::CryptoErr {
            source: original,
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub(crate) fn gas_depletion() -> Self {
        VmError::GasDepletion {
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub(crate) fn generic_err(msg: impl Into<String>) -> Self {
        VmError::GenericErr {
            msg: msg.into(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub(crate) fn instantiation_err(msg: impl Into<String>) -> Self {
        VmError::InstantiationErr {
            msg: msg.into(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub(crate) fn integrity_err() -> Self {
        VmError::IntegrityErr {
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub(crate) fn parse_err(target: impl Into<String>, msg: impl Display) -> Self {
        VmError::ParseErr {
            target_type: target.into(),
            msg: msg.to_string(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub(crate) fn deserialization_limit_exceeded(length: usize, max_length: usize) -> Self {
        VmError::DeserializationLimitExceeded {
            length,
            max_length,
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub(crate) fn serialize_err(source: impl Into<String>, msg: impl Display) -> Self {
        VmError::SerializeErr {
            source_type: source.into(),
            msg: msg.to_string(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub(crate) fn resolve_err(msg: impl Into<String>) -> Self {
        VmError::ResolveErr {
            msg: msg.into(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub(crate) fn result_mismatch(
        function_name: impl Into<String>,
        expected: usize,
        actual: usize,
    ) -> Self {
        VmError::ResultMismatch {
            function_name: function_name.into(),
            expected,
            actual,
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    // Creates a runtime error with the given message.
    // This is private since it is only needed when converting wasmer::RuntimeError
    // to VmError.
    fn runtime_err(msg: impl Into<String>) -> Self {
        VmError::RuntimeErr {
            msg: msg.into(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub(crate) fn static_validation_err(msg: impl Into<String>) -> Self {
        VmError::StaticValidationErr {
            msg: msg.into(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub(crate) fn uninitialized_context_data(kind: impl Into<String>) -> Self {
        VmError::UninitializedContextData {
            kind: kind.into(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub(crate) fn write_access_denied() -> Self {
        VmError::WriteAccessDenied {
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub(crate) fn max_call_depth_exceeded() -> Self {
        VmError::MaxCallDepthExceeded {
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }
}

impl From<BackendError> for VmError {
    fn from(original: BackendError) -> Self {
        match original {
            BackendError::OutOfGas {} => VmError::gas_depletion(),
            _ => VmError::backend_err(original),
        }
    }
}

impl From<CryptoError> for VmError {
    fn from(original: CryptoError) -> Self {
        VmError::crypto_err(original)
    }
}

impl From<wasmer::ExportError> for VmError {
    fn from(original: wasmer::ExportError) -> Self {
        VmError::resolve_err(format!("Could not get export: {}", original))
    }
}

impl From<wasmer::SerializeError> for VmError {
    fn from(original: wasmer::SerializeError) -> Self {
        VmError::cache_err(format!("Could not serialize module: {}", original))
    }
}

impl From<wasmer::DeserializeError> for VmError {
    fn from(original: wasmer::DeserializeError) -> Self {
        VmError::cache_err(format!("Could not deserialize module: {}", original))
    }
}

impl From<wasmer::RuntimeError> for VmError {
    fn from(original: wasmer::RuntimeError) -> Self {
        // Do not use the Display implementation or to_string() of `RuntimeError`
        // because it can contain a system specific stack trace, which can
        // lead to non-deterministic execution.
        //
        // Implementation follows https://github.com/wasmerio/wasmer/blob/2.0.0/lib/engine/src/trap/error.rs#L215
        let message = format!("RuntimeError: {}", original.message());
        debug_assert!(
            original.to_string().starts_with(&message),
            "The error message we created is not a prefix of the error message from Wasmer. Our message: '{}'. Wasmer messsage: '{}'",
            &message,
            original
        );
        VmError::runtime_err(format!("Wasmer runtime error: {}", &message))
    }
}

impl From<wasmer::CompileError> for VmError {
    fn from(original: wasmer::CompileError) -> Self {
        VmError::compile_err(format!("Could not compile: {}", original))
    }
}

impl From<std::convert::Infallible> for VmError {
    fn from(_original: std::convert::Infallible) -> Self {
        unreachable!();
    }
}

impl From<VmError> for wasmer::RuntimeError {
    fn from(original: VmError) -> wasmer::RuntimeError {
        let msg: String = original.to_string();
        wasmer::RuntimeError::new(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // constructors

    #[test]
    fn backend_err_works() {
        let error = VmError::backend_err(BackendError::unknown("something went wrong"));
        match error {
            VmError::BackendErr {
                source: BackendError::Unknown { msg },
                ..
            } => assert_eq!(msg, "something went wrong"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn cache_err_works() {
        let error = VmError::cache_err("something went wrong");
        match error {
            VmError::CacheErr { msg, .. } => assert_eq!(msg, "something went wrong"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn compile_err_works() {
        let error = VmError::compile_err("something went wrong");
        match error {
            VmError::CompileErr { msg, .. } => assert_eq!(msg, "something went wrong"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn conversion_err_works() {
        let error = VmError::conversion_err("i32", "u32", "-9");
        match error {
            VmError::ConversionErr {
                from_type,
                to_type,
                input,
                ..
            } => {
                assert_eq!(from_type, "i32");
                assert_eq!(to_type, "u32");
                assert_eq!(input, "-9");
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn cyrpto_err_works() {
        let error = VmError::crypto_err(CryptoError::generic_err("something went wrong"));
        match error {
            VmError::CryptoErr {
                source: CryptoError::GenericErr { msg, .. },
                ..
            } => assert_eq!(msg, "something went wrong"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn gas_depletion_works() {
        let error = VmError::gas_depletion();
        match error {
            VmError::GasDepletion { .. } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn generic_err_works() {
        let guess = 7;
        let error = VmError::generic_err(format!("{} is too low", guess));
        match error {
            VmError::GenericErr { msg, .. } => {
                assert_eq!(msg, String::from("7 is too low"));
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn instantiation_err_works() {
        let error = VmError::instantiation_err("something went wrong");
        match error {
            VmError::InstantiationErr { msg, .. } => assert_eq!(msg, "something went wrong"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn integrity_err_works() {
        let error = VmError::integrity_err();
        match error {
            VmError::IntegrityErr { .. } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn parse_err_works() {
        let error = VmError::parse_err("Book", "Missing field: title");
        match error {
            VmError::ParseErr {
                target_type, msg, ..
            } => {
                assert_eq!(target_type, "Book");
                assert_eq!(msg, "Missing field: title");
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn serialize_err_works() {
        let error = VmError::serialize_err("Book", "Content too long");
        match error {
            VmError::SerializeErr {
                source_type, msg, ..
            } => {
                assert_eq!(source_type, "Book");
                assert_eq!(msg, "Content too long");
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn resolve_err_works() {
        let error = VmError::resolve_err("function has different signature");
        match error {
            VmError::ResolveErr { msg, .. } => assert_eq!(msg, "function has different signature"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn result_mismatch_works() {
        let error = VmError::result_mismatch("action", 0, 1);
        match error {
            VmError::ResultMismatch {
                function_name,
                expected,
                actual,
                ..
            } => {
                assert_eq!(function_name, "action");
                assert_eq!(expected, 0);
                assert_eq!(actual, 1);
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn runtime_err_works() {
        let error = VmError::runtime_err("something went wrong");
        match error {
            VmError::RuntimeErr { msg, .. } => assert_eq!(msg, "something went wrong"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn static_validation_err_works() {
        let error = VmError::static_validation_err("export xy missing");
        match error {
            VmError::StaticValidationErr { msg, .. } => assert_eq!(msg, "export xy missing"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn uninitialized_context_data_works() {
        let error = VmError::uninitialized_context_data("foo");
        match error {
            VmError::UninitializedContextData { kind, .. } => assert_eq!(kind, "foo"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn write_access_denied() {
        let error = VmError::write_access_denied();
        match error {
            VmError::WriteAccessDenied { .. } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }
}
