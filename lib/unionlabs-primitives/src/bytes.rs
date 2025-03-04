use alloc::borrow::Cow;
use core::{
    array::TryFromSliceError, cmp::Ordering, fmt, marker::PhantomData, ops::Deref, str::FromStr,
};

use crate::{
    encoding::{Encoding, HexPrefixed},
    fixed_bytes::FixedBytesError,
};

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

impl<E: Encoding> FromIterator<u8> for Bytes<E> {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        Self::new(iter.into_iter().collect::<Vec<_>>())
    }
}

impl<'a, E: Encoding> FromIterator<&'a u8> for Bytes<E> {
    fn from_iter<T: IntoIterator<Item = &'a u8>>(iter: T) -> Self {
        Self::new(iter.into_iter().copied().collect::<Vec<_>>())
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

    #[must_use = "converting bytes to bytes with a different encoding has no effect"]
    #[inline]
    pub fn into_encoding<E2: Encoding>(self) -> Bytes<E2> {
        Bytes::new(self.bytes)
    }

    #[must_use = "converting bytes to bytes with a different encoding has no effect"]
    #[inline]
    pub fn as_encoding<E2: Encoding>(&self) -> &Bytes<E2> {
        unsafe { &*core::ptr::from_ref::<Bytes<E>>(self).cast::<Bytes<E2>>() }
    }

    #[must_use = "converting to a vec has no effect"]
    pub fn into_vec(self) -> Vec<u8> {
        self.bytes.into()
    }

    // TODO: Benchmark and optimize if needed
    pub fn extend_from_slice(&mut self, other: &[u8]) {
        let mut vec = self.clone().into_vec();

        vec.extend_from_slice(other);

        *self = Self::new(vec);
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

impl<E: Encoding> PartialEq<Vec<u8>> for Bytes<E> {
    fn eq(&self, other: &Vec<u8>) -> bool {
        (**self).eq(&**other)
    }
}

impl<E: Encoding> PartialEq<&[u8]> for Bytes<E> {
    fn eq(&self, other: &&[u8]) -> bool {
        (**self).eq(*other)
    }
}

impl<E: Encoding> PartialEq<[u8]> for Bytes<E> {
    fn eq(&self, other: &[u8]) -> bool {
        (**self).eq(other)
    }
}

impl<E: Encoding, const N: usize> PartialEq<[u8; N]> for Bytes<E> {
    fn eq(&self, other: &[u8; N]) -> bool {
        (**self).eq(other)
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

#[cfg(feature = "serde")]
impl<E: Encoding> serde::Serialize for Bytes<E> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            <serde_bytes::Bytes>::new(self).serialize(serializer)
        }
    }
}

#[cfg(feature = "serde")]
impl<'de, E: Encoding> serde::Deserialize<'de> for Bytes<E> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            String::deserialize(deserializer)
                .and_then(|s| s.parse().map_err(::serde::de::Error::custom))
        } else {
            <&serde_bytes::Bytes>::deserialize(deserializer).map(|b| Bytes::new(b.to_vec()))
        }
    }
}

#[cfg(feature = "bincode")]
impl<Enc: Encoding> bincode::Encode for Bytes<Enc> {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.bytes.encode(encoder)
    }
}

#[cfg(feature = "bincode")]
impl<Enc: Encoding> bincode::Decode for Bytes<Enc> {
    fn decode<D: bincode::de::Decoder>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Ok(Self::from(<Vec<u8> as bincode::Decode>::decode(decoder)?))
    }
}

#[cfg(feature = "bincode")]
impl<'de, Enc: Encoding> bincode::BorrowDecode<'de> for Bytes<Enc> {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        bincode::Decode::decode(decoder)
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

impl<EBytes: Encoding, EHash: Encoding, const BYTES: usize> TryFrom<Bytes<EBytes>>
    for crate::fixed_bytes::FixedBytes<BYTES, EHash>
{
    type Error = FixedBytesError;

    fn try_from(value: Bytes<EBytes>) -> Result<Self, Self::Error> {
        Self::try_from(value.into_vec())
    }
}

impl<E: Encoding, const BYTES: usize> TryFrom<Bytes<E>> for [u8; BYTES] {
    type Error = Bytes<E>;

    fn try_from(value: Bytes<E>) -> Result<Self, Self::Error> {
        Self::try_from(value.into_vec()).map_err(Bytes::new)
    }
}

impl<E: Encoding, const BYTES: usize> TryFrom<&Bytes<E>> for [u8; BYTES] {
    type Error = TryFromSliceError;

    fn try_from(value: &Bytes<E>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_ref())
    }
}

impl<E: Encoding> From<Vec<u8>> for Bytes<E> {
    fn from(value: Vec<u8>) -> Self {
        Self::new(value)
    }
}

impl<E: Encoding> From<Bytes<E>> for Vec<u8> {
    fn from(value: Bytes<E>) -> Self {
        value.deref().into()
    }
}

impl<E: Encoding> From<Cow<'static, [u8]>> for Bytes<E> {
    fn from(value: Cow<'static, [u8]>) -> Self {
        Self::new(value)
    }
}

impl<E: Encoding> From<Bytes<E>> for Cow<'static, [u8]> {
    fn from(value: Bytes<E>) -> Self {
        value.bytes
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

impl<E: Encoding, const N: usize> From<&[u8; N]> for Bytes<E> {
    fn from(value: &[u8; N]) -> Self {
        Self::new(value.as_slice().to_owned())
    }
}

impl<E: Encoding, const N: usize> From<[u8; N]> for Bytes<E> {
    fn from(value: [u8; N]) -> Self {
        Self::new(value.as_slice().to_owned())
    }
}

impl<EBytes: Encoding, EHash: Encoding, const N: usize>
    From<crate::fixed_bytes::FixedBytes<N, EHash>> for Bytes<EBytes>
{
    fn from(value: crate::fixed_bytes::FixedBytes<N, EHash>) -> Self {
        Self::new(value.get().as_slice().to_owned())
    }
}

// // TODO: Feature gate rlp across the crate
// // #[cfg(feature = "rlp")]
// impl<E: Encoding> rlp::Decodable for Bytes<E> {
//     fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
//         rlp.decoder()
//             .decode_value(|bytes| Ok(Self::new(bytes.to_owned())))
//     }
// }

// // TODO: Feature gate rlp across the crate
// // #[cfg(feature = "rlp")]
// impl<E: Encoding> rlp::Encodable for Bytes<E> {
//     fn rlp_append(&self, s: &mut ::rlp::RlpStream) {
//         s.encoder().encode_value(self.as_ref());
//     }
// }

// #[cfg(feature = "ethabi")]
// impl<E: Encoding> From<alloy::core::primitives::Bytes> for Bytes<E> {
//     fn from(value: alloy::core::primitives::Bytes) -> Self {
//         value.0.to_vec().into()
//     }
// }

// #[cfg(feature = "ethabi")]
// impl<E: Encoding> From<Bytes<E>> for alloy::core::primitives::Bytes {
//     fn from(value: Bytes<E>) -> Self {
//         value.deref().to_owned().into()
//     }
// }

#[cfg(feature = "schemars")]
impl<E: Encoding> schemars::JsonSchema for Bytes<E> {
    fn schema_name() -> String {
        format!("Bytes<{}>", E::NAME)
    }

    fn schema_id() -> alloc::borrow::Cow<'static, str> {
        format!("{}::{}", module_path!(), Self::schema_name()).into()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        use schemars::schema::{InstanceType, Metadata, SchemaObject, SingleOrVec};

        SchemaObject {
            metadata: Some(Box::new(Metadata {
                description: Some(format!(
                    "A string representation of bytes, encoded via {}",
                    E::NAME
                )),
                ..Default::default()
            })),
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::String))),
            ..Default::default()
        }
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::{Base64, HexUnprefixed};

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

    #[test]
    fn as_encoding() {
        let bz = <Bytes>::new(b"bytes");
        let bz_base64 = bz.as_encoding::<Base64>();
        assert_eq!(bz_base64, &bz);
    }
}
