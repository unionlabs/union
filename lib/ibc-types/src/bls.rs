use core::fmt;
use std::ops::{Deref, DerefMut};

use milagro_bls::{AmclError, G2_BYTES};
use serde::{Deserialize, Serialize};
use ssz::{Decode, Encode};
use ssz_types::FixedVector;
use tree_hash::TreeHash;
use typenum::U;

use crate::errors::InvalidLength;

const BLS_PUBLIC_KEY_BYTES_LEN: usize = 48;
const BLS_SIGNATURE_BYTES_LEN: usize = G2_BYTES;

#[derive(Clone)]
pub struct BlsSecretKey(milagro_bls::SecretKey);

impl TryFrom<Vec<u8>> for BlsSecretKey {
    type Error = AmclError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        milagro_bls::SecretKey::from_bytes(&data).map(Self)
    }
}

impl BlsSecretKey {
    #[must_use]
    pub fn public_key(&self) -> BlsPublicKey {
        BlsPublicKey::from(self.clone())
    }

    #[must_use]
    pub fn sign(&self, msg: &[u8]) -> BlsSignature {
        milagro_bls::Signature::new(msg, &self.0).into()
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
#[ssz(struct_behaviour = "transparent")]
pub struct BlsPublicKey(
    #[serde(with = "serde_utils::fixed_size_array")] pub [u8; BLS_PUBLIC_KEY_BYTES_LEN],
);

impl TreeHash for BlsPublicKey {
    fn tree_hash_type() -> tree_hash::TreeHashType {
        <FixedVector<u8, U<BLS_PUBLIC_KEY_BYTES_LEN>>>::tree_hash_type()
    }

    fn tree_hash_packed_encoding(&self) -> tree_hash::PackedEncoding {
        <FixedVector<u8, U<BLS_PUBLIC_KEY_BYTES_LEN>>>::tree_hash_packed_encoding(&self.0.into())
    }

    fn tree_hash_packing_factor() -> usize {
        <FixedVector<u8, U<BLS_PUBLIC_KEY_BYTES_LEN>>>::tree_hash_packing_factor()
    }

    fn tree_hash_root(&self) -> tree_hash::Hash256 {
        <FixedVector<u8, U<BLS_PUBLIC_KEY_BYTES_LEN>>>::tree_hash_root(&self.0.into())
    }
}

impl fmt::Debug for BlsPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PublicKey(0x{})", hex::encode(self.0))
    }
}

impl fmt::Display for BlsPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl From<milagro_bls::AggregatePublicKey> for BlsPublicKey {
    fn from(agg_pk: milagro_bls::AggregatePublicKey) -> Self {
        Self(
            milagro_bls::PublicKey {
                point: agg_pk.point,
            }
            .as_bytes(),
        )
    }
}

impl TryFrom<Vec<u8>> for BlsPublicKey {
    type Error = InvalidLength;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        data.try_into().map(Self).map_err(|invalid| InvalidLength {
            expected: BLS_PUBLIC_KEY_BYTES_LEN,
            found: invalid.len(),
        })
    }
}

impl AsRef<[u8]> for BlsPublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for BlsPublicKey {
    type Error = InvalidLength;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let data_len = data.len();
        data.try_into().map(Self).map_err(|_| InvalidLength {
            expected: BLS_PUBLIC_KEY_BYTES_LEN,
            found: data_len,
        })
    }
}

impl From<BlsPublicKey> for Vec<u8> {
    fn from(value: BlsPublicKey) -> Self {
        value.0.into()
    }
}

impl From<BlsSecretKey> for BlsPublicKey {
    fn from(key: BlsSecretKey) -> Self {
        Self(milagro_bls::PublicKey::from_secret_key(&key.0).as_bytes())
    }
}

impl TryFrom<&BlsPublicKey> for milagro_bls::PublicKey {
    type Error = AmclError;

    fn try_from(public_key: &BlsPublicKey) -> Result<Self, Self::Error> {
        Self::from_bytes(&public_key.0)
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
#[ssz(struct_behaviour = "transparent")]
pub struct BlsSignature(#[serde(with = "serde_utils::hex_string")] [u8; BLS_SIGNATURE_BYTES_LEN]);

impl TreeHash for BlsSignature {
    fn tree_hash_type() -> tree_hash::TreeHashType {
        <FixedVector<u8, U<BLS_SIGNATURE_BYTES_LEN>>>::tree_hash_type()
    }

    fn tree_hash_packed_encoding(&self) -> tree_hash::PackedEncoding {
        <FixedVector<u8, U<BLS_SIGNATURE_BYTES_LEN>>>::tree_hash_packed_encoding(&self.0.into())
    }

    fn tree_hash_packing_factor() -> usize {
        <FixedVector<u8, U<BLS_SIGNATURE_BYTES_LEN>>>::tree_hash_packing_factor()
    }

    fn tree_hash_root(&self) -> tree_hash::Hash256 {
        <FixedVector<u8, U<BLS_SIGNATURE_BYTES_LEN>>>::tree_hash_root(&self.0.into())
    }
}

impl BlsSignature {
    #[must_use]
    pub fn into_bytes(self) -> [u8; BLS_SIGNATURE_BYTES_LEN] {
        self.into()
    }
}

impl fmt::Debug for BlsSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlsSignature({self})")
    }
}

impl fmt::Display for BlsSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl Deref for BlsSignature {
    type Target = [u8; BLS_SIGNATURE_BYTES_LEN];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BlsSignature {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TryFrom<Vec<u8>> for BlsSignature {
    type Error = InvalidLength;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        data.try_into().map(Self).map_err(|invalid| InvalidLength {
            expected: BLS_SIGNATURE_BYTES_LEN,
            found: invalid.len(),
        })
    }
}

impl From<BlsSignature> for Vec<u8> {
    fn from(sig: BlsSignature) -> Self {
        sig.0.into()
    }
}

impl From<[u8; BLS_SIGNATURE_BYTES_LEN]> for BlsSignature {
    fn from(value: [u8; BLS_SIGNATURE_BYTES_LEN]) -> Self {
        Self(value)
    }
}

impl From<BlsSignature> for [u8; BLS_SIGNATURE_BYTES_LEN] {
    fn from(value: BlsSignature) -> Self {
        value.0
    }
}

impl TryFrom<&BlsSignature> for milagro_bls::Signature {
    type Error = AmclError;

    fn try_from(signature: &BlsSignature) -> Result<Self, Self::Error> {
        Self::from_bytes(signature.0.as_ref())
    }
}

impl From<milagro_bls::Signature> for BlsSignature {
    fn from(signature: milagro_bls::Signature) -> Self {
        signature.as_bytes().into()
    }
}
