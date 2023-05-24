use crate::{beacon::Root, internal_prelude::*, types::H256};
use displaydoc::Display;

#[derive(Debug, Display)]
pub enum Error {
    /// bls amcl error: `{0:?}`
    BLSAmclError(milagro_bls::AmclError),
    /// merkleization error: `{0:?}`
    MerkleizationError(ssz_rs::MerkleizationError),
    /// ssz deserialize error: `{0:?}`
    SSZDeserializeError(ssz_rs::DeserializeError),
    /// hex error: `{0:?}`
    FromHexError(hex::FromHexError),
    /// invalid bls signature length: `expected={0} actual={1}`
    InvalidBLSSignatureLenght(usize, usize),
    /// invalid bls public key length: `expected={0} actual={1}`
    InvalidBLSPublicKeyLength(usize, usize),
    /// bls aggreate public key mismatch
    BLSAggregatePublicKeyMismatch,
    /// invalid address length: `expected={0} actual={1}`
    InvalidAddressLength(usize, usize),
    /// other error: `{description}`
    Other { description: String },
}

#[derive(Debug, Display)]
pub enum MerkleError {
    /// invalid merkle branch error: leaf={0:?} branch={1:?} index={2:?} root={3:?}
    InvalidMerkleBranch(H256, Vec<H256>, u64, Root),
    /// too long merkle branch error: leaf={0:?} branch={1:?} index={2:?} root={3:?}
    TooLongMerkleBranch(H256, Vec<H256>, u64, Root),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl From<ssz_rs::MerkleizationError> for Error {
    fn from(value: ssz_rs::MerkleizationError) -> Self {
        Self::MerkleizationError(value)
    }
}

impl From<ssz_rs::DeserializeError> for Error {
    fn from(value: ssz_rs::DeserializeError) -> Self {
        Self::SSZDeserializeError(value)
    }
}

impl From<milagro_bls::AmclError> for Error {
    fn from(value: milagro_bls::AmclError) -> Self {
        Self::BLSAmclError(value)
    }
}

impl From<hex::FromHexError> for Error {
    fn from(value: hex::FromHexError) -> Self {
        Self::FromHexError(value)
    }
}
