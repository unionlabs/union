use core::{fmt::Display, ops::Deref};

use serde::{Deserialize, Serialize};
use unionlabs_primitives::{encoding::Base58, Bytes, FixedBytes};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
// TODO(aeryz): serde_as
pub struct Digest(pub FixedBytes<32, Base58>);

impl Serialize for Digest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = Bytes::<Base58>::new(self.0.into_bytes());

        bytes.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = Bytes::<Base58>::deserialize(deserializer)?;

        Ok(Self(FixedBytes::new(bytes.as_ref().try_into().unwrap())))
    }
}

impl Deref for Digest {
    type Target = FixedBytes<32, Base58>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Digest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
