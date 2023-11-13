use serde::{Deserialize, Serialize};

use crate::{
    hash::H256, tendermint::types::canonical_block_header::CanonicalPartSetHeader, Proto, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalBlockId {
    pub hash: H256,
    pub part_set_header: CanonicalPartSetHeader,
}

impl Proto for CanonicalBlockId {
    type Proto = protos::tendermint::types::CanonicalBlockId;
}

impl TypeUrl for protos::tendermint::types::CanonicalBlockId {
    const TYPE_URL: &'static str = "/tendermint.types.CanonicalBlockId";
}

impl From<CanonicalBlockId> for protos::tendermint::types::CanonicalBlockId {
    fn from(value: CanonicalBlockId) -> Self {
        Self {
            hash: value.hash.into(),
            part_set_header: Some(value.part_set_header.into()),
        }
    }
}
