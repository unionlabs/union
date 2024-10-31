use serde::{Deserialize, Serialize};
use unionlabs::hash::{hash_v2::HexUnprefixed, H256};

use crate::types::canonical_part_set_header::CanonicalPartSetHeader;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalBlockId {
    pub hash: H256<HexUnprefixed>,
    pub part_set_header: CanonicalPartSetHeader,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::types::canonical_block_id::CanonicalBlockId;

    impl From<CanonicalBlockId> for protos::cometbft::types::v1::CanonicalBlockId {
        fn from(value: CanonicalBlockId) -> Self {
            Self {
                hash: value.hash.into(),
                part_set_header: Some(value.part_set_header.into()),
            }
        }
    }

    impl From<CanonicalBlockId> for protos::tendermint::types::CanonicalBlockId {
        fn from(value: CanonicalBlockId) -> Self {
            Self {
                hash: value.hash.into(),
                part_set_header: Some(value.part_set_header.into()),
            }
        }
    }
}
