use std::fmt::Debug;

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum PublicKey {
    Ed25519(#[serde(with = "::serde_utils::hex_string")] Vec<u8>),
    Secp256k1(#[serde(with = "::serde_utils::hex_string")] Vec<u8>),
    Bn254(#[serde(with = "::serde_utils::hex_string")] Vec<u8>),
}

impl Debug for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PublicKey::Ed25519(vec) => f
                .debug_tuple("Ed25519")
                .field(&serde_utils::to_hex(vec))
                .finish(),
            PublicKey::Secp256k1(vec) => f
                .debug_tuple("Secp256k1")
                .field(&serde_utils::to_hex(vec))
                .finish(),
            PublicKey::Bn254(vec) => f
                .debug_tuple("Bn254")
                .field(&serde_utils::to_hex(vec))
                .finish(),
        }
    }
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
