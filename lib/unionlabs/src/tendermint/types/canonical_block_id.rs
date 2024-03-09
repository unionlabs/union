use macros::proto;
use serde::{Deserialize, Serialize};

use crate::{hash::H256, tendermint::types::canonical_block_header::CanonicalPartSetHeader};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[proto(raw = protos::tendermint::types::CanonicalBlockId, from)]
pub struct CanonicalBlockId {
    pub hash: H256,
    pub part_set_header: CanonicalPartSetHeader,
}

impl From<CanonicalBlockId> for protos::tendermint::types::CanonicalBlockId {
    fn from(value: CanonicalBlockId) -> Self {
        Self {
            hash: value.hash.into(),
            part_set_header: Some(value.part_set_header.into()),
        }
    }
}
