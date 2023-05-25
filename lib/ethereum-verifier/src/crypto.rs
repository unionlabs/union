use crate::{byte_vector::ByteVector, Error};
use core::ops::{Deref, DerefMut};
use ssz_rs::prelude::*;
use std::fmt;

const BLS_PUBLIC_KEY_BYTES_LEN: usize = 48;
const BLS_SIGNATURE_BYTES_LEN: usize = 96;

pub fn fast_aggregate_verify(
    public_keys: &[&BlsPublicKey],
    msg: &[u8],
    signature: &BlsSignature,
) -> Result<(), Error> {
    let public_keys = public_keys
        .iter()
        .cloned()
        .map(milagro_bls::PublicKey::try_from)
        .collect::<Result<Vec<milagro_bls::PublicKey>, Error>>()?;
    let public_keys: Vec<&milagro_bls::PublicKey> = public_keys.iter().collect();
    let signature: milagro_bls::Signature = signature.try_into()?;
    let aggregate_signature = milagro_bls::AggregateSignature::aggregate(&[&signature]);
    let aggregate_pubkey =
        milagro_bls::AggregatePublicKey::aggregate(&public_keys).map_err(Error::Bls)?;

    if aggregate_signature.fast_aggregate_verify_pre_aggregated(msg, &aggregate_pubkey) {
        Ok(())
    } else {
        Err(Error::InvalidSignature)
    }
}

pub fn eth_aggregate_public_keys(public_keys: &[BlsPublicKey]) -> Result<BlsPublicKey, Error> {
    if public_keys.is_empty() {
        return Err(Error::EmptyAggregate);
    }
    let public_keys = public_keys
        .iter()
        .map(milagro_bls::PublicKey::try_from)
        .collect::<Result<Vec<milagro_bls::PublicKey>, Error>>()?;
    let public_keys: Vec<&milagro_bls::PublicKey> = public_keys.iter().collect();

    milagro_bls::AggregatePublicKey::aggregate(&public_keys)
        .map(TryFrom::try_from)
        .map_err(Error::Bls)?
}

#[derive(Clone)]
pub struct BlsSecretKey(milagro_bls::SecretKey);

impl TryFrom<&[u8]> for BlsSecretKey {
    type Error = Error;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let inner = milagro_bls::SecretKey::from_bytes(data).map_err(Error::Bls)?;
        Ok(Self(inner))
    }
}

impl BlsSecretKey {
    pub fn public_key(&self) -> BlsPublicKey {
        BlsPublicKey::try_from(self.clone()).unwrap()
    }

    pub fn sign(&self, msg: &[u8]) -> Result<BlsSignature, Error> {
        let signature = milagro_bls::Signature::new(msg, &self.0);
        BlsSignature::try_from(signature)
    }
}

#[derive(
    Clone, Default, Hash, PartialEq, Eq, SimpleSerialize, serde::Serialize, serde::Deserialize,
)]
pub struct BlsPublicKey(ByteVector<BLS_PUBLIC_KEY_BYTES_LEN>);

impl fmt::Debug for BlsPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = &self.0;
        write!(f, "PublicKey({inner})")
    }
}

impl fmt::Display for BlsPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = &self.0;
        write!(f, "{inner}")
    }
}

impl Deref for BlsPublicKey {
    type Target = ByteVector<BLS_PUBLIC_KEY_BYTES_LEN>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BlsPublicKey {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TryFrom<milagro_bls::AggregatePublicKey> for BlsPublicKey {
    type Error = Error;

    fn try_from(agg_pk: milagro_bls::AggregatePublicKey) -> Result<Self, Self::Error> {
        BlsPublicKey::try_from(
            milagro_bls::PublicKey {
                point: agg_pk.point,
            }
            .as_bytes()
            .as_ref(),
        )
    }
}

impl TryFrom<&[u8]> for BlsPublicKey {
    type Error = Error;

    fn try_from(data: &[u8]) -> Result<Self, Error> {
        let inner = ByteVector::try_from(data).map_err(|_| Error::InvalidPublicKey)?;
        Ok(Self(inner))
    }
}

impl TryFrom<BlsSecretKey> for BlsPublicKey {
    type Error = Error;

    fn try_from(key: BlsSecretKey) -> Result<Self, Self::Error> {
        Ok(Self(
            ByteVector::try_from(
                milagro_bls::PublicKey::from_secret_key(&key.0)
                    .as_bytes()
                    .as_slice(),
            )
            .map_err(|_| Error::InvalidPublicKey)?,
        ))
    }
}

impl TryFrom<&BlsPublicKey> for milagro_bls::PublicKey {
    type Error = Error;

    fn try_from(public_key: &BlsPublicKey) -> Result<Self, Error> {
        let pk = Self::from_bytes(public_key.0.as_ref()).map_err(Error::Bls)?;
        if !pk.key_validate() {
            Err(Error::InvalidPublicKey)
        } else {
            Ok(pk)
        }
    }
}

#[derive(
    Clone, Default, Hash, PartialEq, Eq, SimpleSerialize, serde::Serialize, serde::Deserialize,
)]
pub struct BlsSignature(ByteVector<BLS_SIGNATURE_BYTES_LEN>);

impl fmt::Debug for BlsSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = &self.0;
        write!(f, "Signature({inner})")
    }
}

impl fmt::Display for BlsSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = &self.0;
        write!(f, "{inner}")
    }
}

impl Deref for BlsSignature {
    type Target = ByteVector<BLS_SIGNATURE_BYTES_LEN>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BlsSignature {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TryFrom<&[u8]> for BlsSignature {
    type Error = Error;

    fn try_from(data: &[u8]) -> Result<Self, Error> {
        let inner = ByteVector::try_from(data).map_err(|_| Error::InvalidSignature)?;
        Ok(Self(inner))
    }
}

impl TryFrom<&BlsSignature> for milagro_bls::Signature {
    type Error = Error;

    fn try_from(signature: &BlsSignature) -> Result<Self, Self::Error> {
        Self::from_bytes(signature.0.as_ref()).map_err(Error::Bls)
    }
}

impl TryFrom<milagro_bls::Signature> for BlsSignature {
    type Error = Error;

    fn try_from(signature: milagro_bls::Signature) -> Result<Self, Self::Error> {
        TryFrom::try_from(signature.as_bytes().as_ref())
    }
}
