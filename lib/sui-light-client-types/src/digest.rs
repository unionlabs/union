use std::fmt::Display;

use serde::{Deserialize, Serialize};
use unionlabs_primitives::{encoding::Base58, Bytes, FixedBytes};

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Hash, Debug)]
pub struct Digest(pub FixedBytes<32, Base58>);

impl Display for Digest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for Digest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Bytes::<Base58>::new(self.0.get().to_vec()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let b = Bytes::<Base58>::deserialize(deserializer)?;

        Ok(Self(
            b.as_ref().try_into().map_err(serde::de::Error::custom)?,
        ))
    }
}
