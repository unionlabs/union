use macros::model;

use crate::errors::{required, MissingField};

// TODO: These are fixed sizes, not arbitrary bytes
#[model(proto(raw(protos::cometbft::crypto::v1::PublicKey), into, from))]
pub enum PublicKey {
    Ed25519(
        #[serde(with = "::serde_utils::hex_string")]
        #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
        Vec<u8>,
    ),
    Secp256k1(
        #[serde(with = "::serde_utils::hex_string")]
        #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
        Vec<u8>,
    ),
    Bls12381(
        #[serde(with = "::serde_utils::hex_string")]
        #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
        Vec<u8>,
    ),
    Bn254(
        #[serde(with = "::serde_utils::hex_string")]
        #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
        Vec<u8>,
    ),
}

impl From<PublicKey> for protos::cometbft::crypto::v1::PublicKey {
    fn from(value: PublicKey) -> Self {
        Self {
            sum: Some(match value {
                PublicKey::Ed25519(key) => {
                    protos::cometbft::crypto::v1::public_key::Sum::Ed25519(key)
                }
                PublicKey::Secp256k1(key) => {
                    protos::cometbft::crypto::v1::public_key::Sum::Secp256k1(key)
                }
                PublicKey::Bls12381(key) => {
                    protos::cometbft::crypto::v1::public_key::Sum::Bls12381(key)
                }
                PublicKey::Bn254(key) => protos::cometbft::crypto::v1::public_key::Sum::Bn254(key),
            }),
        }
    }
}

impl From<PublicKey> for protos::tendermint::crypto::PublicKey {
    fn from(value: PublicKey) -> Self {
        Self {
            sum: match value {
                PublicKey::Ed25519(key) => {
                    Some(protos::tendermint::crypto::public_key::Sum::Ed25519(key))
                }
                PublicKey::Secp256k1(key) => {
                    Some(protos::tendermint::crypto::public_key::Sum::Secp256k1(key))
                }
                PublicKey::Bls12381(_) | PublicKey::Bn254(_) => None,
            },
        }
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromPublicKeyError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
}

impl TryFrom<protos::cometbft::crypto::v1::PublicKey> for PublicKey {
    type Error = TryFromPublicKeyError;

    fn try_from(value: protos::cometbft::crypto::v1::PublicKey) -> Result<Self, Self::Error> {
        Ok(match required!(value.sum)? {
            protos::cometbft::crypto::v1::public_key::Sum::Ed25519(key) => Self::Ed25519(key),
            protos::cometbft::crypto::v1::public_key::Sum::Secp256k1(key) => Self::Secp256k1(key),
            protos::cometbft::crypto::v1::public_key::Sum::Bls12381(key) => Self::Bls12381(key),
            protos::cometbft::crypto::v1::public_key::Sum::Bn254(key) => Self::Bn254(key),
        })
    }
}

impl TryFrom<protos::tendermint::crypto::PublicKey> for PublicKey {
    type Error = TryFromPublicKeyError;

    fn try_from(value: protos::tendermint::crypto::PublicKey) -> Result<Self, Self::Error> {
        Ok(match required!(value.sum)? {
            protos::tendermint::crypto::public_key::Sum::Ed25519(key) => Self::Ed25519(key),
            protos::tendermint::crypto::public_key::Sum::Secp256k1(key) => Self::Secp256k1(key),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{assert_json_roundtrip, assert_proto_roundtrip};

    #[test]
    fn roundtrip() {
        let json = r#"
            {
              "type": "cometbft/PubKeyBls12381",
              "value": "qU0dNr3Bzxn6J1+beEAyuCm1f+Nd1P5nX+AyW6ODjmFChhRw5DwS25BIcoBtxuVx"
            }
        "#;

        let public_key_raw =
            serde_json::from_str::<protos::cometbft::crypto::v1::PublicKey>(json).unwrap();

        assert_json_roundtrip(&public_key_raw);

        let public_key = PublicKey::try_from(public_key_raw).unwrap();

        assert_json_roundtrip(&public_key);
        assert_proto_roundtrip(&public_key);
    }
}
