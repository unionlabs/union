use serde::{Deserialize, Serialize};
use unionlabs::hash::H256;

use crate::crypto::proof::Proof;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TxProof {
    pub root_hash: H256,
    #[serde(with = "::serde_utils::hex_string")]
    pub data: Vec<u8>,
    pub proof: Proof,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{
        errors::{InvalidLength, MissingField},
        required,
    };

    use crate::{crypto::proof, types::tx_proof::TxProof};

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
        type Error = Error;

        fn try_from(value: protos::tendermint::types::TxProof) -> Result<Self, Self::Error> {
            Ok(Self {
                root_hash: value.root_hash.try_into()?,
                data: value.data,
                proof: required!(value.proof)?.try_into()?,
            })
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid root_hash")]
        RootHash(#[from] InvalidLength),
        #[error("invalid proof")]
        Proof(#[from] proof::proto::Error),
    }
}
