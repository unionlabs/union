use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::{
    errors::InvalidLength, google::protobuf::any::Any, IntoProto, Proto, TryFromProto,
    TryFromProtoBytesError, TypeUrl,
};

pub mod bn254;
pub mod ed25519;
pub mod multisig;
pub mod secp256k1;
pub mod secp256r1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnyPubKey {
    Bn254(bn254::PubKey),
    Ed25519(ed25519::PubKey),
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

#[derive(Debug)]
pub enum TryFromAnyPubKeyError {
    IncorrectTypeUrl { found: String },
    TryFromProto(TryFromProtoBytesError<InvalidLength>),
}

impl TryFrom<protos::google::protobuf::Any> for AnyPubKey {
    type Error = TryFromAnyPubKeyError;

    fn try_from(value: protos::google::protobuf::Any) -> Result<Self, Self::Error> {
        match &*value.type_url {
            <bn254::PubKey as Proto>::Proto::TYPE_URL => {
                bn254::PubKey::try_from_proto_bytes(&value.value)
                    .map(Self::Bn254)
                    .map_err(TryFromAnyPubKeyError::TryFromProto)
            }
            <ed25519::PubKey as Proto>::Proto::TYPE_URL => {
                ed25519::PubKey::try_from_proto_bytes(&value.value)
                    .map(Self::Ed25519)
                    .map_err(TryFromAnyPubKeyError::TryFromProto)
            }

            _ => Err(TryFromAnyPubKeyError::IncorrectTypeUrl {
                found: value.type_url,
            }),
        }
    }
}

impl From<AnyPubKey> for protos::google::protobuf::Any {
    fn from(value: AnyPubKey) -> Self {
        match value {
            AnyPubKey::Bn254(key) => Any(key).into_proto(),
            AnyPubKey::Ed25519(key) => Any(key).into_proto(),
        }
    }
}

impl Proto for AnyPubKey {
    type Proto = protos::google::protobuf::Any;
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
