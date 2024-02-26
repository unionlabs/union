use poseidon_rs::{Fr, Poseidon};

use crate::{fr_from_little_endian, reverse_byte_order, Byte32};

pub const HASH_DOMAIN_ELEMS_BASE: usize = 256;
pub const HASH_DOMAIN_BYTE32: usize = 2 * HASH_DOMAIN_ELEMS_BASE;
pub const HASH_BYTE_LEN: usize = 32;

pub const ZERO_HASH: Hash = Hash([0u8; 32]);

lazy_static::lazy_static! {
    pub static ref POSEIDON: Poseidon = Poseidon::default();
}

pub fn copy_truncated(dst: &mut [u8], src: &[u8]) {
    if dst.len() >= src.len() {
        dst[..src.len()].copy_from_slice(src);
    } else {
        dst.copy_from_slice(&src[..dst.len()]);
    }
}

impl HashScheme for () {
    fn hash_scheme(_: &[Fr], _: &Fr) -> Fr {
        0u64.into()
    }
}

pub struct PoseidonHash;
impl HashScheme for PoseidonHash {
    fn hash_scheme(arr: &[Fr], domain: &Fr) -> Fr {
        match POSEIDON.hash_fixed_with_domain(arr, *domain) {
            Ok(output) => output,
            Err(err) => {
                panic!("inp: {arr:?}, domain: {domain:?}, err: {err:?}");
            }
        }
    }
}
pub trait HashScheme {
    fn hash_scheme(arr: &[Fr], domain: &Fr) -> Fr;
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Copy, PartialOrd, Ord)]
pub struct Hash([u8; 32]);

impl Hash {
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self == &ZERO_HASH
    }

    #[must_use]
    pub fn raw_bytes(&self) -> &[u8] {
        &self.0[..]
    }

    pub fn fr(&self) -> Result<Fr, String> {
        fr_from_little_endian(&self.0)
    }

    #[must_use]
    pub fn hex(&self) -> String {
        hex::encode(self.bytes())
    }

    #[cfg(test)]
    #[must_use]
    pub fn from_hex(val: &str) -> Option<Self> {
        let mut tmp = [0_u8; 32];
        hex::decode_to_slice(val, &mut tmp).ok()?;
        tmp.reverse();
        Some(Self(tmp))
    }

    #[cfg(test)]
    #[must_use]
    pub fn to_hex(&self) -> String {
        hex::encode(self.bytes())
    }

    #[must_use]
    pub fn bytes(&self) -> [u8; 32] {
        let mut dst = [0_u8; 32];
        reverse_byte_order(&mut dst, self.raw_bytes());
        dst
    }

    #[must_use]
    pub fn from_bytes(b: &[u8]) -> Self {
        let mut h = Hash::default();
        copy_truncated(&mut h.0, b);
        h.0.reverse();
        h
    }
}

#[cfg(test)]
impl From<&str> for Hash {
    fn from(val: &str) -> Self {
        Self::from_hex(val).unwrap()
    }
}

impl From<Fr> for Hash {
    fn from(fr: Fr) -> Self {
        (&fr).into()
    }
}

impl From<&Fr> for Hash {
    fn from(fr: &Fr) -> Self {
        Self(fr.to_little_endian())
    }
}

impl From<Byte32> for Hash {
    fn from(val: Byte32) -> Self {
        Self::from_bytes(val.bytes())
    }
}

impl From<&Byte32> for Hash {
    fn from(val: &Byte32) -> Self {
        Self::from_bytes(val.bytes())
    }
}
