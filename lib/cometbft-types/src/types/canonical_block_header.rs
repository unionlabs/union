use serde::{Deserialize, Serialize};
use unionlabs::hash::H256;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalPartSetHeader {
    pub total: u32,
    pub hash: H256,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::types::canonical_block_header::CanonicalPartSetHeader;

    impl From<CanonicalPartSetHeader> for protos::tendermint::types::CanonicalPartSetHeader {
        fn from(value: CanonicalPartSetHeader) -> Self {
            Self {
                hash: value.hash.into(),
                total: value.total,
            }
        }
    }
}
