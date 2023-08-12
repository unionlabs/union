use serde::{Deserialize, Serialize};

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
