use serde::{Deserialize, Serialize};

use crate::{
    errors::{required, MissingField},
    Proto, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PublicKey {
    Ed25519(#[serde(with = "::serde_utils::hex_string")] Vec<u8>),
    Secp256k1(#[serde(with = "::serde_utils::hex_string")] Vec<u8>),
    Bn254(#[serde(with = "::serde_utils::hex_string")] Vec<u8>),
}

impl From<PublicKey> for protos::tendermint::crypto::PublicKey {
    fn from(value: PublicKey) -> Self {
        Self {
            sum: Some(match value {
                PublicKey::Ed25519(key) => {
                    protos::tendermint::crypto::public_key::Sum::Ed25519(key)
                }
                PublicKey::Secp256k1(key) => {
                    protos::tendermint::crypto::public_key::Sum::Secp256k1(key)
                }
                PublicKey::Bn254(key) => protos::tendermint::crypto::public_key::Sum::Bn254(key),
            }),
        }
    }
}

#[derive(Debug)]
pub enum TryFromPublicKeyError {
    MissingField(MissingField),
}

impl TryFrom<protos::tendermint::crypto::PublicKey> for PublicKey {
    type Error = TryFromPublicKeyError;

    fn try_from(value: protos::tendermint::crypto::PublicKey) -> Result<Self, Self::Error> {
        Ok(match required!(value.sum)? {
            protos::tendermint::crypto::public_key::Sum::Ed25519(key) => PublicKey::Ed25519(key),
            protos::tendermint::crypto::public_key::Sum::Secp256k1(key) => {
                PublicKey::Secp256k1(key)
            }
            protos::tendermint::crypto::public_key::Sum::Bn254(key) => PublicKey::Bn254(key),
        })
    }
}

impl Proto for PublicKey {
    type Proto = protos::tendermint::crypto::PublicKey;
}

impl TypeUrl for protos::tendermint::crypto::PublicKey {
    const TYPE_URL: &'static str = "/tendermint.crypto.PublicKey";
}
