use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Binary;

/// SystemError is used for errors inside the VM and is API friendly (i.e. serializable).
///
/// This is used on return values for Querier as a nested result: Result<StdResult<T>, SystemError>
/// The first wrap (SystemError) will trigger if the contract address doesn't exist,
/// the QueryRequest is malformated, etc. The second wrap will be an error message from
/// the contract itself.
///
/// Such errors are only created by the VM. The error type is defined in the standard library, to ensure
/// the contract understands the error format without creating a dependency on cosmwasm-vm.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SystemError {
    InvalidRequest {
        error: String,
        request: Binary,
    },
    InvalidResponse {
        error: String,
        response: Binary,
    },
    NoSuchContract {
        /// The address that was attempted to query
        addr: String,
    },
    /// A Wasm code was not found.
    NoSuchCode {
        /// The code ID that is missing
        code_id: u64,
    },
    Unknown {},
    UnsupportedRequest {
        kind: String,
    },
}

impl std::error::Error for SystemError {}

impl std::fmt::Display for SystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemError::InvalidRequest { error, request } => write!(
                f,
                "Cannot parse request: {} in: {}",
                error,
                String::from_utf8_lossy(request)
            ),
            SystemError::InvalidResponse { error, response } => write!(
                f,
                "Cannot parse response: {} in: {}",
                error,
                String::from_utf8_lossy(response)
            ),
            SystemError::NoSuchContract { addr } => write!(f, "No such contract: {}", addr),
            SystemError::NoSuchCode { code_id } => write!(f, "No such code: {}", code_id),
            SystemError::Unknown {} => write!(f, "Unknown system error"),
            SystemError::UnsupportedRequest { kind } => {
                write!(f, "Unsupported query type: {}", kind)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_slice, to_vec};

    #[test]
    fn system_error_no_such_contract_serialization() {
        let err = SystemError::NoSuchContract {
            addr: "gibtsnicht".to_string(),
        };

        // ser
        let json = to_vec(&err).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&json),
            r#"{"no_such_contract":{"addr":"gibtsnicht"}}"#,
        );

        // de
        let err: SystemError = from_slice(br#"{"no_such_contract":{"addr":"nada"}}"#).unwrap();
        assert_eq!(
            err,
            SystemError::NoSuchContract {
                addr: "nada".to_string()
            }
        );
    }

    #[test]
    fn system_error_no_such_code_serialization() {
        let err = SystemError::NoSuchCode { code_id: 13 };

        // ser
        let json = to_vec(&err).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&json),
            r#"{"no_such_code":{"code_id":13}}"#,
        );

        // de
        let err: SystemError = from_slice(br#"{"no_such_code":{"code_id":987}}"#).unwrap();
        assert_eq!(err, SystemError::NoSuchCode { code_id: 987 },);
    }
}
