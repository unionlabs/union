use crate::{errors::Error, internal_prelude::*};
use core::{
    fmt::Display,
    ops::{Add, Deref, DerefMut, Div, Mul, Sub},
};
use ssz_rs::{Deserialize, List};
use ssz_rs_derive::SimpleSerialize;

#[derive(
    Clone, Debug, PartialEq, Eq, Default, SimpleSerialize, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct H256(#[serde(with = "serde_hex")] pub [u8; 32]);

impl H256 {
    pub fn from_slice(bz: &[u8]) -> Self {
        let mut b: [u8; 32] = Default::default();
        b.copy_from_slice(bz);
        Self(b)
    }

    pub fn from_hex(s: &str) -> Result<Self, Error> {
        Ok(Self::from_slice(&hex::decode(
            s.strip_prefix("0x").unwrap_or(s),
        )?))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn is_zero(&self) -> bool {
        self.0 == <[u8; 32]>::default()
    }
}

impl From<[u8; 32]> for H256 {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

impl From<H256> for [u8; 32] {
    fn from(value: H256) -> Self {
        value.0
    }
}

impl Display for H256 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(hex::encode(self.0).as_str())
    }
}

impl From<H256> for primitive_types::H256 {
    fn from(value: H256) -> Self {
        primitive_types::H256::from_slice(value.as_bytes())
    }
}

pub type Bytes32 = H256;

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, Default, SimpleSerialize, serde::Serialize, PartialOrd, Ord,
)]
#[serde(transparent)]
pub struct U64(pub u64);

impl From<U64> for u64 {
    fn from(value: U64) -> Self {
        value.0
    }
}

impl Display for U64 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for U64 {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u64> for U64 {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl Add<u64> for U64 {
    type Output = U64;
    fn add(self, rhs: u64) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Add for U64 {
    type Output = U64;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for U64 {
    type Output = U64;
    fn sub(self, rhs: U64) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Sub<u64> for U64 {
    type Output = U64;
    fn sub(self, rhs: u64) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl Mul for U64 {
    type Output = U64;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Div for U64 {
    type Output = U64;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl<'de> serde::Deserialize<'de> for U64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct MyVisitor;

        impl<'de> serde::de::Visitor<'de> for MyVisitor {
            type Value = U64;

            fn expecting(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                fmt.write_str("integer or string")
            }

            fn visit_u64<E>(self, val: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(U64(val))
            }

            fn visit_str<E>(self, val: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match val.parse::<u64>() {
                    Ok(val) => self.visit_u64(val),
                    Err(_) => Err(E::custom("failed to parse integer")),
                }
            }
        }

        deserializer.deserialize_any(MyVisitor)
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Default,
    SimpleSerialize,
    serde::Serialize,
    serde::Deserialize,
    PartialOrd,
    Ord,
)]
pub struct U256(ssz_rs::U256);

impl From<ssz_rs::U256> for U256 {
    fn from(value: ssz_rs::U256) -> Self {
        Self(value)
    }
}

impl Deref for U256 {
    type Target = ssz_rs::U256;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl U256 {
    pub fn try_from_bytes_le(bytes: &[u8]) -> Result<Self, Error> {
        Ok(ssz_rs::U256::deserialize(bytes)?.into())
    }

    pub fn from_bytes_le(bytes: [u8; 32]) -> Self {
        ssz_rs::U256::from_bytes_le(bytes).into()
    }
}

#[derive(
    Clone, Debug, PartialEq, Eq, Default, SimpleSerialize, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct Address(#[serde(with = "serde_hex")] pub [u8; 20]);

impl TryFrom<&[u8]> for Address {
    type Error = Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() == 20 {
            let mut addr: [u8; 20] = Default::default();
            addr.copy_from_slice(value);
            Ok(Self(addr))
        } else {
            Err(Error::InvalidAddressLength(20, value.len()))
        }
    }
}

#[derive(
    Clone, Debug, PartialEq, Eq, Default, SimpleSerialize, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct ByteVector<const N: usize>(
    #[serde(with = "serde_hex")] pub(crate) ssz_rs::Vector<u8, N>,
);

impl<const N: usize> TryFrom<&[u8]> for ByteVector<N> {
    type Error = ssz_rs::DeserializeError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        ByteVector::<N>::deserialize(bytes)
    }
}

impl<const N: usize> AsRef<[u8]> for ByteVector<N> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> Deref for ByteVector<N> {
    type Target = ssz_rs::Vector<u8, N>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> DerefMut for ByteVector<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(
    Clone, Debug, PartialEq, Eq, Default, SimpleSerialize, serde::Serialize, serde::Deserialize,
)]
pub struct ByteList<const N: usize>(#[serde(with = "serde_hex")] pub(crate) List<u8, N>);

impl<const N: usize> TryFrom<&[u8]> for ByteList<N> {
    type Error = ssz_rs::DeserializeError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        ByteList::<N>::deserialize(bytes)
    }
}

#[allow(clippy::derived_hash_with_manual_eq)]
impl<const N: usize> core::hash::Hash for ByteList<N> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
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

pub mod serde_hex {
    use super::*;
    use serde::de::Deserialize;
    pub fn serialize<S, T: AsRef<[u8]>>(data: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let encoding = hex::encode(data.as_ref());
        let output = alloc::format!("0x{encoding}");
        serializer.collect_str(&output)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: TryFrom<alloc::vec::Vec<u8>>,
    {
        let s = <String>::deserialize(deserializer)?;

        let data =
            hex::decode(s.strip_prefix("0x").unwrap_or(&s)).map_err(serde::de::Error::custom)?;

        let inner = T::try_from(data)
            .map_err(|_| serde::de::Error::custom("type failed to parse bytes from hex data"))?;
        Ok(inner)
    }
}
