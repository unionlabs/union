use serde::{Deserialize, Serialize};

use crate::tendermint::types::{block_id::BlockId, commit_sig::CommitSig};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Commit {
    pub height: u32,
    pub round: u16,
    pub block_id: BlockId,
    pub signatures: Vec<CommitSig>,
}

#[cfg(feature = "ethabi")]
impl crate::EthAbi for Commit {
    type EthAbi = contracts::glue::TendermintTypesCommitData;
}

#[cfg(feature = "ethabi")]
#[derive(Debug)]
pub enum TryFromEthAbiCommitError {
    Height(std::num::TryFromIntError),
    Round(std::num::TryFromIntError),
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
