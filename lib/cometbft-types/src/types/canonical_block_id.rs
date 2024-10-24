use serde::{Deserialize, Serialize};
use unionlabs::hash::H256;

use crate::types::canonical_block_header::CanonicalPartSetHeader;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalBlockId {
    pub hash: H256,
    pub part_set_header: CanonicalPartSetHeader,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::types::canonical_block_id::CanonicalBlockId;

    impl From<CanonicalBlockId> for protos::tendermint::types::CanonicalBlockId {
        fn from(value: CanonicalBlockId) -> Self {
            Self {
                hash: value.hash.into(),
                part_set_header: Some(value.part_set_header.into()),
            }
        }
    }
}
