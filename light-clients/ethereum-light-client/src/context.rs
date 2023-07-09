use ethereum_verifier::LightClientContext as TLightClientContext;
use ibc_types::{
    ethereum_consts_traits::ChainSpec,
    ibc::lightclients::ethereum::{
        client_state::ClientState, fork_parameters::ForkParameters, sync_committee::SyncCommittee,
    },
};

use crate::consensus_state::TrustedConsensusState;

pub struct LightClientContext<'a, C: ChainSpec> {
    pub client_state: &'a ClientState,
    pub trusted_consensus_state: TrustedConsensusState<C>,
}

impl<'a, C: ChainSpec> LightClientContext<'a, C> {
    pub fn new(
        client_state: &'a ClientState,
        trusted_consensus_state: TrustedConsensusState<C>,
    ) -> Self {
        Self {
            client_state,
            trusted_consensus_state,
        }
    }
}

impl<'a, C: ChainSpec> TLightClientContext for LightClientContext<'a, C> {
    type ChainSpec = C;

    fn finalized_slot(&self) -> u64 {
        self.trusted_consensus_state.state.slot
    }

    fn current_sync_committee(&self) -> Option<&SyncCommittee<C>> {
        self.trusted_consensus_state.current_sync_committee.as_ref()
    }

    fn next_sync_committee(&self) -> Option<&SyncCommittee<C>> {
        self.trusted_consensus_state.next_sync_committee.as_ref()
    }

    fn fork_parameters(&self) -> &ForkParameters {
        &self.client_state.fork_parameters
    }
}
