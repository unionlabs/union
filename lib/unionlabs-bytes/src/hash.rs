use core::{
    cmp::Ordering,
    fmt::{self, Display},
    marker::PhantomData,
    str::FromStr,
};

use crate::{
    bytes::Bytes,
    encoding::{Encoding, HexPrefixed},
};

#[repr(transparent)]
pub struct Hash<const BYTES: usize, E: Encoding = HexPrefixed> {
    // we abuse deprecated a bit here to make sure this field doesn't get read anywhere it shouldn't, enforcing usage of the constructor instead - this makes sure that the const {} block gets monomorphized, causing a post-mono error if BYTES is 0.
    #[deprecated = "this field should never be used directly, use Hash::new() to construct this type and .get{_mut}() to access the data"]
    arr: [u8; BYTES],
    __marker: PhantomData<fn() -> E>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid length (expected {expected_len}, found {found_len})")]
pub struct FixedBytesError {
    expected_len: usize,
    found_len: usize,
}

// impl<const BYTES: usize, E: Encoding> ssz::Ssz for Hash<BYTES, E>
// where
//     typenum::Const<BYTES>: typenum::ToUInt,
//     typenum::U<BYTES>: typenum::Unsigned + typenum::NonZero,
// {
//     const SSZ_FIXED_LEN: Option<core::num::NonZeroUsize> = <[u8; BYTES] as ssz::Ssz>::SSZ_FIXED_LEN;

//     const TREE_HASH_TYPE: ssz::tree_hash::TreeHashType = <[u8; BYTES] as ssz::Ssz>::TREE_HASH_TYPE;

//     fn tree_hash_root(&self) -> ssz::tree_hash::Hash256 {
//         <[u8; BYTES] as ssz::Ssz>::tree_hash_root(self.get())
//     }

//     fn ssz_append(&self, buf: &mut Vec<u8>) {
//         <[u8; BYTES] as ssz::Ssz>::ssz_append(self.get(), buf);
//     }

//     fn ssz_bytes_len(&self) -> core::num::NonZeroUsize {
//         <[u8; BYTES] as ssz::Ssz>::ssz_bytes_len(self.get())
//     }

//     fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::decode::DecodeError> {
//         <[u8; BYTES] as ssz::Ssz>::from_ssz_bytes(bytes).map(Self::new)
//     }
// }

impl<const BYTES: usize, E: Encoding> AsRef<[u8]> for Hash<BYTES, E> {
    fn as_ref(&self) -> &[u8] {
        self.get()
    }
}

impl<const BYTES: usize, E: Encoding> AsMut<[u8]> for Hash<BYTES, E> {
    fn as_mut(&mut self) -> &mut [u8] {
        self.get_mut()
    }
}

impl<const BYTES: usize, E: Encoding> Clone for Hash<BYTES, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<const BYTES: usize, E: Encoding> Copy for Hash<BYTES, E> {}

impl<const BYTES: usize, E: Encoding> core::hash::Hash for Hash<BYTES, E> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        core::hash::Hash::hash(self.get(), state);
    }
}

impl<const BYTES: usize, E: Encoding> Hash<BYTES, E> {
    pub const BITS_LEN: usize = BYTES * 8;
    pub const BYTES_LEN: usize = BYTES;

    #[must_use = "constructing a Hash has no effect"]
    pub const fn new(arr: [u8; BYTES]) -> Self {
        const { assert!(BYTES > 0, "BYTES must be greater than 0") };

        #[expect(deprecated)] // this is the (sole) constructor
        Self {
            arr,
            __marker: PhantomData,
        }
    }

    #[must_use = "constructing a Hash has no effect"]
    pub const fn new_ref(arr: &[u8; BYTES]) -> &Self {
        const { assert!(BYTES > 0, "BYTES must be greater than 0") };

        // SAFETY: Hash has the same layout as [u8; BYTES], guaranteed by repr(transparent)
        unsafe { &*core::ptr::from_ref::<[u8; BYTES]>(arr).cast::<Self>() }
    }

    #[must_use = "reading the inner value has no effect"]
    #[inline]
    pub fn get(&self) -> &[u8; BYTES] {
        #[expect(deprecated)] // this is the (sole) immutable accessor
        &self.arr
    }

    #[must_use = "reading the inner value has no effect"]
    #[inline]
    pub fn get_mut(&mut self) -> &mut [u8; BYTES] {
        #[expect(deprecated)] // this is the (sole) mutable accessor
        &mut self.arr
    }

    #[must_use]
    // TODO: Make this return `Bytes`
    pub fn into_bytes(self) -> Bytes<E> {
        self.get().to_vec().into()
    }

    pub fn iter(&self) -> core::slice::Iter<'_, u8> {
        <&Self as IntoIterator>::into_iter(self)
    }

    #[must_use = "converting a hash to a hash with a different encoding has no effect"]
    #[inline]
    pub fn into_encoding<E2: Encoding>(&self) -> Hash<BYTES, E2> {
        Hash::new(*self.get())
    }

    #[must_use = "converting a hash to a hash with a different encoding has no effect"]
    #[inline]
    pub fn as_encoding<E2: Encoding>(&self) -> &Hash<BYTES, E2> {
        Hash::new_ref(self.get())
    }

    #[must_use]
    pub fn is_zero(&self) -> bool {
        self == &Self::default()
    }
}

impl<const BYTES: usize, E: Encoding> fmt::Debug for Hash<BYTES, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Hash<{BYTES}>({self})"))
    }
}

impl<const BYTES: usize, E: Encoding> Display for Hash<BYTES, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        E::fmt(self.get(), f)
    }
}

impl<const BYTES: usize, E: Encoding, RhsE: Encoding> PartialEq<Hash<BYTES, RhsE>>
    for Hash<BYTES, E>
{
    fn eq(&self, other: &Hash<BYTES, RhsE>) -> bool {
        self.get() == other.get()
    }
}

impl<const BYTES: usize, E: Encoding> Eq for Hash<BYTES, E> {}

impl<const BYTES: usize, E: Encoding, RhsE: Encoding> PartialOrd<Hash<BYTES, RhsE>>
    for Hash<BYTES, E>
{
    fn partial_cmp(&self, other: &Hash<BYTES, RhsE>) -> Option<Ordering> {
        self.get().partial_cmp(other.get())
    }
}

impl<const BYTES: usize, E: Encoding> Ord for Hash<BYTES, E> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get().cmp(other.get())
    }
}

#[cfg(feature = "serde")]
impl<const BYTES: usize, E: Encoding> serde::Serialize for Hash<BYTES, E> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            use serde::ser::SerializeTupleStruct;

            let mut s = serializer.serialize_tuple_struct("Hash", BYTES)?;
            for b in self.get() {
                s.serialize_field(&b)?;
            }
            s.end()
        }
    }
}

#[cfg(feature = "serde")]
impl<'de, const BYTES: usize, E: Encoding> serde::Deserialize<'de> for Hash<BYTES, E> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            String::deserialize(deserializer)
                .and_then(|s| s.parse().map_err(::serde::de::Error::custom))
        } else {
            struct ArrayVisitor<const N: usize>;

            impl<'de, const N: usize> serde::de::Visitor<'de> for ArrayVisitor<N> {
                type Value = [u8; N];

                fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    write!(formatter, "an array of length {N}")
                }

                fn visit_seq<A>(self, mut seq: A) -> ::core::result::Result<[u8; N], A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let mut arr = [0_u8; N];

                    for (i, b) in arr.iter_mut().enumerate() {
                        let val = seq
                            .next_element()?
                            .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;

                        *b = val;
                    }

                    Ok(arr)
                }
            }

            Ok(Self::new(
                deserializer.deserialize_tuple(BYTES, ArrayVisitor::<BYTES>)?,
            ))
        }
    }
}

impl<const BYTES: usize, E: Encoding> FromStr for Hash<BYTES, E> {
    type Err = E::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut out = [0; BYTES];

        E::decode_to_slice(s, &mut out)?;

        Ok(Self::new(out))
    }
}

// #[derive(DebugNoBound, thiserror::Error)]
// pub enum HashDecodeError<E: Encoding> {
//     #[error("invalid encoding")]
//     InvalidEncoding(#[source] E::Error),
//     #[error("invalid length")]
//     FixedBytesError(#[from] FixedBytesError),
// }

impl<E: Encoding, const BYTES: usize> Default for Hash<BYTES, E> {
    fn default() -> Self {
        Self::new([0_u8; BYTES])
    }
}

impl<'a, E: Encoding, const BYTES: usize> IntoIterator for &'a Hash<BYTES, E> {
    type Item = &'a u8;
    type IntoIter = core::slice::Iter<'a, u8>;

    fn into_iter(self) -> core::slice::Iter<'a, u8> {
        self.get().iter()
    }
}

impl<E: Encoding, const BYTES: usize> IntoIterator for Hash<BYTES, E> {
    type Item = u8;
    type IntoIter = core::array::IntoIter<u8, BYTES>;

    fn into_iter(self) -> Self::IntoIter {
        (*self.get()).into_iter()
    }
}

impl<E: Encoding, const BYTES: usize> TryFrom<Vec<u8>> for Hash<BYTES, E> {
    type Error = FixedBytesError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        value
            .try_into()
            .map(Self::new)
            .map_err(|invalid| FixedBytesError {
                expected_len: BYTES,
                found_len: invalid.len(),
            })
    }
}

impl<E: Encoding, const BYTES: usize> TryFrom<&Vec<u8>> for Hash<BYTES, E> {
    type Error = FixedBytesError;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        value
            .as_slice()
            .try_into()
            .map(Self::new)
            .map_err(|_| FixedBytesError {
                expected_len: BYTES,
                found_len: value.len(),
            })
    }
}

impl<E: Encoding, const BYTES: usize> TryFrom<&[u8]> for Hash<BYTES, E> {
    type Error = FixedBytesError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        value
            .try_into()
            .map(Self::new)
            .map_err(|_| FixedBytesError {
                expected_len: BYTES,
                found_len: value.len(),
            })
    }
}

impl<E: Encoding, const BYTES: usize> From<Hash<BYTES, E>> for Vec<u8> {
    fn from(value: Hash<BYTES, E>) -> Self {
        value.get().into()
    }
}

impl<E: Encoding, const BYTES: usize> From<Hash<BYTES, E>> for [u8; BYTES] {
    fn from(value: Hash<BYTES, E>) -> Self {
        *value.get()
    }
}

impl<E: Encoding, const BYTES: usize> From<[u8; BYTES]> for Hash<BYTES, E> {
    fn from(value: [u8; BYTES]) -> Self {
        Self::new(value)
    }
}

#[cfg(feature = "rlp")]
impl<E: Encoding, const BYTES: usize> rlp::Decodable for Hash<BYTES, E> {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        rlp.decoder()
            .decode_value(|bytes| match bytes.len().cmp(&BYTES) {
                core::cmp::Ordering::Less => Err(::rlp::DecoderError::RlpIsTooShort),
                core::cmp::Ordering::Greater => Err(::rlp::DecoderError::RlpIsTooBig),
                core::cmp::Ordering::Equal => {
                    Ok(Self::new(bytes.try_into().expect("size is checked; qed;")))
                }
            })
    }
}

#[cfg(feature = "rlp")]
impl<E: Encoding, const BYTES: usize> rlp::Encodable for Hash<BYTES, E> {
    fn rlp_append(&self, s: &mut ::rlp::RlpStream) {
        s.encoder().encode_value(self.as_ref());
    }
}

#[cfg(feature = "alloy-primitives-compat")]
impl<E: Encoding, const BYTES: usize> From<alloy::core::primitives::FixedBytes<BYTES>>
    for Hash<BYTES, E>
{
    fn from(value: alloy::core::primitives::FixedBytes<BYTES>) -> Self {
        value.0.into()
    }
}

#[cfg(feature = "alloy-primitives-compat")]
impl<E: Encoding> From<Hash<20, E>> for alloy::core::primitives::Address {
    fn from(value: Hash<20, E>) -> Self {
        value.get().into()
    }
}

#[cfg(feature = "alloy-primitives-compat")]
impl<E: Encoding> From<alloy::core::primitives::Address> for Hash<20, E> {
    fn from(value: alloy::core::primitives::Address) -> Self {
        value.0.into()
    }
}

#[cfg(feature = "alloy-primitives-compat")]
impl<E: Encoding, const BYTES: usize> From<Hash<BYTES, E>>
    for alloy::core::primitives::FixedBytes<BYTES>
{
    fn from(value: Hash<BYTES, E>) -> Self {
        value.get().into()
    }
}

#[cfg(test)]
mod tests {
    use hex::FromHexError;

    use super::*;
    use crate::encoding::{Base64, Base64Error, HexPrefixedFromStrError, HexUnprefixed};

    const BASE64_STR: &str = "YWJjZA==";
    const HEX_PREFIXED_STR: &str = "0x61626364";
    const HEX_UNPREFIXED_STR: &str = "61626364";

    const RAW_VALUE: &[u8; 4] = b"abcd";

    #[test]
    fn hex_prefixed() {
        type H = Hash<4, HexPrefixed>;

        let decoded = H::from_str(HEX_PREFIXED_STR).unwrap();

        assert_eq!(HEX_PREFIXED_STR, decoded.to_string());

        assert_eq!(decoded.get(), b"abcd");
    }

    #[test]
    fn hex_prefixed_too_long() {
        type H = Hash<3, HexPrefixed>;

        assert_eq!(
            H::from_str(HEX_PREFIXED_STR),
            Err(HexPrefixedFromStrError::InvalidHex(
                FromHexError::InvalidStringLength
            ))
        );
    }

    #[test]
    fn hex_prefixed_too_short() {
        type H = Hash<5, HexPrefixed>;

        assert_eq!(
            H::from_str(HEX_PREFIXED_STR),
            Err(HexPrefixedFromStrError::InvalidHex(
                FromHexError::InvalidStringLength
            ))
        );
    }

    #[test]
    fn hex_unprefixed() {
        type H = Hash<4, HexUnprefixed>;

        let decoded = H::from_str(HEX_UNPREFIXED_STR).unwrap();

        assert_eq!(HEX_UNPREFIXED_STR, decoded.to_string());

        assert_eq!(decoded.get(), b"abcd");
    }

    #[test]
    fn hex_unprefixed_too_long() {
        type H = Hash<3, HexUnprefixed>;

        assert_eq!(
            H::from_str(HEX_UNPREFIXED_STR),
            Err(FromHexError::InvalidStringLength)
        );
    }

    #[test]
    fn hex_unprefixed_too_short() {
        type H = Hash<5, HexUnprefixed>;

        assert_eq!(
            H::from_str(HEX_UNPREFIXED_STR),
            Err(FromHexError::InvalidStringLength)
        );
    }

    #[test]
    fn base64() {
        type H = Hash<4, Base64>;

        let decoded = H::from_str(BASE64_STR).unwrap();

        assert_eq!(BASE64_STR, decoded.to_string());

        assert_eq!(decoded.get(), RAW_VALUE);
    }

    #[test]
    fn base64_too_long() {
        type H = Hash<3, Base64>;

        assert_eq!(
            H::from_str(BASE64_STR),
            Err(Base64Error::InvalidLength {
                expected_len: 3,
                found_len: 4
            })
        );
    }

    #[test]
    fn base64_too_short() {
        type H = Hash<5, Base64>;

        assert_eq!(
            H::from_str(BASE64_STR),
            Err(Base64Error::InvalidLength {
                expected_len: 5,
                found_len: 4
            })
        );
    }

    #[test]
    fn new_ref() {
        let arr = &[1, 2, 3];

        assert_eq!(<Hash<3, HexPrefixed>>::new_ref(arr).get(), arr);
    }
}
