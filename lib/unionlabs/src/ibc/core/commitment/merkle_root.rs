#[cfg(feature = "ethabi")]
use contracts::glue::IbcCoreCommitmentV1MerkleRootData;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct MerkleRoot {
    // REVIEW: Should this be H256?
    pub hash: Vec<u8>,
}

impl From<MerkleRoot> for protos::ibc::core::commitment::v1::MerkleRoot {
    fn from(value: MerkleRoot) -> Self {
        Self { hash: value.hash }
    }
}

impl From<protos::ibc::core::commitment::v1::MerkleRoot> for MerkleRoot {
    fn from(value: protos::ibc::core::commitment::v1::MerkleRoot) -> Self {
        Self { hash: value.hash }
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
