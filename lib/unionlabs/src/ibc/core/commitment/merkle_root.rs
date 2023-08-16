#[cfg(feature = "ethabi")]
use contracts::glue::IbcCoreCommitmentV1MerkleRootData;
use serde::{Deserialize, Serialize};

use crate::{errors::InvalidLength, ethereum::H256, Proto, TypeUrl};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug)]
pub enum TryFromMerkleRootError {
    Hash(InvalidLength),
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

#[cfg(feature = "ethabi")]
impl From<MerkleRoot> for IbcCoreCommitmentV1MerkleRootData {
    fn from(value: MerkleRoot) -> Self {
        Self {
            hash: value.hash.into(),
        }
    }
}

impl Proto for MerkleRoot {
    type Proto = protos::ibc::core::commitment::v1::MerkleRoot;
}

impl TypeUrl for protos::ibc::core::commitment::v1::MerkleRoot {
    const TYPE_URL: &'static str = "/ibc.core.commitment.v1.MerkleRoot";
}
