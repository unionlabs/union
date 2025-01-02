use macros::model;
use unionlabs_bytes::Bytes;

#[model(proto(raw(protos::ibc::core::commitment::v1::MerklePrefix), into, from))]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
pub struct MerklePrefix {
    pub key_prefix: Bytes,
}

impl From<protos::ibc::core::commitment::v1::MerklePrefix> for MerklePrefix {
    fn from(proto: protos::ibc::core::commitment::v1::MerklePrefix) -> Self {
        Self {
            key_prefix: proto.key_prefix.into(),
        }
    }
}

impl From<MerklePrefix> for protos::ibc::core::commitment::v1::MerklePrefix {
    fn from(value: MerklePrefix) -> Self {
        Self {
            key_prefix: value.key_prefix.into(),
        }
    }
}
