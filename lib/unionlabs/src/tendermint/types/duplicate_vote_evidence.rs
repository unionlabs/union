use macros::model;

use crate::{
    errors::{required, MissingField},
    google::protobuf::timestamp::{Timestamp, TryFromTimestampError},
    tendermint::types::vote::{TryFromVoteError, Vote},
};

#[model(proto(raw(protos::tendermint::types::DuplicateVoteEvidence), from, into))]
pub struct DuplicateVoteEvidence {
    pub vote_a: Vote,
    pub vote_b: Vote,
    pub total_voting_power: i64,
    pub validator_power: i64,
    pub timestamp: Timestamp,
}

impl From<DuplicateVoteEvidence> for protos::tendermint::types::DuplicateVoteEvidence {
    fn from(value: DuplicateVoteEvidence) -> Self {
        Self {
            vote_a: Some(value.vote_a.into()),
            vote_b: Some(value.vote_b.into()),
            total_voting_power: value.total_voting_power,
            validator_power: value.validator_power,
            timestamp: Some(value.timestamp.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromDuplicateVoteEvidenceError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid vote a")]
    VoteA(#[source] TryFromVoteError),
    #[error("invalid vote b")]
    VoteB(#[source] TryFromVoteError),
    #[error("invalid timestamp")]
    Timestamp(#[from] TryFromTimestampError),
}

impl TryFrom<protos::tendermint::types::DuplicateVoteEvidence> for DuplicateVoteEvidence {
    type Error = TryFromDuplicateVoteEvidenceError;

    fn try_from(
        value: protos::tendermint::types::DuplicateVoteEvidence,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            vote_a: required!(value.vote_a)?
                .try_into()
                .map_err(TryFromDuplicateVoteEvidenceError::VoteA)?,
            vote_b: required!(value.vote_b)?
                .try_into()
                .map_err(TryFromDuplicateVoteEvidenceError::VoteB)?,
            total_voting_power: value.total_voting_power,
            validator_power: value.validator_power,
            timestamp: required!(value.timestamp)?.try_into()?,
        })
    }
}
