use alloc::fmt;
use core::{
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};
use ssz_rs::prelude::*;

#[derive(Default, Clone, Eq, SimpleSerialize, serde::Serialize, serde::Deserialize)]
pub struct ByteVector<const N: usize>(#[serde(with = "crate::serde::as_hex")] Vector<u8, N>);

impl<const N: usize> TryFrom<&[u8]> for ByteVector<N> {
    type Error = ssz_rs::DeserializeError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        ByteVector::<N>::deserialize(bytes)
    }
}

// impl here to satisfy clippy
impl<const N: usize> PartialEq for ByteVector<N> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<const N: usize> Hash for ByteVector<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl<const N: usize> fmt::LowerHex for ByteVector<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_bytes_to_lower_hex(f, self)
    }
}

impl<const N: usize> fmt::Debug for ByteVector<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ByteVector<{N}>({self:#x})")
    }
}

impl<const N: usize> fmt::Display for ByteVector<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:#x}")
    }
}

impl<const N: usize> AsRef<[u8]> for ByteVector<N> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> Deref for ByteVector<N> {
    type Target = Vector<u8, N>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> DerefMut for ByteVector<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn write_bytes_to_lower_hex<T: AsRef<[u8]>>(
    f: &mut fmt::Formatter<'_>,
    data: T,
) -> fmt::Result {
    if f.alternate() {
        write!(f, "0x")?;
    }
    for i in data.as_ref() {
        write!(f, "{i:02x}")?;
    }
    Ok(())
}
