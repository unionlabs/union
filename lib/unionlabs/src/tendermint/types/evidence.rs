use macros::model;

use crate::{
    errors::{required, MissingField},
    tendermint::types::{
        duplicate_vote_evidence::{DuplicateVoteEvidence, TryFromDuplicateVoteEvidenceError},
        light_client_attack_evidence::{
            LightClientAttackEvidence, TryFromLightClientAttackEvidenceError,
        },
    },
};

#[model(proto(raw(protos::tendermint::types::Evidence), into, from))]
#[allow(clippy::large_enum_variant)]
pub enum Evidence {
    DuplicateVote(DuplicateVoteEvidence),
    LightClientAttack(LightClientAttackEvidence),
}

impl From<Evidence> for protos::tendermint::types::Evidence {
    fn from(value: Evidence) -> Self {
        Self {
            sum: Some(match value {
                Evidence::DuplicateVote(e) => {
                    protos::tendermint::types::evidence::Sum::DuplicateVoteEvidence(e.into())
                }
                Evidence::LightClientAttack(e) => {
                    protos::tendermint::types::evidence::Sum::LightClientAttackEvidence(e.into())
                }
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromEvidenceError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid duplicate vote evidence")]
    DuplicateVote(#[from] TryFromDuplicateVoteEvidenceError),
    #[error("invalid light client attack evidence")]
    LightClientAttack(#[from] TryFromLightClientAttackEvidenceError),
}

impl TryFrom<protos::tendermint::types::Evidence> for Evidence {
    type Error = TryFromEvidenceError;

    fn try_from(value: protos::tendermint::types::Evidence) -> Result<Self, Self::Error> {
        Ok(match required!(value.sum)? {
            protos::tendermint::types::evidence::Sum::DuplicateVoteEvidence(e) => {
                Self::DuplicateVote(e.try_into()?)
            }
            protos::tendermint::types::evidence::Sum::LightClientAttackEvidence(e) => {
                Self::LightClientAttack(e.try_into()?)
            }
        })
    }
}
