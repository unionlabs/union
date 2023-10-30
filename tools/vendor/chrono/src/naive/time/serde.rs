#![cfg_attr(docsrs, doc(cfg(feature = "serde")))]

use super::NaiveTime;
use core::fmt;
use serde::{de, ser};

// TODO not very optimized for space (binary formats would want something better)
// TODO round-trip for general leap seconds (not just those with second = 60)

impl ser::Serialize for NaiveTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.collect_str(&self)
    }
}

struct NaiveTimeVisitor;

impl<'de> de::Visitor<'de> for NaiveTimeVisitor {
    type Value = NaiveTime;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a formatted time string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        value.parse().map_err(E::custom)
    }
}

impl<'de> de::Deserialize<'de> for NaiveTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(NaiveTimeVisitor)
    }
}

#[test]
fn test_serde_serialize() {
    super::test_encodable_json(serde_json::to_string);
}

#[test]
fn test_serde_deserialize() {
    super::test_decodable_json(|input| serde_json::from_str(input));
}

#[test]
fn test_serde_bincode() {
    // Bincode is relevant to test separately from JSON because
    // it is not self-describing.
    use bincode::{deserialize, serialize};

    let t = NaiveTime::from_hms_nano_opt(3, 5, 7, 98765432).unwrap();
    let encoded = serialize(&t).unwrap();
    let decoded: NaiveTime = deserialize(&encoded).unwrap();
    assert_eq!(t, decoded);
}
