//! This file simply re-exports some methods from serde_json
//! The reason is two fold:
//! 1. To easily ensure that all calling libraries use the same version (minimize code size)
//! 2. To allow us to switch out to eg. serde-json-core more easily
use serde::{Deserialize, Serialize};
use std::any::type_name;

use crate::errors::{VmError, VmResult};

/// Deserializes JSON data into a document of type `T`.
///
/// The deserialization limit ensure it is not possible to slow down the execution by
/// providing overly large JSON documents.
pub fn from_slice<'a, T>(value: &'a [u8], deserialization_limit: usize) -> VmResult<T>
where
    T: Deserialize<'a>,
{
    if value.len() > deserialization_limit {
        return Err(VmError::deserialization_limit_exceeded(
            value.len(),
            deserialization_limit,
        ));
    }
    serde_json::from_slice(value).map_err(|e| VmError::parse_err(type_name::<T>(), e))
}

pub fn to_vec<T>(data: &T) -> VmResult<Vec<u8>>
where
    T: Serialize + ?Sized,
{
    serde_json::to_vec(data).map_err(|e| VmError::serialize_err(type_name::<T>(), e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    const LIMIT: usize = 20_000;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    #[serde(rename_all = "snake_case")]
    enum SomeMsg {
        Refund {},
        ReleaseAll {
            image: String,
            amount: u32,
            time: u64,
            karma: i32,
        },
        Cowsay {
            text: String,
        },
    }

    #[test]
    fn from_slice_works() {
        let deserialized: SomeMsg = from_slice(br#"{"refund":{}}"#, LIMIT).unwrap();
        assert_eq!(deserialized, SomeMsg::Refund {});

        let deserialized: SomeMsg = from_slice(
            br#"{"release_all":{"image":"foo","amount":42,"time":18446744073709551615,"karma":-17}}"#, LIMIT
        )
        .unwrap();
        assert_eq!(
            deserialized,
            SomeMsg::ReleaseAll {
                image: "foo".to_string(),
                amount: 42,
                time: 18446744073709551615,
                karma: -17
            }
        );
    }

    #[test]
    fn from_slice_works_for_special_chars() {
        let deserialized: SomeMsg =
            from_slice(br#"{"cowsay":{"text":"foo\"bar\\\"bla"}}"#, LIMIT).unwrap();
        assert_eq!(
            deserialized,
            SomeMsg::Cowsay {
                text: "foo\"bar\\\"bla".to_string(),
            }
        );
    }

    #[test]
    fn from_slice_errors_when_exceeding_deserialization_limit() {
        let result = from_slice::<SomeMsg>(br#"{"refund":{}}"#, 5);
        match result.unwrap_err() {
            VmError::DeserializationLimitExceeded {
                length, max_length, ..
            } => {
                assert_eq!(length, 13);
                assert_eq!(max_length, 5);
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn to_vec_works() {
        let msg = SomeMsg::Refund {};
        let serialized = to_vec(&msg).unwrap();
        assert_eq!(serialized, br#"{"refund":{}}"#);

        let msg = SomeMsg::ReleaseAll {
            image: "foo".to_string(),
            amount: 42,
            time: 9007199254740999, // Number.MAX_SAFE_INTEGER + 7
            karma: -17,
        };
        let serialized = String::from_utf8(to_vec(&msg).unwrap()).unwrap();
        assert_eq!(
            serialized,
            r#"{"release_all":{"image":"foo","amount":42,"time":9007199254740999,"karma":-17}}"#
        );
    }

    #[test]
    fn to_vec_works_for_special_chars() {
        let msg = SomeMsg::Cowsay {
            text: "foo\"bar\\\"bla".to_string(),
        };
        let serialized = String::from_utf8(to_vec(&msg).unwrap()).unwrap();
        assert_eq!(serialized, r#"{"cowsay":{"text":"foo\"bar\\\"bla"}}"#);
    }
}
