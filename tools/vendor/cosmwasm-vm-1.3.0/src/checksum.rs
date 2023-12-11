use std::fmt;

use sha2::{Digest, Sha256};

use crate::errors::VmError;

/// A SHA-256 checksum of a Wasm blob, used to identify a Wasm code.
/// This must remain stable since this checksum is stored in the blockchain state.
///
/// This is often referred to as "code ID" in go-cosmwasm, even if code ID
/// usually refers to an auto-incrementing number.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Checksum([u8; 32]);

impl Checksum {
    pub fn generate(wasm: &[u8]) -> Self {
        Checksum(Sha256::digest(wasm).into())
    }

    /// Creates a lowercase hex encoded copy of this checksum.
    ///
    /// This takes an owned `self` instead of a reference because `Checksum` is cheap to `Copy`.
    pub fn to_hex(self) -> String {
        self.to_string()
    }
}

impl fmt::Display for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl From<[u8; 32]> for Checksum {
    fn from(data: [u8; 32]) -> Self {
        Checksum(data)
    }
}

impl TryFrom<&[u8]> for Checksum {
    type Error = VmError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 32 {
            return Err(VmError::cache_err("Checksum not of length 32"));
        }
        let mut data = [0u8; 32];
        data.copy_from_slice(value);
        Ok(Checksum(data))
    }
}

impl From<Checksum> for Vec<u8> {
    fn from(original: Checksum) -> Vec<u8> {
        original.0.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_works() {
        let wasm = vec![0x68, 0x69, 0x6a];
        let checksum = Checksum::generate(&wasm);

        // echo -n "hij" | sha256sum
        let expected = [
            0x72, 0x2c, 0x8c, 0x99, 0x3f, 0xd7, 0x5a, 0x76, 0x27, 0xd6, 0x9e, 0xd9, 0x41, 0x34,
            0x4f, 0xe2, 0xa1, 0x42, 0x3a, 0x3e, 0x75, 0xef, 0xd3, 0xe6, 0x77, 0x8a, 0x14, 0x28,
            0x84, 0x22, 0x71, 0x04,
        ];
        assert_eq!(checksum.0, expected);
    }

    #[test]
    fn implemented_display() {
        let wasm = vec![0x68, 0x69, 0x6a];
        let checksum = Checksum::generate(&wasm);
        // echo -n "hij" | sha256sum
        let embedded = format!("Check: {}", checksum);
        assert_eq!(
            embedded,
            "Check: 722c8c993fd75a7627d69ed941344fe2a1423a3e75efd3e6778a142884227104"
        );
        assert_eq!(
            checksum.to_string(),
            "722c8c993fd75a7627d69ed941344fe2a1423a3e75efd3e6778a142884227104"
        );
    }

    #[test]
    fn to_hex_works() {
        let wasm = vec![0x68, 0x69, 0x6a];
        let checksum = Checksum::generate(&wasm);
        // echo -n "hij" | sha256sum
        assert_eq!(
            checksum.to_hex(),
            "722c8c993fd75a7627d69ed941344fe2a1423a3e75efd3e6778a142884227104"
        );
    }

    #[test]
    fn into_vec_works() {
        let checksum = Checksum::generate(&[12u8; 17]);
        let as_vec: Vec<u8> = checksum.into();
        assert_eq!(as_vec, checksum.0);
    }
}
