use serde::{Deserialize, Serialize};
use unionlabs::hash::{hash_v2::HexUnprefixed, H256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalPartSetHeader {
    pub total: u32,
    pub hash: H256<HexUnprefixed>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::types::canonical_part_set_header::CanonicalPartSetHeader;

    impl From<CanonicalPartSetHeader> for protos::cometbft::types::v1::CanonicalPartSetHeader {
        fn from(value: CanonicalPartSetHeader) -> Self {
            Self {
                hash: value.hash.into(),
                total: value.total,
            }
        }
    }

    impl From<CanonicalPartSetHeader> for protos::tendermint::types::CanonicalPartSetHeader {
        fn from(value: CanonicalPartSetHeader) -> Self {
            Self {
                hash: value.hash.into(),
                total: value.total,
            }
        }
    }
}
