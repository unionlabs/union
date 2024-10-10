use macros::model;

use crate::{errors::InvalidLength, hash::H256};

#[model(proto(raw(protos::ibc::core::commitment::v1::MerkleRoot), into, from))]
pub struct MerkleRoot {
    pub hash: H256,
}

impl From<MerkleRoot> for protos::ibc::core::commitment::v1::MerkleRoot {
    fn from(value: MerkleRoot) -> Self {
        Self {
            hash: value.hash.into(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromMerkleRootError {
    #[error("invalid hash")]
    Hash(#[from] InvalidLength),
}

impl TryFrom<protos::ibc::core::commitment::v1::MerkleRoot> for MerkleRoot {
    type Error = TryFromMerkleRootError;

    fn try_from(value: protos::ibc::core::commitment::v1::MerkleRoot) -> Result<Self, Self::Error> {
        Ok(Self {
            hash: value
                .hash
                .try_into()
                .map_err(TryFromMerkleRootError::Hash)?,
        })
    }
}
