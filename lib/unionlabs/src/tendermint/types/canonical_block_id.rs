use macros::model;

use crate::{hash::H256, tendermint::types::canonical_block_header::CanonicalPartSetHeader};

#[model(proto(raw(protos::tendermint::types::CanonicalBlockId), from))]
pub struct CanonicalBlockId {
    pub hash: H256,
    pub part_set_header: CanonicalPartSetHeader,
}

#[cfg(feature = "proto")]
impl From<CanonicalBlockId> for protos::tendermint::types::CanonicalBlockId {
    fn from(value: CanonicalBlockId) -> Self {
        Self {
            hash: value.hash.into(),
            part_set_header: Some(value.part_set_header.into()),
        }
    }
}
