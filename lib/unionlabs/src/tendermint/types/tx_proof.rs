use macros::model;

use crate::{hash::H256, tendermint::crypto::proof::Proof};

#[model(proto(raw(protos::tendermint::types::TxProof), into, from))]
pub struct TxProof {
    pub root_hash: H256,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub data: Vec<u8>,
    pub proof: Proof,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        errors::{required, InvalidLength, MissingField},
        tendermint::{crypto::proof::proto::TryFromProofError, types::tx_proof::TxProof},
    };

    impl From<TxProof> for protos::tendermint::types::TxProof {
        fn from(value: TxProof) -> Self {
            Self {
                root_hash: value.root_hash.into(),
                data: value.data,
                proof: Some(value.proof.into()),
            }
        }
    }

    impl TryFrom<protos::tendermint::types::TxProof> for TxProof {
        type Error = TryFromTxProofError;

        fn try_from(value: protos::tendermint::types::TxProof) -> Result<Self, Self::Error> {
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
}
