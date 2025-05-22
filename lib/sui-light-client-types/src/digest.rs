use core::fmt::Display;

use unionlabs_primitives::{encoding::Base58, Bytes, FixedBytes};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Digest(pub FixedBytes<32, Base58>);

#[cfg(feature = "serde")]
impl serde::Serialize for Digest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = Bytes::<Base58>::new(self.0.into_bytes());

        bytes.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Digest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = Bytes::<Base58>::deserialize(deserializer)?;

        Ok(Self(FixedBytes::new(bytes.as_ref().try_into().unwrap())))
    }
}

impl Display for Digest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
