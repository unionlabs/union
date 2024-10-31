use alloc::borrow::Cow;
use core::{cmp::Ordering, fmt, marker::PhantomData, ops::Deref, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::hash::hash_v2::{Encoding, HexPrefixed};

pub struct Bytes<E: Encoding = HexPrefixed> {
    bytes: Cow<'static, [u8]>,
    __marker: PhantomData<fn() -> E>,
}

impl<E: Encoding> AsRef<[u8]> for Bytes<E> {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl<E: Encoding> Clone for Bytes<E> {
    fn clone(&self) -> Self {
        Self::new(self.bytes.clone())
    }
}

impl<E: Encoding> core::hash::Hash for Bytes<E> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        core::hash::Hash::hash(&**self, state);
    }
}

impl<E: Encoding> Bytes<E> {
    #[must_use = "constructing a Bytes has no effect"]
    pub fn new(bytes: impl Into<Cow<'static, [u8]>>) -> Self {
        Self {
            bytes: bytes.into(),
            __marker: PhantomData,
        }
    }

    #[must_use = "constructing a Bytes has no effect"]
    pub const fn new_static(bytes: &'static [u8]) -> Self {
        Self {
            bytes: Cow::Borrowed(bytes),
            __marker: PhantomData,
        }
    }

    pub fn iter(&self) -> core::slice::Iter<'_, u8> {
        <&Self as IntoIterator>::into_iter(self)
    }

    #[must_use = "converting a hash to a hash with a different encoding has no effect"]
    #[inline]
    pub fn into_encoding<E2: Encoding>(self) -> Bytes<E2> {
        Bytes::new(self.bytes)
    }

    #[must_use = "converting to a vec has no effect"]
    pub fn into_vec(self) -> Vec<u8> {
        self.bytes.into()
    }
}

impl<E: Encoding> Deref for Bytes<E> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

impl<E: Encoding> fmt::Debug for Bytes<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Bytes({self})"))
    }
}

impl<E: Encoding> fmt::Display for Bytes<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        E::fmt(self, f)
    }
}

impl<E: Encoding, RhsE: Encoding> PartialEq<Bytes<RhsE>> for Bytes<E> {
    fn eq(&self, other: &Bytes<RhsE>) -> bool {
        (**self).eq(&**other)
    }
}

impl<E: Encoding> Eq for Bytes<E> {}

impl<E: Encoding, RhsE: Encoding> PartialOrd<Bytes<RhsE>> for Bytes<E> {
    fn partial_cmp(&self, other: &Bytes<RhsE>) -> Option<Ordering> {
        (**self).partial_cmp(&**other)
    }
}

impl<E: Encoding> Ord for Bytes<E> {
    fn cmp(&self, other: &Self) -> Ordering {
        (**self).cmp(&**other)
    }
}

impl<E: Encoding> Serialize for Bytes<E> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            <serde_bytes::Bytes>::new(self).serialize(serializer)
        }
    }
}

impl<'de, E: Encoding> Deserialize<'de> for Bytes<E> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            String::deserialize(deserializer)
                .and_then(|s| s.parse().map_err(::serde::de::Error::custom))
        } else {
            <&serde_bytes::Bytes>::deserialize(deserializer).map(|b| Bytes::new(b.to_vec()))
        }
    }
}

impl<E: Encoding> FromStr for Bytes<E> {
    type Err = E::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        E::decode(s).map(Self::new)
    }
}

impl<E: Encoding> Default for Bytes<E> {
    fn default() -> Self {
        Self::new_static(&[])
    }
}

impl<'a, E: Encoding> IntoIterator for &'a Bytes<E> {
    type Item = &'a u8;
    type IntoIter = core::slice::Iter<'a, u8>;

    fn into_iter(self) -> core::slice::Iter<'a, u8> {
        (**self).iter()
    }
}

impl<E: Encoding> IntoIterator for Bytes<E> {
    type Item = u8;
    type IntoIter = alloc::vec::IntoIter<u8>;

    #[allow(clippy::unnecessary_to_owned)]
    fn into_iter(self) -> Self::IntoIter {
        self.bytes.to_vec().into_iter()
    }
}

impl<E: Encoding> From<Vec<u8>> for Bytes<E> {
    fn from(value: Vec<u8>) -> Self {
        Self::new(value)
    }
}

impl<E: Encoding> From<&Vec<u8>> for Bytes<E> {
    fn from(value: &Vec<u8>) -> Self {
        Self::new(value.to_owned())
    }
}

impl<E: Encoding> From<&[u8]> for Bytes<E> {
    fn from(value: &[u8]) -> Self {
        Self::new(value.to_owned())
    }
}

impl<E: Encoding> From<Bytes<E>> for Vec<u8> {
    fn from(value: Bytes<E>) -> Self {
        value.deref().into()
    }
}

// TODO: Feature gate rlp across the crate
// #[cfg(feature = "rlp")]
impl<E: Encoding> rlp::Decodable for Bytes<E> {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        rlp.decoder()
            .decode_value(|bytes| Ok(Self::new(bytes.to_owned())))
    }
}

// TODO: Feature gate rlp across the crate
// #[cfg(feature = "rlp")]
impl<E: Encoding> rlp::Encodable for Bytes<E> {
    fn rlp_append(&self, s: &mut ::rlp::RlpStream) {
        s.encoder().encode_value(self.as_ref());
    }
}

#[cfg(feature = "ethabi")]
impl<E: Encoding> From<alloy::core::primitives::Bytes> for Bytes<E> {
    fn from(value: alloy::core::primitives::Bytes) -> Self {
        value.0.to_vec().into()
    }
}

#[cfg(feature = "ethabi")]
impl<E: Encoding> From<Bytes<E>> for alloy::core::primitives::Bytes {
    fn from(value: Bytes<E>) -> Self {
        value.deref().to_owned().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::hash_v2::{Base64, HexUnprefixed};

    const BASE64_STR: &str = "YWJjZA==";
    const HEX_PREFIXED_STR: &str = "0x61626364";
    const HEX_UNPREFIXED_STR: &str = "61626364";

    const RAW_VALUE: &[u8; 4] = b"abcd";

    #[test]
    fn hex_prefixed() {
        let decoded = <Bytes<HexPrefixed>>::from_str(HEX_PREFIXED_STR).unwrap();

        assert_eq!(HEX_PREFIXED_STR, decoded.to_string());

        assert_eq!(&*decoded, b"abcd");
    }

    #[test]
    fn hex_unprefixed() {
        let decoded = <Bytes<HexUnprefixed>>::from_str(HEX_UNPREFIXED_STR).unwrap();

        assert_eq!(HEX_UNPREFIXED_STR, decoded.to_string());

        assert_eq!(&*decoded, b"abcd");
    }

    #[test]
    fn base64() {
        let decoded = <Bytes<Base64>>::from_str(BASE64_STR).unwrap();

        assert_eq!(BASE64_STR, decoded.to_string());

        assert_eq!(&*decoded, RAW_VALUE);
    }
}
