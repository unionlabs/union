use macros::model;

use crate::{errors::InvalidLength, hash::H256};

#[derive(Default)]
#[model(proto(raw(protos::tendermint::types::PartSetHeader), into, from))]
pub struct PartSetHeader {
    pub total: u32,
    pub hash: H256,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TryFromPartSetHeaderError {
    Hash(InvalidLength),
}

impl TryFrom<protos::tendermint::types::PartSetHeader> for PartSetHeader {
    type Error = TryFromPartSetHeaderError;

    fn try_from(value: protos::tendermint::types::PartSetHeader) -> Result<Self, Self::Error> {
        Ok(Self {
            total: value.total,
            hash: value
                .hash
                .try_into()
                .map_err(TryFromPartSetHeaderError::Hash)?,
        })
    }
}

impl From<PartSetHeader> for protos::tendermint::types::PartSetHeader {
    fn from(value: PartSetHeader) -> Self {
        Self {
            total: value.total,
            hash: value.hash.into(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<PartSetHeader> for contracts::glue::TendermintTypesPartSetHeaderData {
    fn from(value: PartSetHeader) -> Self {
        Self {
            total: value.total,
            hash: value.hash.into_bytes().into(),
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
            hash: value
                .hash
                .to_vec()
                .try_into()
                .map_err(TryFromEthAbiPartSetHeaderError::Hash)?,
        })
    }
}
