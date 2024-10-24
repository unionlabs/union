use serde::{Deserialize, Serialize};
use unionlabs::google::protobuf::timestamp::Timestamp;

use crate::types::vote::Vote;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DuplicateVoteEvidence {
    pub vote_a: Vote,
    pub vote_b: Vote,
    pub total_voting_power: i64,
    pub validator_power: i64,
    pub timestamp: Timestamp,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{
        errors::MissingField, google::protobuf::timestamp::TryFromTimestampError, required,
    };

    use crate::types::{duplicate_vote_evidence::DuplicateVoteEvidence, vote};

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
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid vote a")]
        VoteA(#[source] vote::proto::Error),
        #[error("invalid vote b")]
        VoteB(#[source] vote::proto::Error),
        #[error("invalid timestamp")]
        Timestamp(#[from] TryFromTimestampError),
    }

    impl TryFrom<protos::tendermint::types::DuplicateVoteEvidence> for DuplicateVoteEvidence {
        type Error = Error;

        fn try_from(
            value: protos::tendermint::types::DuplicateVoteEvidence,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                vote_a: required!(value.vote_a)?.try_into().map_err(Error::VoteA)?,
                vote_b: required!(value.vote_b)?.try_into().map_err(Error::VoteB)?,
                total_voting_power: value.total_voting_power,
                validator_power: value.validator_power,
                timestamp: required!(value.timestamp)?.try_into()?,
            })
        }
    }
}
