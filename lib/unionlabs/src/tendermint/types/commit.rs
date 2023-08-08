use serde::{Deserialize, Serialize};

use crate::{
    bounded_int::{BoundedI32, BoundedI64},
    errors::MissingField,
    tendermint::types::{block_id::BlockId, commit_sig::CommitSig},
    Proto, TryFromProtoErrorOf, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Commit {
    pub height: BoundedI64<0, { i64::MAX }>,
    pub round: BoundedI32<0, { i32::MAX }>,
    pub block_id: BlockId,
    pub signatures: Vec<CommitSig>,
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
impl crate::EthAbi for Commit {
    type EthAbi = contracts::glue::TendermintTypesCommitData;
}

#[cfg(feature = "ethabi")]
#[derive(Debug)]
pub enum TryFromEthAbiCommitError {
    Height(crate::bounded_int::BoundedIntError<i64>),
    Round(crate::bounded_int::BoundedIntError<i32>),
    BlockId(crate::TryFromEthAbiErrorOf<BlockId>),
    Signatures(crate::TryFromEthAbiErrorOf<CommitSig>),
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

#[derive(Debug)]
pub enum TryFromCommitError {
    MissingField(MissingField),
    BlockId(TryFromProtoErrorOf<BlockId>),
    IntegerOverflow(&'static str),
    Signatures,
}

impl TryFrom<protos::tendermint::types::Commit> for Commit {
    type Error = TryFromCommitError;

    fn try_from(value: protos::tendermint::types::Commit) -> Result<Self, Self::Error> {
        Ok(Self {
            height: value
                .height
                .try_into()
                .map_err(|_| TryFromCommitError::IntegerOverflow("height"))?,
            round: value
                .round
                .try_into()
                .map_err(|_| TryFromCommitError::IntegerOverflow("round"))?,
            block_id: value
                .block_id
                .ok_or(TryFromCommitError::MissingField(MissingField("block_id")))?
                .try_into()
                .map_err(TryFromCommitError::BlockId)?,
            signatures: value
                .signatures
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| TryFromCommitError::Signatures)?,
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

impl TypeUrl for protos::tendermint::types::Commit {
    const TYPE_URL: &'static str = "/tendermint.types.Commit";
}

impl Proto for Commit {
    type Proto = protos::tendermint::types::Commit;
}
