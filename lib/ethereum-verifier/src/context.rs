use beacon_api_types::{ChainSpec, ForkParameters, SyncCommitteeSsz};

pub trait LightClientContext {
    type ChainSpec: ChainSpec;

    fn finalized_slot(&self) -> u64;

    fn current_sync_committee(&self) -> Option<&SyncCommitteeSsz<Self::ChainSpec>>;

    fn next_sync_committee(&self) -> Option<&SyncCommitteeSsz<Self::ChainSpec>>;

    fn fork_parameters(&self) -> &ForkParameters;
}
