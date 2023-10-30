#[cfg(not(target_arch = "wasm32"))]
use cosmwasm_crypto::CryptoError;
#[cfg(feature = "backtraces")]
use std::backtrace::Backtrace;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecoverPubkeyError {
    #[error("Invalid hash format")]
    InvalidHashFormat,
    #[error("Invalid signature format")]
    InvalidSignatureFormat,
    #[error("Invalid recovery parameter. Supported values: 0 and 1.")]
    InvalidRecoveryParam,
    #[error("Unknown error: {error_code}")]
    UnknownErr {
        error_code: u32,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
}

impl RecoverPubkeyError {
    pub fn unknown_err(error_code: u32) -> Self {
        RecoverPubkeyError::UnknownErr {
            error_code,
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }
}

impl PartialEq<RecoverPubkeyError> for RecoverPubkeyError {
    fn eq(&self, rhs: &RecoverPubkeyError) -> bool {
        match self {
            RecoverPubkeyError::InvalidHashFormat => {
                matches!(rhs, RecoverPubkeyError::InvalidHashFormat)
            }
            RecoverPubkeyError::InvalidSignatureFormat => {
                matches!(rhs, RecoverPubkeyError::InvalidSignatureFormat)
            }
            RecoverPubkeyError::InvalidRecoveryParam => {
                matches!(rhs, RecoverPubkeyError::InvalidRecoveryParam)
            }
            RecoverPubkeyError::UnknownErr { error_code, .. } => {
                if let RecoverPubkeyError::UnknownErr {
                    error_code: rhs_error_code,
                    ..
                } = rhs
                {
                    error_code == rhs_error_code
                } else {
                    false
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<CryptoError> for RecoverPubkeyError {
    fn from(original: CryptoError) -> Self {
        match original {
            CryptoError::InvalidHashFormat { .. } => RecoverPubkeyError::InvalidHashFormat,
            CryptoError::InvalidPubkeyFormat { .. } => panic!("Conversion not supported"),
            CryptoError::InvalidSignatureFormat { .. } => {
                RecoverPubkeyError::InvalidSignatureFormat
            }
            CryptoError::GenericErr { .. } => RecoverPubkeyError::unknown_err(original.code()),
            CryptoError::InvalidRecoveryParam { .. } => RecoverPubkeyError::InvalidRecoveryParam,
            CryptoError::BatchErr { .. } => panic!("Conversion not supported"),
        }
    }
}
