use std::fmt;
use std::ops::Deref;

use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};

use crate::{Binary, StdError, StdResult};

/// This is a wrapper around Vec<u8> to add hex de/serialization
/// with serde. It also adds some helper methods to help encode inline.
///
/// This is similar to `cosmwasm_std::Binary` but uses hex.
/// See also <https://github.com/CosmWasm/cosmwasm/blob/main/docs/MESSAGE_TYPES.md>.
#[derive(Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord, JsonSchema)]
pub struct HexBinary(#[schemars(with = "String")] Vec<u8>);

impl HexBinary {
    pub fn from_hex(input: &str) -> StdResult<Self> {
        let vec = hex::decode(input).map_err(StdError::invalid_hex)?;
        Ok(Self(vec))
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
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
    /// # use cosmwasm_std::HexBinary;
    /// let data = HexBinary::from(&[0xfb, 0x1f, 0x37]);
    /// let array: [u8; 3] = data.to_array().unwrap();
    /// assert_eq!(array, [0xfb, 0x1f, 0x37]);
    /// ```
    ///
    /// Copy to integer
    ///
    /// ```
    /// # use cosmwasm_std::HexBinary;
    /// let data = HexBinary::from(&[0x8b, 0x67, 0x64, 0x84, 0xb5, 0xfb, 0x1f, 0x37]);
    /// let num = u64::from_be_bytes(data.to_array().unwrap());
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

impl fmt::Display for HexBinary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl fmt::Debug for HexBinary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use an output inspired by tuples (https://doc.rust-lang.org/std/fmt/struct.Formatter.html#method.debug_tuple)
        // but with a custom implementation to avoid the need for an intemediate hex string.
        write!(f, "HexBinary(")?;
        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

/// Just like Vec<u8>, HexBinary is a smart pointer to [u8].
/// This implements `*data` for us and allows us to
/// do `&*data`, returning a `&[u8]` from a `&HexBinary`.
/// With [deref coercions](https://doc.rust-lang.org/1.22.1/book/first-edition/deref-coercions.html#deref-coercions),
/// this allows us to use `&data` whenever a `&[u8]` is required.
impl Deref for HexBinary {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl AsRef<[u8]> for HexBinary {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

// Slice
impl From<&[u8]> for HexBinary {
    fn from(binary: &[u8]) -> Self {
        Self(binary.to_vec())
    }
}

// Array reference
impl<const LENGTH: usize> From<&[u8; LENGTH]> for HexBinary {
    fn from(source: &[u8; LENGTH]) -> Self {
        Self(source.to_vec())
    }
}

// Owned array
impl<const LENGTH: usize> From<[u8; LENGTH]> for HexBinary {
    fn from(source: [u8; LENGTH]) -> Self {
        Self(source.into())
    }
}

impl From<Vec<u8>> for HexBinary {
    fn from(vec: Vec<u8>) -> Self {
        Self(vec)
    }
}

impl From<HexBinary> for Vec<u8> {
    fn from(original: HexBinary) -> Vec<u8> {
        original.0
    }
}

impl From<Binary> for HexBinary {
    fn from(original: Binary) -> Self {
        Self(original.into())
    }
}

impl From<HexBinary> for Binary {
    fn from(original: HexBinary) -> Binary {
        Binary::from(original.0)
    }
}

/// Implement `HexBinary == std::vec::Vec<u8>`
impl PartialEq<Vec<u8>> for HexBinary {
    fn eq(&self, rhs: &Vec<u8>) -> bool {
        // Use Vec<u8> == Vec<u8>
        self.0 == *rhs
    }
}

/// Implement `std::vec::Vec<u8> == HexBinary`
impl PartialEq<HexBinary> for Vec<u8> {
    fn eq(&self, rhs: &HexBinary) -> bool {
        // Use Vec<u8> == Vec<u8>
        *self == rhs.0
    }
}

/// Implement `HexBinary == &[u8]`
impl PartialEq<&[u8]> for HexBinary {
    fn eq(&self, rhs: &&[u8]) -> bool {
        // Use &[u8] == &[u8]
        self.as_slice() == *rhs
    }
}

/// Implement `&[u8] == HexBinary`
impl PartialEq<HexBinary> for &[u8] {
    fn eq(&self, rhs: &HexBinary) -> bool {
        // Use &[u8] == &[u8]
        *self == rhs.as_slice()
    }
}

/// Implement `HexBinary == [u8; LENGTH]`
impl<const LENGTH: usize> PartialEq<[u8; LENGTH]> for HexBinary {
    fn eq(&self, rhs: &[u8; LENGTH]) -> bool {
        self.as_slice() == rhs.as_slice()
    }
}

/// Implement `[u8; LENGTH] == HexBinary`
impl<const LENGTH: usize> PartialEq<HexBinary> for [u8; LENGTH] {
    fn eq(&self, rhs: &HexBinary) -> bool {
        self.as_slice() == rhs.as_slice()
    }
}

/// Implement `HexBinary == &[u8; LENGTH]`
impl<const LENGTH: usize> PartialEq<&[u8; LENGTH]> for HexBinary {
    fn eq(&self, rhs: &&[u8; LENGTH]) -> bool {
        self.as_slice() == rhs.as_slice()
    }
}

/// Implement `&[u8; LENGTH] == HexBinary`
impl<const LENGTH: usize> PartialEq<HexBinary> for &[u8; LENGTH] {
    fn eq(&self, rhs: &HexBinary) -> bool {
        self.as_slice() == rhs.as_slice()
    }
}

/// Serializes as a hex string
impl Serialize for HexBinary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

/// Deserializes as a hex string
impl<'de> Deserialize<'de> for HexBinary {
    fn deserialize<D>(deserializer: D) -> Result<HexBinary, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(HexVisitor)
    }
}

struct HexVisitor;

impl<'de> de::Visitor<'de> for HexVisitor {
    type Value = HexBinary;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("valid hex encoded string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match HexBinary::from_hex(v) {
            Ok(data) => Ok(data),
            Err(_) => Err(E::custom(format!("invalid hex: {}", v))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{from_slice, to_vec, StdError};
    use std::collections::hash_map::DefaultHasher;
    use std::collections::HashSet;
    use std::hash::{Hash, Hasher};

    #[test]
    fn from_hex_works() {
        let data = HexBinary::from_hex("").unwrap();
        assert_eq!(data, b"");
        let data = HexBinary::from_hex("61").unwrap();
        assert_eq!(data, b"a");
        let data = HexBinary::from_hex("00").unwrap();
        assert_eq!(data, b"\0");

        let data = HexBinary::from_hex("68656c6c6f").unwrap();
        assert_eq!(data, b"hello");
        let data = HexBinary::from_hex("68656C6C6F").unwrap();
        assert_eq!(data, b"hello");
        let data = HexBinary::from_hex("72616e646f6d695a").unwrap();
        assert_eq!(data.as_slice(), b"randomiZ");

        // odd
        match HexBinary::from_hex("123").unwrap_err() {
            StdError::InvalidHex { msg, .. } => {
                assert_eq!(msg, "Odd number of digits")
            }
            _ => panic!("Unexpected error type"),
        }
        // non-hex
        match HexBinary::from_hex("efgh").unwrap_err() {
            StdError::InvalidHex { msg, .. } => {
                assert_eq!(msg, "Invalid character 'g' at position 2")
            }
            _ => panic!("Unexpected error type"),
        }
        // 0x prefixed
        match HexBinary::from_hex("0xaa").unwrap_err() {
            StdError::InvalidHex { msg, .. } => {
                assert_eq!(msg, "Invalid character 'x' at position 1")
            }
            _ => panic!("Unexpected error type"),
        }
        // spaces
        assert!(matches!(
            HexBinary::from_hex("aa ").unwrap_err(),
            StdError::InvalidHex { .. }
        ));
        assert!(matches!(
            HexBinary::from_hex(" aa").unwrap_err(),
            StdError::InvalidHex { .. }
        ));
        assert!(matches!(
            HexBinary::from_hex("a a").unwrap_err(),
            StdError::InvalidHex { .. }
        ));
        assert!(matches!(
            HexBinary::from_hex(" aa ").unwrap_err(),
            StdError::InvalidHex { .. }
        ));
    }

    #[test]
    fn to_hex_works() {
        let binary: &[u8] = b"";
        let encoded = HexBinary::from(binary).to_hex();
        assert_eq!(encoded, "");

        let binary: &[u8] = b"hello";
        let encoded = HexBinary::from(binary).to_hex();
        assert_eq!(encoded, "68656c6c6f");

        let binary = vec![12u8, 187, 0, 17, 250, 1];
        let encoded = HexBinary(binary).to_hex();
        assert_eq!(encoded, "0cbb0011fa01");
    }

    #[test]
    fn to_array_works() {
        // simple
        let binary = HexBinary::from(&[1, 2, 3]);
        let array: [u8; 3] = binary.to_array().unwrap();
        assert_eq!(array, [1, 2, 3]);

        // empty
        let binary = HexBinary::from(&[]);
        let array: [u8; 0] = binary.to_array().unwrap();
        assert_eq!(array, [] as [u8; 0]);

        // invalid size
        let binary = HexBinary::from(&[1, 2, 3]);
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
        let binary =
            HexBinary::from_hex("b75d7d24e428c7859440498efe7caa3997cefb08c99bdd581b6b1f9f866096f0")
                .unwrap();
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
        let binary = HexBinary::from_hex(
            "b75d7d24e428c7859440498efe7caa3997cefb08c99bdd581b6b1f9f866096f073c8c3b0316abe",
        )
        .unwrap();
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
    fn from_slice_works() {
        let original: &[u8] = &[0u8, 187, 61, 11, 250, 0];
        let binary: HexBinary = original.into();
        assert_eq!(binary.as_slice(), [0u8, 187, 61, 11, 250, 0]);
    }

    #[test]
    fn from_fixed_length_array_works() {
        let original = &[];
        let binary: HexBinary = original.into();
        assert_eq!(binary.len(), 0);

        let original = &[0u8];
        let binary: HexBinary = original.into();
        assert_eq!(binary.as_slice(), [0u8]);

        let original = &[0u8, 187, 61, 11, 250, 0];
        let binary: HexBinary = original.into();
        assert_eq!(binary.as_slice(), [0u8, 187, 61, 11, 250, 0]);

        let original = &[
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1,
        ];
        let binary: HexBinary = original.into();
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
        let binary: HexBinary = original.into();
        assert_eq!(binary.len(), 0);

        let original = [0u8];
        let binary: HexBinary = original.into();
        assert_eq!(binary.as_slice(), [0u8]);

        let original = [0u8, 187, 61, 11, 250, 0];
        let binary: HexBinary = original.into();
        assert_eq!(binary.as_slice(), [0u8, 187, 61, 11, 250, 0]);

        let original = [
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1,
        ];
        let binary: HexBinary = original.into();
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
        let a: HexBinary = b"".into();
        assert_eq!(a.len(), 0);

        let a: HexBinary = b".".into();
        assert_eq!(a.len(), 1);

        let a: HexBinary = b"...".into();
        assert_eq!(a.len(), 3);

        let a: HexBinary = b"...............................".into();
        assert_eq!(a.len(), 31);

        let a: HexBinary = b"................................".into();
        assert_eq!(a.len(), 32);

        let a: HexBinary = (b".................................").into();
        assert_eq!(a.len(), 33);
    }

    #[test]
    fn from_vec_works() {
        let original = vec![0u8, 187, 61, 11, 250, 0];
        let original_ptr = original.as_ptr();
        let binary: HexBinary = original.into();
        assert_eq!(binary.as_slice(), [0u8, 187, 61, 11, 250, 0]);
        assert_eq!(binary.0.as_ptr(), original_ptr, "vector must not be copied");
    }

    #[test]
    fn into_vec_works() {
        // Into<Vec<u8>> for HexBinary
        let original = HexBinary(vec![0u8, 187, 61, 11, 250, 0]);
        let original_ptr = original.0.as_ptr();
        let vec: Vec<u8> = original.into();
        assert_eq!(vec.as_slice(), [0u8, 187, 61, 11, 250, 0]);
        assert_eq!(vec.as_ptr(), original_ptr, "vector must not be copied");

        // From<HexBinary> for Vec<u8>
        let original = HexBinary(vec![7u8, 35, 49, 101, 0, 255]);
        let original_ptr = original.0.as_ptr();
        let vec = Vec::<u8>::from(original);
        assert_eq!(vec.as_slice(), [7u8, 35, 49, 101, 0, 255]);
        assert_eq!(vec.as_ptr(), original_ptr, "vector must not be copied");
    }

    #[test]
    fn from_binary_works() {
        let original = Binary::from([0u8, 187, 61, 11, 250, 0]);
        let original_ptr = original.as_ptr();
        let binary: HexBinary = original.into();
        assert_eq!(binary.as_slice(), [0u8, 187, 61, 11, 250, 0]);
        assert_eq!(binary.0.as_ptr(), original_ptr, "vector must not be copied");
    }

    #[test]
    fn into_binary_works() {
        // Into<Binary> for HexBinary
        let original = HexBinary(vec![0u8, 187, 61, 11, 250, 0]);
        let original_ptr = original.0.as_ptr();
        let bin: Binary = original.into();
        assert_eq!(bin.as_slice(), [0u8, 187, 61, 11, 250, 0]);
        assert_eq!(bin.as_ptr(), original_ptr, "vector must not be copied");

        // From<HexBinary> for Binary
        let original = HexBinary(vec![7u8, 35, 49, 101, 0, 255]);
        let original_ptr = original.0.as_ptr();
        let bin = Binary::from(original);
        assert_eq!(bin.as_slice(), [7u8, 35, 49, 101, 0, 255]);
        assert_eq!(bin.as_ptr(), original_ptr, "vector must not be copied");
    }

    #[test]
    fn serialization_works() {
        let binary = HexBinary(vec![0u8, 187, 61, 11, 250, 0]);

        let json = to_vec(&binary).unwrap();
        let deserialized: HexBinary = from_slice(&json).unwrap();

        assert_eq!(binary, deserialized);
    }

    #[test]
    fn deserialize_from_valid_string() {
        let hex = "00bb3d0bfa00";
        // this is the binary behind above string
        let expected = vec![0u8, 187, 61, 11, 250, 0];

        let serialized = to_vec(&hex).unwrap();
        let deserialized: HexBinary = from_slice(&serialized).unwrap();
        assert_eq!(expected, deserialized.as_slice());
    }

    #[test]
    fn deserialize_from_invalid_string() {
        let invalid_str = "**BAD!**";
        let serialized = to_vec(&invalid_str).unwrap();
        let res = from_slice::<HexBinary>(&serialized);
        assert!(res.is_err());
    }

    #[test]
    fn hex_binary_implements_debug() {
        // Some data
        let data = HexBinary(vec![0x07, 0x35, 0xAA, 0xcb, 0x00, 0xff]);
        assert_eq!(format!("{:?}", data), "HexBinary(0735aacb00ff)",);

        // Empty
        let data = HexBinary(vec![]);
        assert_eq!(format!("{:?}", data), "HexBinary()",);
    }

    #[test]
    fn hex_binary_implements_deref() {
        // Dereference to [u8]
        let data = HexBinary(vec![7u8, 35, 49, 101, 0, 255]);
        assert_eq!(*data, [7u8, 35, 49, 101, 0, 255]);

        // This checks deref coercions from &Binary to &[u8] works
        let data = HexBinary(vec![7u8, 35, 49, 101, 0, 255]);
        assert_eq!(data.len(), 6);
        let data_slice: &[u8] = &data;
        assert_eq!(data_slice, &[7u8, 35, 49, 101, 0, 255]);
    }

    #[test]
    fn hex_binary_implements_as_ref() {
        // Can use as_ref (this we already get via the Deref implementation)
        let data = HexBinary(vec![7u8, 35, 49, 101, 0, 255]);
        assert_eq!(data.as_ref(), &[7u8, 35, 49, 101, 0, 255]);

        let data = HexBinary(vec![7u8, 35, 49, 101, 0, 255]);
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

        let data = HexBinary(vec![7u8, 35, 49, 101, 0, 255]);
        hash(data);

        let data = HexBinary(vec![7u8, 35, 49, 101, 0, 255]);
        let data_ref = &data;
        hash(data_ref);
    }

    #[test]
    fn hex_binary_implements_hash() {
        let a1 = HexBinary::from([0, 187, 61, 11, 250, 0]);
        let mut hasher = DefaultHasher::new();
        a1.hash(&mut hasher);
        let a1_hash = hasher.finish();

        let a2 = HexBinary::from([0, 187, 61, 11, 250, 0]);
        let mut hasher = DefaultHasher::new();
        a2.hash(&mut hasher);
        let a2_hash = hasher.finish();

        let b = HexBinary::from([16, 21, 33, 0, 255, 9]);
        let mut hasher = DefaultHasher::new();
        b.hash(&mut hasher);
        let b_hash = hasher.finish();

        assert_eq!(a1_hash, a2_hash);
        assert_ne!(a1_hash, b_hash);
    }

    /// This requires Hash and Eq to be implemented
    #[test]
    fn hex_binary_can_be_used_in_hash_set() {
        let a1 = HexBinary::from([0, 187, 61, 11, 250, 0]);
        let a2 = HexBinary::from([0, 187, 61, 11, 250, 0]);
        let b = HexBinary::from([16, 21, 33, 0, 255, 9]);

        let mut set = HashSet::new();
        set.insert(a1.clone());
        set.insert(a2.clone());
        set.insert(b.clone());
        assert_eq!(set.len(), 2);

        let set1 = HashSet::<HexBinary>::from_iter(vec![b.clone(), a1.clone()]);
        let set2 = HashSet::from_iter(vec![a1, a2, b]);
        assert_eq!(set1, set2);
    }

    #[test]
    fn hex_binary_implements_partial_eq_with_vector() {
        let a = HexBinary(vec![5u8; 3]);
        let b = vec![5u8; 3];
        let c = vec![9u8; 3];
        assert_eq!(a, b);
        assert_eq!(b, a);
        assert_ne!(a, c);
        assert_ne!(c, a);
    }

    #[test]
    fn hex_binary_implements_partial_eq_with_slice_and_array() {
        let a = HexBinary(vec![0xAA, 0xBB]);

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
