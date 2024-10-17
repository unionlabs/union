use macros::model;

#[cfg(feature = "ethabi")]
use crate::cometbft::types::part_set_header::TryFromEthAbiPartSetHeaderError;
use crate::{
    cometbft::types::part_set_header::{PartSetHeader, TryFromPartSetHeaderError},
    errors::{required, InvalidLength, MissingField},
    hash::H256,
};

#[derive(Default)]
#[model(proto(raw(protos::cometbft::types::v1::BlockId), into, from))]
pub struct BlockId {
    /// Hash of the previous block. This is only None on block 1, as the genesis block does not have a hash.
    pub hash: Option<H256>,
    pub part_set_header: PartSetHeader,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromBlockIdError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid hash")]
    Hash(#[source] InvalidLength),
    #[error("invalid part set header")]
    PartSetHeader(#[from] TryFromPartSetHeaderError),
}

impl TryFrom<protos::cometbft::types::v1::BlockId> for BlockId {
    type Error = TryFromBlockIdError;

    fn try_from(value: protos::cometbft::types::v1::BlockId) -> Result<Self, Self::Error> {
        Ok(Self {
            hash: maybe_empty_h256(&value.hash).map_err(TryFromBlockIdError::Hash)?,
            part_set_header: required!(value.part_set_header)?.try_into()?,
        })
    }
}

impl TryFrom<protos::tendermint::types::BlockId> for BlockId {
    type Error = TryFromBlockIdError;

    fn try_from(value: protos::tendermint::types::BlockId) -> Result<Self, Self::Error> {
        Ok(Self {
            hash: maybe_empty_h256(&value.hash).map_err(TryFromBlockIdError::Hash)?,
            part_set_header: required!(value.part_set_header)?.try_into()?,
        })
    }
}

pub(crate) fn maybe_empty_h256(value: &[u8]) -> Result<Option<H256>, InvalidLength> {
    Ok(if value.is_empty() {
        None
    } else {
        Some(
            value
                .try_into()
                .map_err(|err: InvalidLength| InvalidLength {
                    expected: crate::errors::ExpectedLength::Either(0, 32),
                    found: err.found,
                })?,
        )
    })
}

impl From<BlockId> for protos::cometbft::types::v1::BlockId {
    fn from(value: BlockId) -> Self {
        Self {
            hash: value.hash.map(Into::into).unwrap_or_default(),
            part_set_header: Some(value.part_set_header.into()),
        }
    }
}

impl From<BlockId> for protos::tendermint::types::BlockId {
    fn from(value: BlockId) -> Self {
        Self {
            hash: value.hash.map(Into::into).unwrap_or_default(),
            part_set_header: Some(value.part_set_header.into()),
        }
    }
}

#[test]
#[cfg(test)]
fn proto_roundtrip() {
    crate::test_utils::assert_proto_roundtrip(&BlockId {
        hash: Some([1; 32].into()),
        part_set_header: PartSetHeader {
            total: 1,
            hash: Some([2; 32].into()),
        },
    });

    crate::test_utils::assert_proto_roundtrip(&BlockId {
        hash: None,
        part_set_header: PartSetHeader {
            total: 1,
            hash: None,
        },
    });
}

#[cfg(feature = "ethabi")]
impl From<BlockId> for contracts::glue::TendermintTypesBlockIDData {
    fn from(value: BlockId) -> Self {
        Self {
            hash: value.hash.map(|h| h.get().into()).unwrap_or_default(),
            part_set_header: value.part_set_header.into(),
        }
    }
}

#[cfg(feature = "ethabi")]
#[derive(Debug, Clone, PartialEq)]
pub enum TryFromEthAbiBlockIdError {
    Hash(crate::errors::InvalidLength),
    PartSetHeader(TryFromEthAbiPartSetHeaderError),
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::glue::TendermintTypesBlockIDData> for BlockId {
    type Error = TryFromEthAbiBlockIdError;

    fn try_from(value: contracts::glue::TendermintTypesBlockIDData) -> Result<Self, Self::Error> {
        Ok(Self {
            hash: maybe_empty_h256(&value.hash).map_err(TryFromEthAbiBlockIdError::Hash)?,
            part_set_header: value
                .part_set_header
                .try_into()
                .map_err(TryFromEthAbiBlockIdError::PartSetHeader)?,
        })
    }
}
