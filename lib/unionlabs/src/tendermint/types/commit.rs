use macros::model;

use crate::{
    bounded::{BoundedI32, BoundedI64},
    tendermint::types::{block_id::BlockId, commit_sig::CommitSig},
};

#[model(proto(raw(protos::tendermint::types::Commit), into, from))]
pub struct Commit {
    pub height: BoundedI64<0, { i64::MAX }>,
    pub round: BoundedI32<0, { i32::MAX }>,
    pub block_id: BlockId,
    pub signatures: Vec<CommitSig>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        bounded::BoundedIntError,
        errors::{required, MissingField},
        tendermint::types::{
            block_id::proto::TryFromBlockIdError, commit::Commit,
            commit_sig::proto::TryFromCommitSigError,
        },
    };

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
}
