use macros::model;

use crate::{
    cometbft::crypto::proof::{Proof, TryFromProofError},
    errors::{required, InvalidLength, MissingField},
    hash::H256,
};

#[model(proto(raw(protos::cometbft::types::v1::TxProof), into, from))]
pub struct TxProof {
    pub root_hash: H256,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub data: Vec<u8>,
    pub proof: Proof,
}

impl From<TxProof> for protos::cometbft::types::v1::TxProof {
    fn from(value: TxProof) -> Self {
        Self {
            root_hash: value.root_hash.into(),
            data: value.data,
            proof: Some(value.proof.into()),
        }
    }
}

impl TryFrom<protos::cometbft::types::v1::TxProof> for TxProof {
    type Error = TryFromTxProofError;

    fn try_from(value: protos::cometbft::types::v1::TxProof) -> Result<Self, Self::Error> {
        Ok(Self {
            root_hash: value.root_hash.try_into()?,
            data: value.data,
            proof: required!(value.proof)?.try_into()?,
        })
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromTxProofError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid root_hash")]
    RootHash(#[from] InvalidLength),
    #[error("invalid proof")]
    Proof(#[from] TryFromProofError),
}
