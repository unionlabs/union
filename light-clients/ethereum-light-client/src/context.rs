use crate::{
    client_state::ClientState, consensus_state::TrustedConsensusState, eth_types::SyncCommittee,
};
use ethereum_verifier::{
    primitives::Slot, ForkParameters, LightClientContext as TLightClientContext,
};

pub struct LightClientContext<'a> {
    pub client_state: &'a ClientState,
    pub trusted_consensus_state: TrustedConsensusState,
}

impl<'a> LightClientContext<'a> {
    pub fn new(
        client_state: &'a ClientState,
        trusted_consensus_state: TrustedConsensusState,
    ) -> Self {
        Self {
            client_state,
            trusted_consensus_state,
        }
    }
}

impl<'a> TLightClientContext for LightClientContext<'a> {
    fn finalized_slot(&self) -> Slot {
        self.trusted_consensus_state.state.slot
    }

    fn current_sync_committee(&self) -> Option<&SyncCommittee> {
        self.trusted_consensus_state.current_sync_committee.as_ref()
    }

    fn next_sync_committee(&self) -> Option<&SyncCommittee> {
        self.trusted_consensus_state.next_sync_committee.as_ref()
    }

    fn fork_parameters(&self) -> &ForkParameters {
        &self.client_state.fork_parameters
    }
}
