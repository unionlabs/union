use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::super::errors::SystemError;

/// This is the outer result type returned by a querier to the contract.
///
/// We use a custom type here instead of Rust's Result because we want to be able to
/// define the serialization, which is a public interface. Every language that compiles
/// to Wasm and runs in the ComsWasm VM needs to create the same JSON representation.
///
/// # Examples
///
/// Success:
///
/// ```
/// # use cosmwasm_std::{to_vec, Binary, ContractResult, SystemResult};
/// let data = Binary::from(b"hello, world");
/// let result = SystemResult::Ok(ContractResult::Ok(data));
/// assert_eq!(to_vec(&result).unwrap(), br#"{"ok":{"ok":"aGVsbG8sIHdvcmxk"}}"#);
/// ```
///
/// Failure:
///
/// ```
/// # use cosmwasm_std::{to_vec, Binary, ContractResult, SystemResult, SystemError};
/// let error = SystemError::Unknown {};
/// let result: SystemResult<Binary> = SystemResult::Err(error);
/// assert_eq!(to_vec(&result).unwrap(), br#"{"error":{"unknown":{}}}"#);
/// ```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SystemResult<S> {
    Ok(S),
    #[serde(rename = "error")]
    Err(SystemError),
}

// Implementations here mimic the Result API and should be implemented via a conversion to Result
// to ensure API consistency
impl<S> SystemResult<S> {
    /// Converts a `ContractResult<S>` to a `Result<S, SystemError>` as a convenient way
    /// to access the full Result API.
    pub fn into_result(self) -> Result<S, SystemError> {
        Result::<S, SystemError>::from(self)
    }

    pub fn unwrap(self) -> S {
        self.into_result().unwrap()
    }
}

impl<S: fmt::Debug> SystemResult<S> {
    pub fn unwrap_err(self) -> SystemError {
        self.into_result().unwrap_err()
    }
}

impl<S> From<Result<S, SystemError>> for SystemResult<S> {
    fn from(original: Result<S, SystemError>) -> SystemResult<S> {
        match original {
            Ok(value) => SystemResult::Ok(value),
            Err(err) => SystemResult::Err(err),
        }
    }
}

impl<S> From<SystemResult<S>> for Result<S, SystemError> {
    fn from(original: SystemResult<S>) -> Result<S, SystemError> {
        match original {
            SystemResult::Ok(value) => Ok(value),
            SystemResult::Err(err) => Err(err),
        }
    }
}
