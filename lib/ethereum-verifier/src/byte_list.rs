use super::byte_vector::write_bytes_to_lower_hex;
use ssz_rs::prelude::*;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

#[derive(Default, Clone, Eq, SimpleSerialize, serde::Serialize, serde::Deserialize)]
pub struct ByteList<const N: usize>(#[serde(with = "crate::serde::as_hex")] List<u8, N>);

impl<const N: usize> TryFrom<&[u8]> for ByteList<N> {
    type Error = ssz_rs::DeserializeError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        ByteList::<N>::deserialize(bytes)
    }
}

// impl here to satisfy clippy
impl<const N: usize> PartialEq for ByteList<N> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<const N: usize> Hash for ByteList<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl<const N: usize> fmt::LowerHex for ByteList<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_bytes_to_lower_hex(f, self)
    }
}

impl<const N: usize> fmt::Debug for ByteList<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ByteList<{N}>(len={})({:#x})", self.len(), self)
    }
}

impl<const N: usize> fmt::Display for ByteList<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:#x}")
    }
}

impl<const N: usize> AsRef<[u8]> for ByteList<N> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> Deref for ByteList<N> {
    type Target = List<u8, N>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> DerefMut for ByteList<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
