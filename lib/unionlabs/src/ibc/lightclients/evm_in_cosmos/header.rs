use macros::model;

use crate::{
    errors::{required, MissingField},
    ibc::{
        core::{
            client::height::Height,
            commitment::merkle_proof::{MerkleProof, TryFromMerkleProofError},
        },
        lightclients::ethereum::{
            account_proof::{AccountProof, TryFromAccountProofError},
            consensus_state::TryFromConsensusStateError,
        },
    },
};

#[model(proto(
    raw(protos::union::ibc::lightclients::evmincosmos::v1::Header),
    into,
    from
))]
pub struct Header {
    pub l1_height: Height,
    pub l2_slot: u64,
    pub l2_consensus_state: crate::ibc::lightclients::ethereum::consensus_state::ConsensusState,
    pub l2_inclusion_proof: MerkleProof,
    pub account_proof: AccountProof,
}

impl From<Header> for protos::union::ibc::lightclients::evmincosmos::v1::Header {
    fn from(value: Header) -> Self {
        Self {
            l1_height: Some(value.l1_height.into()),
            l2_slot: value.l2_slot,
            l2_consensus_state: Some(value.l2_consensus_state.into()),
            l2_inclusion_proof: Some(value.l2_inclusion_proof.into()),
            account_proof: Some(value.account_proof.into()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromHeaderError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid l2_consensus_state")]
    L2ConsensusState(#[source] TryFromConsensusStateError),
    #[error("invalid l2_inclusion_proof")]
    L2InclusionProof(#[source] TryFromMerkleProofError),
    #[error("invalid account_proof")]
    AccountProof(#[source] TryFromAccountProofError),
}

impl TryFrom<protos::union::ibc::lightclients::evmincosmos::v1::Header> for Header {
    type Error = TryFromHeaderError;

    fn try_from(
        value: protos::union::ibc::lightclients::evmincosmos::v1::Header,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            l1_height: required!(value.l1_height)?.into(),
            l2_slot: value.l2_slot,
            l2_consensus_state: required!(value.l2_consensus_state)?
                .try_into()
                .map_err(TryFromHeaderError::L2ConsensusState)?,
            l2_inclusion_proof: required!(value.l2_inclusion_proof)?
                .try_into()
                .map_err(TryFromHeaderError::L2InclusionProof)?,
            account_proof: required!(value.account_proof)?
                .try_into()
                .map_err(TryFromHeaderError::AccountProof)?,
        })
    }
}
