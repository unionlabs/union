use serde::{Deserialize, Serialize};
use unionlabs::primitives::{encoding::Base64, Bytes};

// TODO: These are fixed sizes, not arbitrary bytes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum PublicKey {
    #[serde(rename = "tendermint/PubKeyEd25519")]
    Ed25519(Bytes<Base64>),
    #[serde(rename = "tendermint/PubKeySecp256k1")]
    Secp256k1(Bytes<Base64>),
    #[serde(rename = "cometbft/PubKeyBls12_381")]
    Bls12_381(Bytes<Base64>),
    #[serde(rename = "cometbft/PubKeyBn254", alias = "tendermint/PubKeyBn254")]
    Bn254(Bytes<Base64>),
}

impl PublicKey {
    pub fn inner(&self) -> &[u8] {
        match self {
            PublicKey::Ed25519(bytes)
            | PublicKey::Secp256k1(bytes)
            | PublicKey::Bls12_381(bytes)
            | PublicKey::Bn254(bytes) => bytes,
        }
    }
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{errors::MissingField, impl_proto_via_try_from_into, required};

    use crate::crypto::public_key::PublicKey;

    impl_proto_via_try_from_into!(PublicKey => protos::cometbft::crypto::v1::PublicKey);

    impl From<PublicKey> for protos::cometbft::crypto::v1::PublicKey {
        fn from(value: PublicKey) -> Self {
            Self {
                sum: Some(match value {
                    PublicKey::Ed25519(key) => {
                        protos::cometbft::crypto::v1::public_key::Sum::Ed25519(key.to_vec())
                    }
                    PublicKey::Secp256k1(key) => {
                        protos::cometbft::crypto::v1::public_key::Sum::Secp256k1(key.to_vec())
                    }
                    PublicKey::Bn254(key) => {
                        protos::cometbft::crypto::v1::public_key::Sum::Bn254(key.to_vec())
                    }
                    PublicKey::Bls12_381(key) => {
                        protos::cometbft::crypto::v1::public_key::Sum::Bls12381(key.to_vec())
                    }
                }),
            }
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
    }

    impl TryFrom<protos::cometbft::crypto::v1::PublicKey> for PublicKey {
        type Error = Error;

        fn try_from(value: protos::cometbft::crypto::v1::PublicKey) -> Result<Self, Self::Error> {
            Ok(match required!(value.sum)? {
                protos::cometbft::crypto::v1::public_key::Sum::Ed25519(key) => {
                    Self::Ed25519(key.into())
                }
                protos::cometbft::crypto::v1::public_key::Sum::Secp256k1(key) => {
                    Self::Secp256k1(key.into())
                }
                protos::cometbft::crypto::v1::public_key::Sum::Bn254(key) => {
                    Self::Bn254(key.into())
                }
                protos::cometbft::crypto::v1::public_key::Sum::Bls12381(key) => {
                    Self::Bls12_381(key.into())
                }
            })
        }
    }

    impl From<PublicKey> for protos::tendermint::crypto::PublicKey {
        fn from(value: PublicKey) -> Self {
            Self {
                sum: Some(match value {
                    PublicKey::Ed25519(key) => {
                        protos::tendermint::crypto::public_key::Sum::Ed25519(key.to_vec())
                    }
                    PublicKey::Secp256k1(key) => {
                        protos::tendermint::crypto::public_key::Sum::Secp256k1(key.to_vec())
                    }
                    PublicKey::Bn254(key) => {
                        protos::tendermint::crypto::public_key::Sum::Bn254(key.to_vec())
                    }
                    PublicKey::Bls12_381(key) => {
                        protos::tendermint::crypto::public_key::Sum::Bls12381(key.to_vec())
                    }
                }),
            }
        }
    }

    impl TryFrom<protos::tendermint::crypto::PublicKey> for PublicKey {
        type Error = Error;

        fn try_from(value: protos::tendermint::crypto::PublicKey) -> Result<Self, Self::Error> {
            Ok(match required!(value.sum)? {
                protos::tendermint::crypto::public_key::Sum::Ed25519(key) => {
                    Self::Ed25519(key.into())
                }
                protos::tendermint::crypto::public_key::Sum::Secp256k1(key) => {
                    Self::Secp256k1(key.into())
                }
                protos::tendermint::crypto::public_key::Sum::Bn254(key) => Self::Bn254(key.into()),
                protos::tendermint::crypto::public_key::Sum::Bls12381(key) => {
                    Self::Bls12_381(key.into())
                }
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use unionlabs::test_utils::{assert_json_roundtrip, assert_proto_roundtrip};

    use super::*;

    #[test]
    fn roundtrip() {
        let json = r#"
            {
              "type": "cometbft/PubKeyBls12_381",
              "value": "qU0dNr3Bzxn6J1+beEAyuCm1f+Nd1P5nX+AyW6ODjmFChhRw5DwS25BIcoBtxuVx"
            }
        "#;

        let public_key = serde_json::from_str::<PublicKey>(json).unwrap();

        assert_json_roundtrip(&public_key);
        assert_proto_roundtrip(&public_key);
    }
}
