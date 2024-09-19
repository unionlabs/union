use crate::macros::hex_string_array_wrapper;

hex_string_array_wrapper! {
    pub struct H64(pub [u8; 8]);
    pub struct H160(pub [u8; 20]);
    pub struct H256(pub [u8; 32]);
    pub struct H384(pub [u8; 48]);
    pub struct H512(pub [u8; 64]);
    pub struct H2048(pub [u8; 256]);
}

// pub type H256 /* <P = unionlabs_hash::Prefixed> */ = unionlabs_hash::Hash<32 /* , P */>;

impl H256 {
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        // use this if we ever swap out the inner value for primitive_types::H256
        // self.0.into_iter().flat_map(|n| n.to_le_bytes()).collect()
        self.0.to_vec()
    }
}

impl From<H256> for primitive_types::H256 {
    fn from(value: H256) -> Self {
        Self(value.0)
    }
}

impl From<primitive_types::H256> for H256 {
    fn from(value: primitive_types::H256) -> Self {
        Self(value.0)
    }
}

impl From<H160> for primitive_types::H160 {
    fn from(value: H160) -> Self {
        Self(value.0)
    }
}

impl From<primitive_types::H160> for H160 {
    fn from(value: primitive_types::H160) -> Self {
        Self(value.0)
    }
}

pub struct BytesBitIterator<'a> {
    bz: &'a [u8],
    pos: core::ops::Range<usize>,
}

impl<'a> BytesBitIterator<'a> {
    pub fn new(bz: &'a impl AsRef<[u8]>) -> Self {
        BytesBitIterator {
            bz: bz.as_ref(),
            pos: (0..bz.as_ref().len() * 8),
        }
    }

    /// Returns the `index`-th bit in the bytes.
    fn get_bit(&self, index: usize) -> bool {
        // debug_assert_eq!(self.hash_bytes.len(), Hash::LENGTH); // invariant
        // debug_assert_lt!(index, Hash::LENGTH_IN_BITS); // assumed precondition
        let pos = index / 8;
        let bit = 7 - index % 8;
        (self.bz[pos] >> bit) & 1 != 0
    }
}

impl<'a> core::iter::Iterator for BytesBitIterator<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos.next().map(|x| self.get_bit(x))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.pos.size_hint()
    }
}

impl<'a> core::iter::DoubleEndedIterator for BytesBitIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.pos.next_back().map(|x| self.get_bit(x))
    }
}

pub mod hash_v2 {
    use core::{
        cmp::Ordering,
        fmt::{self, Display},
        marker::PhantomData,
        str::FromStr,
    };

    use serde::{ser::SerializeTupleStruct, Deserialize, Deserializer, Serialize, Serializer};

    use crate::errors::{ExpectedLength, InvalidLength};

    trait Sealed {}

    #[expect(private_bounds)]
    pub trait Encoding: Sealed {
        type Error: core::error::Error + 'static;

        fn fmt(bytes: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result;

        fn decode<T: AsRef<[u8]>>(data: T) -> Result<Vec<u8>, Self::Error>;

        fn decode_to_slice<T: AsRef<[u8]>>(data: T, out: &mut [u8]) -> Result<(), Self::Error>;
    }

    pub struct HexPrefixed;
    impl Sealed for HexPrefixed {}
    impl Encoding for HexPrefixed {
        type Error = HexPrefixedFromStrError;

        fn fmt(bytes: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_fmt(format_args!("0x{}", hex::encode(bytes)))
        }

        fn decode<T: AsRef<[u8]>>(data: T) -> Result<Vec<u8>, Self::Error> {
            let data = data
                .as_ref()
                .strip_prefix(b"0x")
                .ok_or(HexPrefixedFromStrError::MissingPrefix)?;

            hex::decode(data).map_err(HexPrefixedFromStrError::InvalidHex)
        }

        fn decode_to_slice<T: AsRef<[u8]>>(data: T, out: &mut [u8]) -> Result<(), Self::Error> {
            let data = data
                .as_ref()
                .strip_prefix(b"0x")
                .ok_or(HexPrefixedFromStrError::MissingPrefix)?;

            hex::decode_to_slice(data, out).map_err(HexPrefixedFromStrError::InvalidHex)
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum HexPrefixedFromStrError {
        #[error("invalid hex")]
        InvalidHex(#[from] hex::FromHexError),
        #[error("missing 0x prefix")]
        MissingPrefix,
    }

    pub struct HexUnprefixed;
    impl Sealed for HexUnprefixed {}
    impl Encoding for HexUnprefixed {
        type Error = hex::FromHexError;

        fn fmt(bytes: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&hex::encode(bytes))
        }

        fn decode<T: AsRef<[u8]>>(data: T) -> Result<Vec<u8>, Self::Error> {
            hex::decode(data)
        }

        fn decode_to_slice<T: AsRef<[u8]>>(data: T, out: &mut [u8]) -> Result<(), Self::Error> {
            hex::decode_to_slice(data, out)
        }
    }

    pub struct Base64;
    impl Sealed for Base64 {}
    impl Encoding for Base64 {
        type Error = Base64Error;

        fn fmt(bytes: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
            use base64::prelude::*;

            f.write_str(&BASE64_STANDARD.encode(bytes))
        }

        fn decode<T: AsRef<[u8]>>(data: T) -> Result<Vec<u8>, Self::Error> {
            use base64::prelude::*;

            BASE64_STANDARD
                .decode(data)
                .map_err(Base64Error::InvalidEncoding)
        }

        fn decode_to_slice<T: AsRef<[u8]>>(data: T, out: &mut [u8]) -> Result<(), Base64Error> {
            use base64::prelude::*;

            let vec = BASE64_STANDARD.decode(data)?;

            if vec.len() == out.len() {
                out.copy_from_slice(&vec);

                Ok(())
            } else {
                Err(Base64Error::InvalidLength(InvalidLength {
                    expected: ExpectedLength::Exact(out.len()),
                    found: vec.len(),
                }))
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
    pub enum Base64Error {
        #[error("invalid encoding")]
        InvalidEncoding(#[from] base64::DecodeError),
        #[error("invalid encoding")]
        InvalidLength(#[from] InvalidLength),
    }

    pub struct Hash<const BYTES: usize, E: Encoding = HexPrefixed> {
        // we abuse deprecated a bit here to make sure this field doesn't get read anywhere it shouldn't, enforcing usage of the constructor instead - this makes sure that the const {} block gets monomorphized, causing a post-mono error if BYTES is 0.
        #[deprecated = "this field should never be used directly, use Hash::new() to construct this type and .get{_mut}() to access the data"]
        arr: [u8; BYTES],
        __marker: PhantomData<fn() -> E>,
    }

    impl<const BYTES: usize, E: Encoding> AsRef<[u8]> for Hash<BYTES, E> {
        fn as_ref(&self) -> &[u8] {
            self.get()
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

    impl<const BYTES: usize> Hash<BYTES, HexPrefixed> {
        // this impl can't be on the generic one otherwise type inference shits the bed
        pub const BYTES_LEN: usize = BYTES;
    }

    impl<const BYTES: usize, E: Encoding> Hash<BYTES, E> {
        #[must_use = "constructing a Hash has no effect"]
        pub fn new(arr: [u8; BYTES]) -> Self {
            const { assert!(BYTES > 0, "BYTES must be greater than 0") };

            #[expect(deprecated)] // this is the (sole) constructor
            Self {
                arr,
                __marker: PhantomData,
            }
        }

        #[must_use = "reading the inner value has no effect"]
        pub fn get(&self) -> &[u8; BYTES] {
            #[expect(deprecated)] // this is the (sole) immutable accessor
            &self.arr
        }

        #[must_use = "reading the inner value has no effect"]
        pub fn get_mut(&mut self) -> &mut [u8; BYTES] {
            #[expect(deprecated)] // this is the (sole) mutable accessor
            &mut self.arr
        }

        #[must_use]
        pub fn into_bytes(self) -> Vec<u8> {
            self.get().to_vec()
        }

        pub fn iter(&self) -> core::slice::Iter<'_, u8> {
            <&Self as IntoIterator>::into_iter(self)
        }

        #[must_use = "converting a hash to a hash with a different encoding has no effect"]
        pub fn into_encoding<E2: Encoding>(&self) -> Hash<BYTES, E2> {
            Hash::new(*self.get())
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

    impl<const BYTES: usize, E: Encoding> Serialize for Hash<BYTES, E> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            if serializer.is_human_readable() {
                serializer.collect_str(self)
            } else {
                let mut s = serializer.serialize_tuple_struct("Hash", BYTES)?;
                for b in self.get() {
                    s.serialize_field(&b)?;
                }
                s.end()
            }
        }
    }

    impl<'de, const BYTES: usize, E: Encoding> Deserialize<'de> for Hash<BYTES, E> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            if deserializer.is_human_readable() {
                String::deserialize(deserializer)
                    .and_then(|s| s.parse().map_err(::serde::de::Error::custom))
            } else {
                struct ArrayVisitor<const BYTES: usize>;

                impl<'de, const N: usize> serde::de::Visitor<'de> for ArrayVisitor<N> {
                    type Value = [u8; N];

                    fn expecting(
                        &self,
                        formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
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
    //     InvalidLength(#[from] InvalidLength),
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
        type Error = InvalidLength;

        fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
            value
                .try_into()
                .map(Self::new)
                .map_err(|invalid| InvalidLength {
                    expected: ExpectedLength::Exact(BYTES),
                    found: invalid.len(),
                })
        }
    }

    impl<E: Encoding, const BYTES: usize> TryFrom<&Vec<u8>> for Hash<BYTES, E> {
        type Error = InvalidLength;

        fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
            value
                .as_slice()
                .try_into()
                .map(Self::new)
                .map_err(|_| InvalidLength {
                    expected: ExpectedLength::Exact(BYTES),
                    found: value.len(),
                })
        }
    }

    impl<E: Encoding, const BYTES: usize> TryFrom<&[u8]> for Hash<BYTES, E> {
        type Error = InvalidLength;

        fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
            value.try_into().map(Self::new).map_err(|_| InvalidLength {
                expected: ExpectedLength::Exact(BYTES),
                found: value.len(),
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

    // #[cfg(feature = "generic-array")]
    // impl<P: IsPrefixed, const BYTES: usize> From<GenericArray<u8, typenum::U<BYTES>>> for Hash<BYTES, E>
    // where
    //     typenum::Const<BYTES>: typenum::ToUInt<Output: ArrayLength>,
    // {
    //     fn from(arr: GenericArray<u8, typenum::U<BYTES>>) -> Self {
    //         Self::new(
    //             arr.to_vec()
    //                 .try_into()
    //                 .expect("GenericArray has the correct length; qed;"),
    //         )
    //     }
    // }

    // #[cfg(feature = "generic-array")]
    // impl<P: IsPrefixed, const BYTES: usize> From<Hash<BYTES, E>> for GenericArray<u8, typenum::U<BYTES>>
    // where
    //     typenum::Const<BYTES>: typenum::ToUInt<Output: ArrayLength>,
    // {
    //     fn from(arr: Hash<BYTES, E>) -> Self {
    //         GenericArray::<u8, typenum::U<BYTES>>::from_slice(arr.get()).to_owned()
    //     }
    // }

    // #[cfg(feature = "rlp")]
    // impl<P: IsPrefixed, const BYTES: usize> rlp::Decodable for Hash<BYTES, E> {
    //     fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
    //         rlp.decoder()
    //             .decode_value(|bytes| match bytes.len().cmp(&BYTES) {
    //                 core::cmp::Ordering::Less => Err(::rlp::DecoderError::RlpIsTooShort),
    //                 core::cmp::Ordering::Greater => Err(::rlp::DecoderError::RlpIsTooBig),
    //                 core::cmp::Ordering::Equal => {
    //                     Ok(Self::new(bytes.try_into().expect("size is checked; qed;")))
    //                 }
    //             })
    //     }
    // }

    // #[cfg(feature = "rlp")]
    // impl<P: IsPrefixed, const BYTES: usize> rlp::Encodable for Hash<BYTES, E> {
    //     fn rlp_append(&self, s: &mut ::rlp::RlpStream) {
    //         s.encoder().encode_value(self.as_ref());
    //     }
    // }

    #[cfg(test)]
    mod tests {
        use hex::FromHexError;

        use super::*;

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
                Err(Base64Error::InvalidLength(InvalidLength {
                    expected: ExpectedLength::Exact(3),
                    found: 4
                }))
            );
        }

        #[test]
        fn base64_too_short() {
            type H = Hash<5, Base64>;

            assert_eq!(
                H::from_str(BASE64_STR),
                Err(Base64Error::InvalidLength(InvalidLength {
                    expected: ExpectedLength::Exact(5),
                    found: 4
                }))
            );
        }
    }
}
