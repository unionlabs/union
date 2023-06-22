use contracts::glue::IbcCoreCommitmentV1MerkleRootData;

#[derive(Debug, Clone)]
pub struct MerkleRoot {
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

impl From<MerkleRoot> for IbcCoreCommitmentV1MerkleRootData {
    fn from(value: MerkleRoot) -> Self {
        Self {
            hash: value.hash.into(),
        }
    }
}
