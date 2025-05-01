use core::{
    borrow::Borrow,
    cmp::Ordering,
    fmt::{self, Display},
    marker::PhantomData,
    slice::SliceIndex,
    str::FromStr,
};

use crate::{
    encoding::{Encoding, HexPrefixed},
    Bytes,
};

#[repr(transparent)]
pub struct FixedBytes<const BYTES: usize, E: Encoding = HexPrefixed> {
    // we abuse deprecated a bit here to make sure this field doesn't get read anywhere it shouldn't, enforcing usage of the constructor instead - this makes sure that the const {} block gets monomorphized, causing a post-mono error if BYTES is 0.
    #[deprecated = "this field should never be used directly, use Hash::new() to construct this type and .get{_mut}() to access the data"]
    arr: [u8; BYTES],
    __marker: PhantomData<fn() -> E>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid length (expected {expected_len}, found {found_len})")]
#[allow(clippy::module_name_repetitions)]
pub struct FixedBytesError {
    pub expected_len: usize,
    pub found_len: usize,
}

impl<const BYTES: usize, E: Encoding> AsRef<[u8]> for FixedBytes<BYTES, E> {
    fn as_ref(&self) -> &[u8] {
        self.get()
    }
}

impl<const BYTES: usize, E: Encoding> AsMut<[u8]> for FixedBytes<BYTES, E> {
    fn as_mut(&mut self) -> &mut [u8] {
        self.get_mut()
    }
}

impl<const BYTES: usize, E: Encoding> Clone for FixedBytes<BYTES, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<const BYTES: usize, E: Encoding> Copy for FixedBytes<BYTES, E> {}

impl<const BYTES: usize, E: Encoding> core::hash::Hash for FixedBytes<BYTES, E> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        core::hash::Hash::hash(self.get(), state);
    }
}

impl<const BYTES: usize, E: Encoding, I: SliceIndex<[u8]>> core::ops::Index<I>
    for FixedBytes<BYTES, E>
{
    type Output = <I as SliceIndex<[u8]>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.get()[index]
    }
}

impl<const BYTES: usize, E: Encoding, I: SliceIndex<[u8]>> core::ops::IndexMut<I>
    for FixedBytes<BYTES, E>
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.get_mut()[index]
    }
}

impl<const BYTES: usize, E: Encoding> FixedBytes<BYTES, E> {
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
    pub fn into_bytes(self) -> Bytes<E> {
        self.get().to_vec().into()
    }

    pub fn iter(&self) -> core::slice::Iter<'_, u8> {
        <&Self as IntoIterator>::into_iter(self)
    }

    #[must_use = "converting a hash to a hash with a different encoding has no effect"]
    #[inline]
    pub fn into_encoding<E2: Encoding>(&self) -> FixedBytes<BYTES, E2> {
        FixedBytes::new(*self.get())
    }

    #[must_use = "converting a hash to a hash with a different encoding has no effect"]
    #[inline]
    pub fn as_encoding<E2: Encoding>(&self) -> &FixedBytes<BYTES, E2> {
        FixedBytes::new_ref(self.get())
    }

    #[must_use]
    pub fn is_zero(&self) -> bool {
        self == &Self::default()
    }
}

impl<const BYTES: usize, E: Encoding> fmt::Debug for FixedBytes<BYTES, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FixedBytes<{BYTES}>({self})"))
    }
}

impl<const BYTES: usize, E: Encoding> Display for FixedBytes<BYTES, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        E::fmt(self.get(), f)
    }
}

impl<const BYTES: usize, E: Encoding, RhsE: Encoding> PartialEq<FixedBytes<BYTES, RhsE>>
    for FixedBytes<BYTES, E>
{
    fn eq(&self, other: &FixedBytes<BYTES, RhsE>) -> bool {
        self.get() == other.get()
    }
}

impl<const BYTES: usize, E: Encoding> Eq for FixedBytes<BYTES, E> {}

impl<const BYTES: usize, E: Encoding, RhsE: Encoding> PartialOrd<FixedBytes<BYTES, RhsE>>
    for FixedBytes<BYTES, E>
{
    fn partial_cmp(&self, other: &FixedBytes<BYTES, RhsE>) -> Option<Ordering> {
        self.get().partial_cmp(other.get())
    }
}

impl<const BYTES: usize, E: Encoding> Ord for FixedBytes<BYTES, E> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get().cmp(other.get())
    }
}

#[cfg(feature = "serde")]
impl<const BYTES: usize, E: Encoding> serde::Serialize for FixedBytes<BYTES, E> {
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
impl<'de, const BYTES: usize, E: Encoding> serde::Deserialize<'de> for FixedBytes<BYTES, E> {
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

impl<const BYTES: usize, E: Encoding> FromStr for FixedBytes<BYTES, E> {
    type Err = E::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut out = [0; BYTES];

        E::decode_to_slice(s, &mut out)?;

        Ok(Self::new(out))
    }
}

impl<E: Encoding, const BYTES: usize> Default for FixedBytes<BYTES, E> {
    fn default() -> Self {
        Self::new([0_u8; BYTES])
    }
}

impl<'a, E: Encoding, const BYTES: usize> IntoIterator for &'a FixedBytes<BYTES, E> {
    type Item = &'a u8;
    type IntoIter = core::slice::Iter<'a, u8>;

    fn into_iter(self) -> core::slice::Iter<'a, u8> {
        self.get().iter()
    }
}

impl<E: Encoding, const BYTES: usize> IntoIterator for FixedBytes<BYTES, E> {
    type Item = u8;
    type IntoIter = core::array::IntoIter<u8, BYTES>;

    fn into_iter(self) -> Self::IntoIter {
        (*self.get()).into_iter()
    }
}

impl<E: Encoding, const BYTES: usize> TryFrom<Vec<u8>> for FixedBytes<BYTES, E> {
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

impl<E: Encoding, const BYTES: usize> TryFrom<&Vec<u8>> for FixedBytes<BYTES, E> {
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

impl<E: Encoding, const BYTES: usize> TryFrom<&[u8]> for FixedBytes<BYTES, E> {
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

impl<E: Encoding, const BYTES: usize> From<FixedBytes<BYTES, E>> for Vec<u8> {
    fn from(value: FixedBytes<BYTES, E>) -> Self {
        value.get().into()
    }
}

impl<E: Encoding, const BYTES: usize> From<FixedBytes<BYTES, E>> for [u8; BYTES] {
    fn from(value: FixedBytes<BYTES, E>) -> Self {
        *value.get()
    }
}

impl<E: Encoding, const BYTES: usize> From<[u8; BYTES]> for FixedBytes<BYTES, E> {
    fn from(value: [u8; BYTES]) -> Self {
        Self::new(value)
    }
}

impl<E: Encoding, const BYTES: usize> Borrow<[u8; BYTES]> for FixedBytes<BYTES, E> {
    fn borrow(&self) -> &[u8; BYTES] {
        self.get()
    }
}

impl<E: Encoding, const BYTES: usize> Borrow<[u8; BYTES]> for &'_ FixedBytes<BYTES, E> {
    fn borrow(&self) -> &[u8; BYTES] {
        self.get()
    }
}

#[cfg(feature = "rlp")]
impl<E: Encoding, const BYTES: usize> rlp::Decodable for FixedBytes<BYTES, E> {
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
impl<E: Encoding, const BYTES: usize> rlp::Encodable for FixedBytes<BYTES, E> {
    fn rlp_append(&self, s: &mut ::rlp::RlpStream) {
        s.encoder().encode_value(self.as_ref());
    }
}

#[cfg(feature = "bincode")]
impl<Enc: Encoding, const BYTES: usize> bincode::Encode for FixedBytes<BYTES, Enc> {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.get().encode(encoder)
    }
}

#[cfg(feature = "bincode")]
impl<Context, Enc: Encoding, const BYTES: usize> bincode::Decode<Context>
    for FixedBytes<BYTES, Enc>
{
    fn decode<D: bincode::de::Decoder>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Ok(Self::new(bincode::Decode::decode(decoder)?))
    }
}

#[cfg(feature = "bincode")]
impl<'de, Context, Enc: Encoding, const BYTES: usize> bincode::BorrowDecode<'de, Context>
    for FixedBytes<BYTES, Enc>
{
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        bincode::Decode::decode(decoder)
    }
}

#[cfg(feature = "schemars")]
impl<E: Encoding, const BYTES: usize> schemars::JsonSchema for FixedBytes<BYTES, E> {
    fn schema_name() -> String {
        format!("FixedBytes<{}, {BYTES}>", E::NAME)
    }

    fn schema_id() -> alloc::borrow::Cow<'static, str> {
        format!("{}::{}", module_path!(), Self::schema_name()).into()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        use schemars::schema::{InstanceType, Metadata, SchemaObject, SingleOrVec};

        SchemaObject {
            metadata: Some(Box::new(Metadata {
                description: Some(format!(
                    "A string representation of fixed bytes of length {BYTES}, encoded via {}",
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
    use hex::FromHexError;

    use super::*;
    use crate::encoding::{
        Base58, Base58Error, Base64, Base64Error, HexPrefixedFromStrError, HexUnprefixed,
    };

    const BASE64_STR: &str = "YWJjZA==";
    const BASE58_STR: &str = "3VNr6P";
    const HEX_PREFIXED_STR: &str = "0x61626364";
    const HEX_UNPREFIXED_STR: &str = "61626364";

    const RAW_VALUE: &[u8; 4] = b"abcd";

    #[test]
    fn hex_prefixed() {
        type H = FixedBytes<4, HexPrefixed>;

        let decoded = H::from_str(HEX_PREFIXED_STR).unwrap();

        assert_eq!(HEX_PREFIXED_STR, decoded.to_string());

        assert_eq!(decoded.get(), b"abcd");
    }

    #[test]
    fn hex_prefixed_too_long() {
        type H = FixedBytes<3, HexPrefixed>;

        assert_eq!(
            H::from_str(HEX_PREFIXED_STR),
            Err(HexPrefixedFromStrError::InvalidHex(
                FromHexError::InvalidStringLength
            ))
        );
    }

    #[test]
    fn hex_prefixed_too_short() {
        type H = FixedBytes<5, HexPrefixed>;

        assert_eq!(
            H::from_str(HEX_PREFIXED_STR),
            Err(HexPrefixedFromStrError::InvalidHex(
                FromHexError::InvalidStringLength
            ))
        );
    }

    #[test]
    fn hex_unprefixed() {
        type H = FixedBytes<4, HexUnprefixed>;

        let decoded = H::from_str(HEX_UNPREFIXED_STR).unwrap();

        assert_eq!(HEX_UNPREFIXED_STR, decoded.to_string());

        assert_eq!(decoded.get(), b"abcd");
    }

    #[test]
    fn hex_unprefixed_too_long() {
        type H = FixedBytes<3, HexUnprefixed>;

        assert_eq!(
            H::from_str(HEX_UNPREFIXED_STR),
            Err(FromHexError::InvalidStringLength)
        );
    }

    #[test]
    fn hex_unprefixed_too_short() {
        type H = FixedBytes<5, HexUnprefixed>;

        assert_eq!(
            H::from_str(HEX_UNPREFIXED_STR),
            Err(FromHexError::InvalidStringLength)
        );
    }

    #[test]
    fn base64() {
        type H = FixedBytes<4, Base64>;

        let decoded = H::from_str(BASE64_STR).unwrap();

        assert_eq!(BASE64_STR, decoded.to_string());

        assert_eq!(decoded.get(), RAW_VALUE);
    }

    #[test]
    fn base64_too_long() {
        type H = FixedBytes<3, Base64>;

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
        type H = FixedBytes<5, Base64>;

        assert_eq!(
            H::from_str(BASE64_STR),
            Err(Base64Error::InvalidLength {
                expected_len: 5,
                found_len: 4
            })
        );
    }

    #[test]
    fn base58() {
        type H = FixedBytes<4, Base58>;

        let decoded = H::from_str(BASE58_STR).unwrap();

        assert_eq!(BASE58_STR, decoded.to_string());

        assert_eq!(decoded.get(), RAW_VALUE);
    }

    #[test]
    fn base58_too_long() {
        type H = FixedBytes<3, Base58>;

        assert_eq!(
            H::from_str(BASE58_STR),
            Err(Base58Error::InvalidEncoding(
                bs58::decode::Error::BufferTooSmall
            ))
        );
    }

    #[test]
    fn base58_too_short() {
        type H = FixedBytes<5, Base58>;

        assert_eq!(
            H::from_str(BASE58_STR),
            Err(Base58Error::InvalidLength {
                expected_len: 5,
                found_len: 4
            })
        );
    }

    #[test]
    fn new_ref() {
        let arr = &[1, 2, 3];

        assert_eq!(<FixedBytes<3, HexPrefixed>>::new_ref(arr).get(), arr);
    }

    #[test]
    fn bincode() {
        let bz: FixedBytes<3> = [1, 2, 3].into();

        let bincode_bz = bincode::encode_to_vec(bz, bincode::config::legacy()).unwrap();

        // not length prefixed (Bytes is)
        assert_eq!(bz.get(), &*bincode_bz);
    }
}
