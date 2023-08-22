use unionlabs::{
    bls::BlsPublicKey,
    ethereum_consts_traits::ChainSpec,
    ibc::lightclients::ethereum::{consensus_state::ConsensusState, sync_committee::SyncCommittee},
};

use crate::errors::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct TrustedConsensusState<C: ChainSpec> {
    pub state: ConsensusState,
    pub current_sync_committee: Option<SyncCommittee<C>>,
    pub next_sync_committee: Option<SyncCommittee<C>>,
}

impl<C: ChainSpec> TrustedConsensusState<C> {
    pub fn new(
        consensus_state: ConsensusState,
        sync_committee: SyncCommittee<C>,
        aggregate_public_key: BlsPublicKey,
        is_next: bool,
    ) -> Result<Self, Error> {
        if aggregate_public_key != sync_committee.aggregate_pubkey {
            return Err(Error::InvalidSyncCommittee);
        }

        if !is_next {
            return if sync_committee.aggregate_pubkey == consensus_state.current_sync_committee {
                Ok(Self {
                    state: consensus_state,
                    current_sync_committee: Some(sync_committee),
                    next_sync_committee: None,
                })
            } else {
                Err(Error::InvalidSyncCommittee)
            };
        }

        if let Some(next_sync_committee) = consensus_state.next_sync_committee.clone() {
            if sync_committee.aggregate_pubkey == next_sync_committee {
                Ok(Self {
                    state: consensus_state,
                    current_sync_committee: None,
                    next_sync_committee: Some(sync_committee),
                })
            } else {
                Err(Error::InvalidSyncCommittee)
            }
        } else {
            Err(Error::NoNextSyncCommittee)
        }
    }

    pub fn current_sync_committee_aggregate_key(&self) -> BlsPublicKey {
        self.state.current_sync_committee.clone()
    }
}
