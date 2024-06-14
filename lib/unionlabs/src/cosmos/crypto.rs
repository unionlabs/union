use core::fmt::Debug;

use macros::model;

use crate::{
    encoding::Decode, errors::InvalidLength, google::protobuf::any::Any, TryFromProtoBytesError,
    TypeUrl,
};

pub mod bn254;
pub mod ed25519;
pub mod multisig;
pub mod secp256k1;
pub mod secp256r1;

#[model(proto(raw(protos::google::protobuf::Any), into, from))]
pub enum AnyPubKey {
    Bn254(bn254::PubKey),
    Ed25519(ed25519::PubKey),
    Secp256k1(secp256k1::PubKey),
}

impl AnyPubKey {
    #[must_use]
    pub fn as_bn254(&self) -> Option<&bn254::PubKey> {
        if let Self::Bn254(v) = self {
            Some(v)
        } else {
            None
        }
    }

    #[must_use]
    pub fn as_ed25519(&self) -> Option<&ed25519::PubKey> {
        if let Self::Ed25519(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromAnyPubKeyError {
    // TODO: This is also in any.rs, find a way to generalize?
    #[error(
        "invalid type url `{found}`, expected one of: {}",
        expected.iter().map(|x| format!("`{x}`")).collect::<Vec<_>>().join(", ")
    )]
    InvalidTypeUrl {
        found: String,
        expected: Vec<String>,
    },
    #[error("unable to decode pub key from proto bytes")]
    TryFromProto(TryFromProtoBytesError<InvalidLength>),
}

impl TryFrom<protos::google::protobuf::Any> for AnyPubKey {
    type Error = TryFromAnyPubKeyError;

    fn try_from(value: protos::google::protobuf::Any) -> Result<Self, Self::Error> {
        if value.type_url == bn254::PubKey::type_url() {
            bn254::PubKey::decode(&value.value)
                .map(Self::Bn254)
                .map_err(TryFromAnyPubKeyError::TryFromProto)
        } else if value.type_url == ed25519::PubKey::type_url() {
            ed25519::PubKey::decode(&value.value)
                .map(Self::Ed25519)
                .map_err(TryFromAnyPubKeyError::TryFromProto)
        } else if value.type_url == secp256k1::PubKey::type_url() {
            secp256k1::PubKey::decode(&value.value)
                .map(Self::Secp256k1)
                .map_err(TryFromAnyPubKeyError::TryFromProto)
        } else {
            Err(TryFromAnyPubKeyError::InvalidTypeUrl {
                found: value.type_url,
                expected: vec![
                    bn254::PubKey::type_url(),
                    ed25519::PubKey::type_url(),
                    secp256k1::PubKey::type_url(),
                ],
            })
        }
    }
}

impl From<AnyPubKey> for protos::google::protobuf::Any {
    fn from(value: AnyPubKey) -> Self {
        match value {
            AnyPubKey::Bn254(key) => Any(key).into(),
            AnyPubKey::Ed25519(key) => Any(key).into(),
            AnyPubKey::Secp256k1(key) => Any(key).into(),
        }
    }
}

impl From<Any<bn254::PubKey>> for AnyPubKey {
    fn from(value: Any<bn254::PubKey>) -> Self {
        Self::Bn254(value.0)
    }
}

impl From<Any<ed25519::PubKey>> for AnyPubKey {
    fn from(value: Any<ed25519::PubKey>) -> Self {
        Self::Ed25519(value.0)
    }
}

impl From<Any<secp256k1::PubKey>> for AnyPubKey {
    fn from(value: Any<secp256k1::PubKey>) -> Self {
        Self::Secp256k1(value.0)
    }
}
