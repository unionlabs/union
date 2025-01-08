use std::fmt::Display;

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

#[cfg(feature = "serde")]
impl serde::Serialize for ChainId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for ChainId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <String as serde::Deserialize>::deserialize(deserializer)?;

        ChainId::from_string(s).map_err(serde::de::Error::custom)
    }
}
