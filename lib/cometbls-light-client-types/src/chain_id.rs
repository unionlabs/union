use std::fmt::Display;

use serde::{
    de::{self, Visitor},
    Deserialize, Serialize,
};
use unionlabs::errors::{ExpectedLength, InvalidLength};
#[cfg(feature = "ethabi")]
use {alloy::core::primitives::FixedBytes, std::string::FromUtf8Error};

/// A CometBLS Chain ID is a maximum 31-byte utf8 string.
///
/// The size limitation is required such that the entire ID will fit in the bn254 scalar field. The *actual* maximum size is 254 bits, but it's truncated down to 31 bytes for simplicity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainId(String);

impl ChainId {
    pub const MAX_LEN: usize = 31;

    pub fn from_string(s: impl Into<String>) -> Result<Self, InvalidLength> {
        let s = s.into();

        if s.len() > Self::MAX_LEN {
            Err(InvalidLength {
                expected: ExpectedLength::Between(0, Self::MAX_LEN),
                found: s.len(),
            })
        } else {
            Ok(Self(s))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[cfg(feature = "ethabi")]
    pub fn try_from_fixed_bytes(bz: FixedBytes<31>) -> Result<Self, FromUtf8Error> {
        String::from_utf8(bz.into_iter().skip_while(|b| *b == 0).collect()).map(Self)
    }

    #[cfg(feature = "ethabi")]
    #[must_use]
    pub fn into_fixed_bytes(self) -> FixedBytes<31> {
        let mut bz = <FixedBytes<31>>::default();

        bz[Self::MAX_LEN - self.0.len()..].copy_from_slice(self.0.as_bytes());

        bz
    }
}

impl Display for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for ChainId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ChainId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ChainIdVisitor;

        impl<'de> Visitor<'de> for ChainIdVisitor {
            type Value = ChainId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string between 0 and 31 bytes")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                ChainId::from_string(v).map_err(|_| {
                    de::Error::invalid_value(
                        de::Unexpected::Str(v),
                        &"a string between 0 and 31 bytes",
                    )
                })
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(v)
            }
        }

        deserializer.deserialize_any(ChainIdVisitor)
    }
}
