use serde::{Deserializer, Serializer};
use serde_utils::hex::{self, PrefixedHexVisitor};
use typenum::Unsigned;

use crate::FixedVector;

pub fn serialize<S, U>(bytes: &FixedVector<u8, U>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    U: Unsigned,
{
    serializer.serialize_str(&hex::encode(&bytes[..]))
}

pub fn deserialize<'de, D, U>(deserializer: D) -> Result<FixedVector<u8, U>, D::Error>
where
    D: Deserializer<'de>,
    U: Unsigned,
{
    let vec = deserializer.deserialize_string(PrefixedHexVisitor)?;
    FixedVector::new(vec)
        .map_err(|e| serde::de::Error::custom(format!("invalid fixed vector: {:?}", e)))
}
