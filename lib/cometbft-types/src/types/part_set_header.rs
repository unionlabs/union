use serde::{Deserialize, Serialize};
use unionlabs::hash::{hash_v2::HexUnprefixed, H256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PartSetHeader {
    pub total: u32,
    /// Hash of the previous block. This is only None on block 1, as the genesis block does not have a hash.
    pub hash: Option<H256<HexUnprefixed>>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::errors::InvalidLength;

    use crate::types::{block_id::proto::maybe_empty_h256, part_set_header::PartSetHeader};

    impl TryFrom<protos::cometbft::types::v1::PartSetHeader> for PartSetHeader {
        type Error = Error;

        fn try_from(
            value: protos::cometbft::types::v1::PartSetHeader,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                total: value.total,
                hash: maybe_empty_h256(&value.hash).map_err(Error::Hash)?,
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid hash")]
        Hash(#[source] InvalidLength),
    }

    impl From<PartSetHeader> for protos::cometbft::types::v1::PartSetHeader {
        fn from(value: PartSetHeader) -> Self {
            Self {
                total: value.total,
                hash: value.hash.map(Into::into).unwrap_or_default(),
            }
        }
    }

    impl TryFrom<protos::tendermint::types::PartSetHeader> for PartSetHeader {
        type Error = Error;

        fn try_from(value: protos::tendermint::types::PartSetHeader) -> Result<Self, Self::Error> {
            Ok(Self {
                total: value.total,
                hash: maybe_empty_h256(&value.hash).map_err(Error::Hash)?,
            })
        }
    }

    impl From<PartSetHeader> for protos::tendermint::types::PartSetHeader {
        fn from(value: PartSetHeader) -> Self {
            Self {
                total: value.total,
                hash: value.hash.map(Into::into).unwrap_or_default(),
            }
        }
    }
}
