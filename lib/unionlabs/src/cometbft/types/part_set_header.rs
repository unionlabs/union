use macros::model;

use crate::{cometbft::types::block_id::maybe_empty_h256, errors::InvalidLength, hash::H256};

#[derive(Default)]
#[model(proto(raw(protos::cometbft::types::v1::PartSetHeader), into, from))]
pub struct PartSetHeader {
    pub total: u32,
    /// Hash of the previous block. This is only None on block 1, as the genesis block does not have a hash.
    pub hash: Option<H256>,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromPartSetHeaderError {
    #[error("invalid hash")]
    Hash(#[source] InvalidLength),
}

impl TryFrom<protos::cometbft::types::v1::PartSetHeader> for PartSetHeader {
    type Error = TryFromPartSetHeaderError;

    fn try_from(value: protos::cometbft::types::v1::PartSetHeader) -> Result<Self, Self::Error> {
        Ok(Self {
            total: value.total,
            hash: maybe_empty_h256(&value.hash).map_err(TryFromPartSetHeaderError::Hash)?,
        })
    }
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

#[cfg(feature = "ethabi")]
impl From<PartSetHeader> for contracts::glue::TendermintTypesPartSetHeaderData {
    fn from(value: PartSetHeader) -> Self {
        Self {
            total: value.total,
            hash: value.hash.map(|h| h.get().into()).unwrap_or_default(),
        }
    }
}

#[cfg(feature = "ethabi")]
#[derive(Debug, Clone, PartialEq)]
pub enum TryFromEthAbiPartSetHeaderError {
    Hash(InvalidLength),
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::glue::TendermintTypesPartSetHeaderData> for PartSetHeader {
    type Error = TryFromEthAbiPartSetHeaderError;

    fn try_from(
        value: contracts::glue::TendermintTypesPartSetHeaderData,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            total: value.total,
            hash: maybe_empty_h256(&value.hash).map_err(TryFromEthAbiPartSetHeaderError::Hash)?,
        })
    }
}
