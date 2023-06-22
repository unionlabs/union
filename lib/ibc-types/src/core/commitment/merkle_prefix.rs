use contracts::ibc_handler::IbcCoreCommitmentV1MerklePrefixData;

#[derive(Debug, Clone)]
pub struct MerklePrefix {
    pub key_prefix: Vec<u8>,
}

impl From<protos::ibc::core::commitment::v1::MerklePrefix> for MerklePrefix {
    fn from(proto: protos::ibc::core::commitment::v1::MerklePrefix) -> Self {
        Self {
            key_prefix: proto.key_prefix,
        }
    }
}

impl From<MerklePrefix> for protos::ibc::core::commitment::v1::MerklePrefix {
    fn from(value: MerklePrefix) -> Self {
        Self {
            key_prefix: value.key_prefix,
        }
    }
}

impl From<MerklePrefix> for IbcCoreCommitmentV1MerklePrefixData {
    fn from(value: MerklePrefix) -> Self {
        Self {
            key_prefix: value.key_prefix.into(),
        }
    }
}

impl From<IbcCoreCommitmentV1MerklePrefixData> for MerklePrefix {
    fn from(value: IbcCoreCommitmentV1MerklePrefixData) -> Self {
        Self {
            key_prefix: value.key_prefix.to_vec(),
        }
    }
}
