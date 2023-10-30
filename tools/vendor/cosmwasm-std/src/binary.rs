use std::fmt;
use std::ops::Deref;

use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};

use crate::errors::{StdError, StdResult};

/// Binary is a wrapper around Vec<u8> to add base64 de/serialization
/// with serde. It also adds some helper methods to help encode inline.
///
/// This is only needed as serde-json-{core,wasm} has a horrible encoding for Vec<u8>.
/// See also <https://github.com/CosmWasm/cosmwasm/blob/main/docs/MESSAGE_TYPES.md>.
#[derive(Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord, JsonSchema)]
pub struct Binary(#[schemars(with = "String")] pub Vec<u8>);

impl Binary {
    /// take an (untrusted) string and decode it into bytes.
    /// fails if it is not valid base64
    pub fn from_base64(encoded: &str) -> StdResult<Self> {
        let binary = base64::decode(encoded).map_err(StdError::invalid_base64)?;
        Ok(Binary(binary))
    }

    /// encode to base64 string (guaranteed to be success as we control the data inside).
    /// this returns normalized form (with trailing = if needed)
    pub fn to_base64(&self) -> String {
        base64::encode(&self.0)
    }

    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    /// Copies content into fixed-sized array.
    ///
    /// # Examples
    ///
    /// Copy to array of explicit length
    ///
    /// ```
    /// # use cosmwasm_std::Binary;
    /// let binary = Binary::from(&[0xfb, 0x1f, 0x37]);
    /// let array: [u8; 3] = binary.to_array().unwrap();
    /// assert_eq!(array, [0xfb, 0x1f, 0x37]);
    /// ```
    ///
    /// Copy to integer
    ///
    /// ```
    /// # use cosmwasm_std::Binary;
    /// let binary = Binary::from(&[0x8b, 0x67, 0x64, 0x84, 0xb5, 0xfb, 0x1f, 0x37]);
    /// let num = u64::from_be_bytes(binary.to_array().unwrap());
    /// assert_eq!(num, 10045108015024774967);
    /// ```
    pub fn to_array<const LENGTH: usize>(&self) -> StdResult<[u8; LENGTH]> {
        if self.len() != LENGTH {
            return Err(StdError::invalid_data_size(LENGTH, self.len()));
        }

        let mut out: [u8; LENGTH] = [0; LENGTH];
        out.copy_from_slice(&self.0);
        Ok(out)
    }
}

impl fmt::Display for Binary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_base64())
    }
}

impl fmt::Debug for Binary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use an output inspired by tuples (https://doc.rust-lang.org/std/fmt/struct.Formatter.html#method.debug_tuple)
        // but with a custom implementation to avoid the need for an intemediate hex string.
        write!(f, "Binary(")?;
        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

/// Just like Vec<u8>, Binary is a smart pointer to [u8].
/// This implements `*binary` for us and allows us to
/// do `&*binary`, returning a `&[u8]` from a `&Binary`.
/// With [deref coercions](https://doc.rust-lang.org/1.22.1/book/first-edition/deref-coercions.html#deref-coercions),
/// this allows us to use `&binary` whenever a `&[u8]` is required.
impl Deref for Binary {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl AsRef<[u8]> for Binary {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

// Slice
impl From<&[u8]> for Binary {
    fn from(binary: &[u8]) -> Self {
        Self(binary.to_vec())
    }
}

// Array reference
impl<const LENGTH: usize> From<&[u8; LENGTH]> for Binary {
    fn from(source: &[u8; LENGTH]) -> Self {
        Self(source.to_vec())
    }
}

// Owned array
impl<const LENGTH: usize> From<[u8; LENGTH]> for Binary {
    fn from(source: [u8; LENGTH]) -> Self {
        Self(source.into())
    }
}

impl From<Vec<u8>> for Binary {
    fn from(vec: Vec<u8>) -> Self {
        Self(vec)
    }
}

impl From<Binary> for Vec<u8> {
    fn from(original: Binary) -> Vec<u8> {
        original.0
    }
}

/// Implement `encoding::Binary == std::vec::Vec<u8>`
impl PartialEq<Vec<u8>> for Binary {
    fn eq(&self, rhs: &Vec<u8>) -> bool {
        // Use Vec<u8> == Vec<u8>
        self.0 == *rhs
    }
}

/// Implement `std::vec::Vec<u8> == encoding::Binary`
impl PartialEq<Binary> for Vec<u8> {
    fn eq(&self, rhs: &Binary) -> bool {
        // Use Vec<u8> == Vec<u8>
        *self == rhs.0
    }
}

/// Implement `Binary == &[u8]`
impl PartialEq<&[u8]> for Binary {
    fn eq(&self, rhs: &&[u8]) -> bool {
        // Use &[u8] == &[u8]
        self.as_slice() == *rhs
    }
}

/// Implement `&[u8] == Binary`
impl PartialEq<Binary> for &[u8] {
    fn eq(&self, rhs: &Binary) -> bool {
        // Use &[u8] == &[u8]
        *self == rhs.as_slice()
    }
}

/// Implement `Binary == &[u8; LENGTH]`
impl<const LENGTH: usize> PartialEq<&[u8; LENGTH]> for Binary {
    fn eq(&self, rhs: &&[u8; LENGTH]) -> bool {
        self.as_slice() == rhs.as_slice()
    }
}

/// Implement `&[u8; LENGTH] == Binary`
impl<const LENGTH: usize> PartialEq<Binary> for &[u8; LENGTH] {
    fn eq(&self, rhs: &Binary) -> bool {
        self.as_slice() == rhs.as_slice()
    }
}

/// Implement `Binary == [u8; LENGTH]`
impl<const LENGTH: usize> PartialEq<[u8; LENGTH]> for Binary {
    fn eq(&self, rhs: &[u8; LENGTH]) -> bool {
        self.as_slice() == rhs.as_slice()
    }
}

/// Implement `[u8; LENGTH] == Binary`
impl<const LENGTH: usize> PartialEq<Binary> for [u8; LENGTH] {
    fn eq(&self, rhs: &Binary) -> bool {
        self.as_slice() == rhs.as_slice()
    }
}

/// Serializes as a base64 string
impl Serialize for Binary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_base64())
    }
}

/// Deserializes as a base64 string
impl<'de> Deserialize<'de> for Binary {
    fn deserialize<D>(deserializer: D) -> Result<Binary, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(Base64Visitor)
    }
}

struct Base64Visitor;

impl<'de> de::Visitor<'de> for Base64Visitor {
    type Value = Binary;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("valid base64 encoded string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match Binary::from_base64(v) {
            Ok(binary) => Ok(binary),
            Err(_) => Err(E::custom(format!("invalid base64: {}", v))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::StdError;
    use crate::serde::{from_slice, to_vec};
    use std::collections::hash_map::DefaultHasher;
    use std::collections::HashSet;
    use std::hash::{Hash, Hasher};

    #[test]
    fn encode_decode() {
        let binary: &[u8] = b"hello";
        let encoded = Binary::from(binary).to_base64();
        assert_eq!(8, encoded.len());
        let decoded = Binary::from_base64(&encoded).unwrap();
        assert_eq!(binary, decoded.as_slice());
    }

    #[test]
    fn encode_decode_non_ascii() {
        let binary = vec![12u8, 187, 0, 17, 250, 1];
        let encoded = Binary(binary.clone()).to_base64();
        assert_eq!(8, encoded.len());
        let decoded = Binary::from_base64(&encoded).unwrap();
        assert_eq!(binary.deref(), decoded.deref());
    }

    #[test]
    fn to_array_works() {
        // simple
        let binary = Binary::from(&[1, 2, 3]);
        let array: [u8; 3] = binary.to_array().unwrap();
        assert_eq!(array, [1, 2, 3]);

        // empty
        let binary = Binary::from(&[]);
        let array: [u8; 0] = binary.to_array().unwrap();
        assert_eq!(array, [] as [u8; 0]);

        // invalid size
        let binary = Binary::from(&[1, 2, 3]);
        let error = binary.to_array::<8>().unwrap_err();
        match error {
            StdError::InvalidDataSize {
                expected, actual, ..
            } => {
                assert_eq!(expected, 8);
                assert_eq!(actual, 3);
            }
            err => panic!("Unexpected error: {:?}", err),
        }

        // long array (32 bytes)
        let binary = Binary::from_base64("t119JOQox4WUQEmO/nyqOZfO+wjJm91YG2sfn4ZglvA=").unwrap();
        let array: [u8; 32] = binary.to_array().unwrap();
        assert_eq!(
            array,
            [
                0xb7, 0x5d, 0x7d, 0x24, 0xe4, 0x28, 0xc7, 0x85, 0x94, 0x40, 0x49, 0x8e, 0xfe, 0x7c,
                0xaa, 0x39, 0x97, 0xce, 0xfb, 0x08, 0xc9, 0x9b, 0xdd, 0x58, 0x1b, 0x6b, 0x1f, 0x9f,
                0x86, 0x60, 0x96, 0xf0,
            ]
        );

        // very long array > 32 bytes (requires Rust 1.47+)
        let binary =
            Binary::from_base64("t119JOQox4WUQEmO/nyqOZfO+wjJm91YG2sfn4ZglvBzyMOwMWq+").unwrap();
        let array: [u8; 39] = binary.to_array().unwrap();
        assert_eq!(
            array,
            [
                0xb7, 0x5d, 0x7d, 0x24, 0xe4, 0x28, 0xc7, 0x85, 0x94, 0x40, 0x49, 0x8e, 0xfe, 0x7c,
                0xaa, 0x39, 0x97, 0xce, 0xfb, 0x08, 0xc9, 0x9b, 0xdd, 0x58, 0x1b, 0x6b, 0x1f, 0x9f,
                0x86, 0x60, 0x96, 0xf0, 0x73, 0xc8, 0xc3, 0xb0, 0x31, 0x6a, 0xbe,
            ]
        );
    }

    #[test]
    fn from_valid_string() {
        let valid_base64 = "cmFuZG9taVo=";
        let binary = Binary::from_base64(valid_base64).unwrap();
        assert_eq!(b"randomiZ", binary.as_slice());
    }

    // this accepts input without a trailing = but outputs normal form
    #[test]
    fn from_shortened_string() {
        let short = "cmFuZG9taVo";
        let long = "cmFuZG9taVo=";
        let binary = Binary::from_base64(short).unwrap();
        assert_eq!(b"randomiZ", binary.as_slice());
        assert_eq!(long, binary.to_base64());
    }

    #[test]
    fn from_invalid_string() {
        let invalid_base64 = "cm%uZG9taVo";
        let res = Binary::from_base64(invalid_base64);
        match res.unwrap_err() {
            StdError::InvalidBase64 { msg, .. } => assert_eq!(msg, "Invalid byte 37, offset 2."),
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn from_slice_works() {
        let original: &[u8] = &[0u8, 187, 61, 11, 250, 0];
        let binary: Binary = original.into();
        assert_eq!(binary.as_slice(), [0u8, 187, 61, 11, 250, 0]);
    }

    #[test]
    fn from_fixed_length_array_works() {
        let original = &[];
        let binary: Binary = original.into();
        assert_eq!(binary.len(), 0);

        let original = &[0u8];
        let binary: Binary = original.into();
        assert_eq!(binary.as_slice(), [0u8]);

        let original = &[0u8, 187, 61, 11, 250, 0];
        let binary: Binary = original.into();
        assert_eq!(binary.as_slice(), [0u8, 187, 61, 11, 250, 0]);

        let original = &[
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1,
        ];
        let binary: Binary = original.into();
        assert_eq!(
            binary.as_slice(),
            [
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1,
            ]
        );
    }

    #[test]
    fn from_owned_fixed_length_array_works() {
        let original = [];
        let binary: Binary = original.into();
        assert_eq!(binary.len(), 0);

        let original = [0u8];
        let binary: Binary = original.into();
        assert_eq!(binary.as_slice(), [0u8]);

        let original = [0u8, 187, 61, 11, 250, 0];
        let binary: Binary = original.into();
        assert_eq!(binary.as_slice(), [0u8, 187, 61, 11, 250, 0]);

        let original = [
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1,
        ];
        let binary: Binary = original.into();
        assert_eq!(
            binary.as_slice(),
            [
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1,
            ]
        );
    }

    #[test]
    fn from_literal_works() {
        let a: Binary = b"".into();
        assert_eq!(a.len(), 0);

        let a: Binary = b".".into();
        assert_eq!(a.len(), 1);

        let a: Binary = b"...".into();
        assert_eq!(a.len(), 3);

        let a: Binary = b"...............................".into();
        assert_eq!(a.len(), 31);

        let a: Binary = b"................................".into();
        assert_eq!(a.len(), 32);

        let a: Binary = b".................................".into();
        assert_eq!(a.len(), 33);
    }

    #[test]
    fn from_vec_works() {
        let original = vec![0u8, 187, 61, 11, 250, 0];
        let original_ptr = original.as_ptr();
        let binary: Binary = original.into();
        assert_eq!(binary.as_slice(), [0u8, 187, 61, 11, 250, 0]);
        assert_eq!(binary.0.as_ptr(), original_ptr, "vector must not be copied");
    }

    #[test]
    fn into_vec_works() {
        // Into<Vec<u8>> for Binary
        let original = Binary(vec![0u8, 187, 61, 11, 250, 0]);
        let original_ptr = original.0.as_ptr();
        let vec: Vec<u8> = original.into();
        assert_eq!(vec.as_slice(), [0u8, 187, 61, 11, 250, 0]);
        assert_eq!(vec.as_ptr(), original_ptr, "vector must not be copied");

        // From<Binary> for Vec<u8>
        let original = Binary(vec![7u8, 35, 49, 101, 0, 255]);
        let original_ptr = original.0.as_ptr();
        let vec = Vec::<u8>::from(original);
        assert_eq!(vec.as_slice(), [7u8, 35, 49, 101, 0, 255]);
        assert_eq!(vec.as_ptr(), original_ptr, "vector must not be copied");
    }

    #[test]
    fn serialization_works() {
        let binary = Binary(vec![0u8, 187, 61, 11, 250, 0]);

        let json = to_vec(&binary).unwrap();
        let deserialized: Binary = from_slice(&json).unwrap();

        assert_eq!(binary, deserialized);
    }

    #[test]
    fn deserialize_from_valid_string() {
        let b64_str = "ALs9C/oA";
        // this is the binary behind above string
        let expected = vec![0u8, 187, 61, 11, 250, 0];

        let serialized = to_vec(&b64_str).unwrap();
        let deserialized: Binary = from_slice(&serialized).unwrap();
        assert_eq!(expected, deserialized.as_slice());
    }

    #[test]
    fn deserialize_from_invalid_string() {
        let invalid_str = "**BAD!**";
        let serialized = to_vec(&invalid_str).unwrap();
        let res = from_slice::<Binary>(&serialized);
        assert!(res.is_err());
    }

    #[test]
    fn binary_implements_debug() {
        // Some data
        let binary = Binary(vec![0x07, 0x35, 0xAA, 0xcb, 0x00, 0xff]);
        assert_eq!(format!("{:?}", binary), "Binary(0735aacb00ff)",);

        // Empty
        let binary = Binary(vec![]);
        assert_eq!(format!("{:?}", binary), "Binary()",);
    }

    #[test]
    fn binary_implements_deref() {
        // Dereference to [u8]
        let binary = Binary(vec![7u8, 35, 49, 101, 0, 255]);
        assert_eq!(*binary, [7u8, 35, 49, 101, 0, 255]);

        // This checks deref coercions from &Binary to &[u8] works
        let binary = Binary(vec![7u8, 35, 49, 101, 0, 255]);
        assert_eq!(binary.len(), 6);
        let binary_slice: &[u8] = &binary;
        assert_eq!(binary_slice, &[7u8, 35, 49, 101, 0, 255]);
    }

    #[test]
    fn binary_implements_as_ref() {
        // Can use as_ref (this we already get via the Deref implementation)
        let data = Binary(vec![7u8, 35, 49, 101, 0, 255]);
        assert_eq!(data.as_ref(), &[7u8, 35, 49, 101, 0, 255]);

        let data = Binary(vec![7u8, 35, 49, 101, 0, 255]);
        let data_ref = &data;
        assert_eq!(data_ref.as_ref(), &[7u8, 35, 49, 101, 0, 255]);

        // Implements as ref

        // This is a dummy function to mimic the signature of
        // https://docs.rs/sha2/0.10.6/sha2/trait.Digest.html#tymethod.digest
        fn hash(data: impl AsRef<[u8]>) -> u64 {
            let mut hasher = DefaultHasher::new();
            data.as_ref().hash(&mut hasher);
            hasher.finish()
        }

        let data = Binary(vec![7u8, 35, 49, 101, 0, 255]);
        hash(data);

        let data = Binary(vec![7u8, 35, 49, 101, 0, 255]);
        let data_ref = &data;
        hash(data_ref);
    }

    #[test]
    fn binary_implements_hash() {
        let a1 = Binary::from([0, 187, 61, 11, 250, 0]);
        let mut hasher = DefaultHasher::new();
        a1.hash(&mut hasher);
        let a1_hash = hasher.finish();

        let a2 = Binary::from([0, 187, 61, 11, 250, 0]);
        let mut hasher = DefaultHasher::new();
        a2.hash(&mut hasher);
        let a2_hash = hasher.finish();

        let b = Binary::from([16, 21, 33, 0, 255, 9]);
        let mut hasher = DefaultHasher::new();
        b.hash(&mut hasher);
        let b_hash = hasher.finish();

        assert_eq!(a1_hash, a2_hash);
        assert_ne!(a1_hash, b_hash);
    }

    /// This requires Hash and Eq to be implemented
    #[test]
    fn binary_can_be_used_in_hash_set() {
        let a1 = Binary::from([0, 187, 61, 11, 250, 0]);
        let a2 = Binary::from([0, 187, 61, 11, 250, 0]);
        let b = Binary::from([16, 21, 33, 0, 255, 9]);

        let mut set = HashSet::new();
        set.insert(a1.clone());
        set.insert(a2.clone());
        set.insert(b.clone());
        assert_eq!(set.len(), 2);

        let set1 = HashSet::<Binary>::from_iter(vec![b.clone(), a1.clone()]);
        let set2 = HashSet::from_iter(vec![a1, a2, b]);
        assert_eq!(set1, set2);
    }

    #[test]
    fn binary_implements_partial_eq_with_vector() {
        let a = Binary(vec![5u8; 3]);
        let b = vec![5u8; 3];
        let c = vec![9u8; 3];
        assert_eq!(a, b);
        assert_eq!(b, a);
        assert_ne!(a, c);
        assert_ne!(c, a);
    }

    #[test]
    fn binary_implements_partial_eq_with_slice_and_array() {
        let a = Binary(vec![0xAA, 0xBB]);

        // Slice: &[u8]
        assert_eq!(a, b"\xAA\xBB" as &[u8]);
        assert_eq!(b"\xAA\xBB" as &[u8], a);
        assert_ne!(a, b"\x11\x22" as &[u8]);
        assert_ne!(b"\x11\x22" as &[u8], a);

        // Array reference: &[u8; 2]
        assert_eq!(a, b"\xAA\xBB");
        assert_eq!(b"\xAA\xBB", a);
        assert_ne!(a, b"\x11\x22");
        assert_ne!(b"\x11\x22", a);

        // Array: [u8; 2]
        assert_eq!(a, [0xAA, 0xBB]);
        assert_eq!([0xAA, 0xBB], a);
        assert_ne!(a, [0x11, 0x22]);
        assert_ne!([0x11, 0x22], a);
    }
}
