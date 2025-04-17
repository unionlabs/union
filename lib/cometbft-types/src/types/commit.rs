use serde::{Deserialize, Serialize};
use unionlabs::bounded::{BoundedI32, BoundedI64};

use crate::types::{
    block_id::BlockId,
    commit_sig::{CommitSig, CommitSigRaw},
};

/// Commit of a block for a CometBFT chain.
///
/// This is generic over the underlying `CommitSig`, to allow for threading commit sigs of chains that have deviated from CometBFT (eg. berachain). It defaults to the standard [`CommitSig`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Commit {
    #[serde(with = "::serde_utils::string")]
    pub height: BoundedI64<0, { i64::MAX }>,
    pub round: BoundedI32<0, { i32::MAX }>,
    pub block_id: BlockId,
    pub signatures: Vec<CommitSigRaw>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{bounded::BoundedIntError, errors::MissingField, required};

    use crate::types::{block_id, commit::Commit, commit_sig};

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

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid height")]
        Height(#[source] BoundedIntError<i64>),
        #[error("invalid round")]
        Round(#[source] BoundedIntError<i32>),
        #[error("invalid block id")]
        BlockId(#[source] block_id::proto::Error),
        #[error("invalid signatures")]
        Signatures(#[source] commit_sig::Error),
    }

    impl TryFrom<protos::cometbft::types::v1::Commit> for Commit {
        type Error = Error;

        fn try_from(value: protos::cometbft::types::v1::Commit) -> Result<Self, Self::Error> {
            Ok(Self {
                height: value.height.try_into().map_err(Error::Height)?,
                round: value.round.try_into().map_err(Error::Round)?,
                block_id: required!(value.block_id)?
                    .try_into()
                    .map_err(Error::BlockId)?,
                signatures: value
                    .signatures
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Error::Signatures)?,
            })
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

    impl TryFrom<protos::tendermint::types::Commit> for Commit {
        type Error = Error;

        fn try_from(value: protos::tendermint::types::Commit) -> Result<Self, Self::Error> {
            Ok(Self {
                height: value.height.try_into().map_err(Error::Height)?,
                round: value.round.try_into().map_err(Error::Round)?,
                block_id: required!(value.block_id)?
                    .try_into()
                    .map_err(Error::BlockId)?,
                signatures: value
                    .signatures
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Error::Signatures)?,
            })
        }
    }
}
