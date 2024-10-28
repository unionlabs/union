use beacon_api_types::{ChainSpec, ForkParameters, SyncCommittee};

pub trait LightClientContext {
    type ChainSpec: ChainSpec;

    fn finalized_slot(&self) -> u64;

    fn current_sync_committee(&self) -> Option<&SyncCommittee>;

    fn next_sync_committee(&self) -> Option<&SyncCommittee>;

    fn fork_parameters(&self) -> &ForkParameters;
}
