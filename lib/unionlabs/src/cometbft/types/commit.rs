use macros::model;

#[cfg(feature = "ethabi")]
use crate::cometbft::types::{
    block_id::TryFromEthAbiBlockIdError, commit_sig::TryFromEthAbiCommitSigError,
};
use crate::{
    bounded::{BoundedI32, BoundedI64, BoundedIntError},
    cometbft::types::{
        block_id::{BlockId, TryFromBlockIdError},
        commit_sig::{CommitSig, TryFromCommitSigError},
    },
    errors::{required, MissingField},
};

#[model(proto(raw(protos::cometbft::types::v1::Commit), into, from))]
pub struct Commit {
    pub height: BoundedI64<0, { i64::MAX }>,
    pub round: BoundedI32<0, { i32::MAX }>,
    pub block_id: BlockId,
    pub signatures: Vec<CommitSig>,
}

impl From<Commit> for protos::cometbft::types::v1::Commit {
    fn from(value: Commit) -> Self {
        Self {
            height: value.height.into(),
            round: value.round.into(),
            block_id: Some(value.block_id.into()),
            signatures: value.signatures.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Commit> for protos::tendermint::types::Commit {
    fn from(value: Commit) -> Self {
        Self {
            height: value.height.into(),
            round: value.round.into(),
            block_id: Some(value.block_id.into()),
            signatures: value.signatures.into_iter().map(Into::into).collect(),
        }
    }
}

#[cfg(feature = "ethabi")]
#[derive(Debug, Clone, PartialEq)]
pub enum TryFromEthAbiCommitError {
    Height(crate::bounded::BoundedIntError<i64>),
    Round(crate::bounded::BoundedIntError<i32>),
    BlockId(TryFromEthAbiBlockIdError),
    Signatures(TryFromEthAbiCommitSigError),
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::glue::TendermintTypesCommitData> for Commit {
    type Error = TryFromEthAbiCommitError;

    fn try_from(value: contracts::glue::TendermintTypesCommitData) -> Result<Self, Self::Error> {
        Ok(Self {
            height: value
                .height
                .try_into()
                .map_err(TryFromEthAbiCommitError::Height)?,
            round: value
                .round
                .try_into()
                .map_err(TryFromEthAbiCommitError::Round)?,
            block_id: value
                .block_id
                .try_into()
                .map_err(TryFromEthAbiCommitError::BlockId)?,
            signatures: value
                .signatures
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()
                .map_err(TryFromEthAbiCommitError::Signatures)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromCommitError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid height")]
    Height(#[source] BoundedIntError<i64>),
    #[error("invalid round")]
    Round(#[source] BoundedIntError<i32>),
    #[error("invalid block id")]
    BlockId(#[source] TryFromBlockIdError),
    #[error("invalid signatures")]
    Signatures(#[source] TryFromCommitSigError),
}

impl TryFrom<protos::cometbft::types::v1::Commit> for Commit {
    type Error = TryFromCommitError;

    fn try_from(value: protos::cometbft::types::v1::Commit) -> Result<Self, Self::Error> {
        Ok(Self {
            height: value
                .height
                .try_into()
                .map_err(TryFromCommitError::Height)?,
            round: value.round.try_into().map_err(TryFromCommitError::Round)?,
            block_id: required!(value.block_id)?
                .try_into()
                .map_err(TryFromCommitError::BlockId)?,
            signatures: value
                .signatures
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()
                .map_err(TryFromCommitError::Signatures)?,
        })
    }
}

impl TryFrom<protos::tendermint::types::Commit> for Commit {
    type Error = TryFromCommitError;

    fn try_from(value: protos::tendermint::types::Commit) -> Result<Self, Self::Error> {
        Ok(Self {
            height: value
                .height
                .try_into()
                .map_err(TryFromCommitError::Height)?,
            round: value.round.try_into().map_err(TryFromCommitError::Round)?,
            block_id: required!(value.block_id)?
                .try_into()
                .map_err(TryFromCommitError::BlockId)?,
            signatures: value
                .signatures
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()
                .map_err(TryFromCommitError::Signatures)?,
        })
    }
}

#[cfg(feature = "ethabi")]
impl From<Commit> for contracts::glue::TendermintTypesCommitData {
    fn from(value: Commit) -> Self {
        Self {
            height: value.height.into(),
            round: value.round.into(),
            block_id: value.block_id.into(),
            signatures: value.signatures.into_iter().map(Into::into).collect(),
        }
    }
}
