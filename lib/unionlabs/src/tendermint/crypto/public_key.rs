use macros::model;

use crate::errors::{required, MissingField};

// TODO: These are fixed sizes, not arbitrary bytes
#[model(proto(raw(protos::tendermint::crypto::PublicKey), into, from))]
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
    Bn254(
        #[serde(with = "::serde_utils::hex_string")]
        #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
        Vec<u8>,
    ),
    Bls12_381(
        #[serde(with = "::serde_utils::hex_string")]
        #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
        Vec<u8>,
    ),
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
                PublicKey::Bls12_381(key) => {
                    protos::tendermint::crypto::public_key::Sum::Bls12_381(key)
                }
            }),
        }
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromPublicKeyError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
}

impl TryFrom<protos::tendermint::crypto::PublicKey> for PublicKey {
    type Error = TryFromPublicKeyError;

    fn try_from(value: protos::tendermint::crypto::PublicKey) -> Result<Self, Self::Error> {
        Ok(match required!(value.sum)? {
            protos::tendermint::crypto::public_key::Sum::Ed25519(key) => Self::Ed25519(key),
            protos::tendermint::crypto::public_key::Sum::Secp256k1(key) => Self::Secp256k1(key),
            protos::tendermint::crypto::public_key::Sum::Bn254(key) => Self::Bn254(key),
            protos::tendermint::crypto::public_key::Sum::Bls12_381(key) => Self::Bls12_381(key),
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
              "type": "cometbft/PubKeyBls12_381",
              "value": "qU0dNr3Bzxn6J1+beEAyuCm1f+Nd1P5nX+AyW6ODjmFChhRw5DwS25BIcoBtxuVx"
            }
        "#;

        let public_key_raw =
            serde_json::from_str::<protos::tendermint::crypto::PublicKey>(json).unwrap();

        assert_json_roundtrip(&public_key_raw);

        let public_key = PublicKey::try_from(public_key_raw).unwrap();

        assert_json_roundtrip(&public_key);
        assert_proto_roundtrip(&public_key);
    }
}
