//! Serialize `VariableList<u8, N>` as 0x-prefixed hex string.
use serde::{Deserializer, Serializer};
use typenum::Unsigned;

use crate::VariableList;

pub fn serialize<S, N>(bytes: &VariableList<u8, N>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    N: Unsigned,
{
    serde_utils::hex_string::serialize(bytes, serializer)
}

pub fn deserialize<'de, D, N>(deserializer: D) -> Result<VariableList<u8, N>, D::Error>
where
    D: Deserializer<'de>,
    N: Unsigned,
{
    let bytes = serde_utils::hex_string::deserialize::<_, Vec<u8>>(deserializer)?;
    VariableList::new(bytes)
        .map_err(|e| serde::de::Error::custom(format!("invalid variable list: {:?}", e)))
}
