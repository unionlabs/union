use macros::model;

#[model(proto(raw(protos::ibc::core::commitment::v1::MerklePrefix), into, from))]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
pub struct MerklePrefix {
    #[serde(with = "::serde_utils::hex_string")]
    #[cfg_attr(feature = "schemars", schemars(with = "String"))]
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
