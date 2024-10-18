use macros::model;

use crate::hash::H256;

#[derive(Default)]
#[model(proto(raw(protos::tendermint::types::PartSetHeader), into, from))]
pub struct PartSetHeader {
    pub total: u32,
    /// Hash of the previous block. This is only None on block 1, as the genesis block does not have a hash.
    pub hash: Option<H256>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        errors::InvalidLength,
        tendermint::types::{block_id::proto::maybe_empty_h256, part_set_header::PartSetHeader},
    };

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum TryFromPartSetHeaderError {
        #[error("invalid hash")]
        Hash(#[source] InvalidLength),
    }

    impl TryFrom<protos::tendermint::types::PartSetHeader> for PartSetHeader {
        type Error = TryFromPartSetHeaderError;

        fn try_from(value: protos::tendermint::types::PartSetHeader) -> Result<Self, Self::Error> {
            Ok(Self {
                total: value.total,
                hash: maybe_empty_h256(&value.hash).map_err(TryFromPartSetHeaderError::Hash)?,
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
