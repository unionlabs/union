use macros::model;

use crate::primitives::{FixedBytesError, H256, encoding::Base64};

#[model(proto(raw(protos::ibc::core::commitment::v1::MerkleRoot), into, from))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct MerkleRoot {
    pub hash: H256<Base64>,
}

impl From<H256<Base64>> for MerkleRoot {
    fn from(value: H256<Base64>) -> Self {
        Self { hash: value }
    }
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
    Hash(#[from] FixedBytesError),
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
