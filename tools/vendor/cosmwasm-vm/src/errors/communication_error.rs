use std::fmt::Debug;
use thiserror::Error;

use super::region_validation_error::RegionValidationError;

/// An error in the communcation between contract and host. Those happen around imports and exports.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum CommunicationError {
    #[error(
        "The Wasm memory address {} provided by the contract could not be dereferenced: {}",
        offset,
        msg
    )]
    DerefErr {
        /// the position in a Wasm linear memory
        offset: u32,
        msg: String,
    },
    #[error("Got an invalid value for iteration order: {}", value)]
    InvalidOrder { value: i32 },
    #[error("Got an invalid region: {}", source)]
    InvalidRegion {
        #[from]
        source: RegionValidationError,
    },
    /// Whenever UTF-8 bytes cannot be decoded into a unicode string, e.g. in String::from_utf8 or str::from_utf8.
    #[error("Cannot decode UTF8 bytes into string: {}", msg)]
    InvalidUtf8 { msg: String },
    #[error("Region length too big. Got {}, limit {}", length, max_length)]
    // Note: this only checks length, not capacity
    RegionLengthTooBig { length: usize, max_length: usize },
    #[error("Region too small. Got {}, required {}", size, required)]
    RegionTooSmall { size: usize, required: usize },
    #[error("Got a zero Wasm address")]
    ZeroAddress {},
}

impl CommunicationError {
    pub(crate) fn deref_err(offset: u32, msg: impl Into<String>) -> Self {
        CommunicationError::DerefErr {
            offset,
            msg: msg.into(),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn invalid_order(value: i32) -> Self {
        CommunicationError::InvalidOrder { value }
    }

    #[allow(dead_code)]
    pub(crate) fn invalid_utf8(msg: impl ToString) -> Self {
        CommunicationError::InvalidUtf8 {
            msg: msg.to_string(),
        }
    }

    pub(crate) fn region_length_too_big(length: usize, max_length: usize) -> Self {
        CommunicationError::RegionLengthTooBig { length, max_length }
    }

    pub(crate) fn region_too_small(size: usize, required: usize) -> Self {
        CommunicationError::RegionTooSmall { size, required }
    }

    pub(crate) fn zero_address() -> Self {
        CommunicationError::ZeroAddress {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // constructors

    #[test]
    fn deref_err() {
        let error = CommunicationError::deref_err(345, "broken stuff");
        match error {
            CommunicationError::DerefErr { offset, msg, .. } => {
                assert_eq!(offset, 345);
                assert_eq!(msg, "broken stuff");
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn invalid_order() {
        let error = CommunicationError::invalid_order(-745);
        match error {
            CommunicationError::InvalidOrder { value, .. } => assert_eq!(value, -745),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn invalid_utf8() {
        let error = CommunicationError::invalid_utf8("broken");
        match error {
            CommunicationError::InvalidUtf8 { msg, .. } => assert_eq!(msg, "broken"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn region_length_too_big_works() {
        let error = CommunicationError::region_length_too_big(50, 20);
        match error {
            CommunicationError::RegionLengthTooBig {
                length, max_length, ..
            } => {
                assert_eq!(length, 50);
                assert_eq!(max_length, 20);
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn region_too_small_works() {
        let error = CommunicationError::region_too_small(12, 33);
        match error {
            CommunicationError::RegionTooSmall { size, required, .. } => {
                assert_eq!(size, 12);
                assert_eq!(required, 33);
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn zero_address() {
        let error = CommunicationError::zero_address();
        match error {
            CommunicationError::ZeroAddress { .. } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }
}
