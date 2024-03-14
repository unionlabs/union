#[cfg(feature = "ethabi")]
use contracts::ibc_handler::IbcCoreCommitmentV1MerklePrefixData;
use macros::model;

#[model(proto(raw(protos::ibc::core::commitment::v1::MerklePrefix), into, from))]
pub struct MerklePrefix {
    #[serde(with = "::serde_utils::hex_string")]
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

#[cfg(feature = "ethabi")]
impl From<MerklePrefix> for IbcCoreCommitmentV1MerklePrefixData {
    fn from(value: MerklePrefix) -> Self {
        Self {
            key_prefix: value.key_prefix.into(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<IbcCoreCommitmentV1MerklePrefixData> for MerklePrefix {
    fn from(value: IbcCoreCommitmentV1MerklePrefixData) -> Self {
        Self {
            key_prefix: value.key_prefix.to_vec(),
        }
    }
}
