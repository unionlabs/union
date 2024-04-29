#[cfg(feature = "ethabi")]
use contracts::glue::IbcCoreCommitmentV1MerkleRootData;
use macros::model;

use crate::{errors::InvalidLength, hash::H256};

// #[cfg_attr(
//     feature = "ethabi",
//     derive(
//         ethers_contract_derive::EthAbiType,
//         ethers_contract_derive::EthAbiCodec
//     )
// )]
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

#[derive(Debug, PartialEq, Clone)]
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

#[cfg(feature = "ethabi")]
impl TryFrom<IbcCoreCommitmentV1MerkleRootData> for MerkleRoot {
    type Error = TryFromMerkleRootError;

    fn try_from(value: IbcCoreCommitmentV1MerkleRootData) -> Result<Self, Self::Error> {
        Ok(Self {
            hash: value
                .hash
                .try_into()
                .map_err(TryFromMerkleRootError::Hash)?,
        })
    }
}
