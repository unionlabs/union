use macros::model;

use crate::{cometbft::types::canonical_block_header::CanonicalPartSetHeader, hash::H256};

#[model(proto(raw(protos::cometbft::types::v1::CanonicalBlockId), from))]
pub struct CanonicalBlockId {
    pub hash: H256,
    pub part_set_header: CanonicalPartSetHeader,
}

impl From<CanonicalBlockId> for protos::cometbft::types::v1::CanonicalBlockId {
    fn from(value: CanonicalBlockId) -> Self {
        Self {
            hash: value.hash.into(),
            part_set_header: Some(value.part_set_header.into()),
        }
    }
}
