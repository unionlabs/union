use unionlabs::{
    ethereum::config::ChainSpec,
    ibc::lightclients::ethereum::{fork_parameters::ForkParameters, sync_committee::SyncCommittee},
};

pub trait LightClientContext {
    type ChainSpec: ChainSpec;

    fn finalized_slot(&self) -> u64;

    fn current_sync_committee(&self) -> Option<&SyncCommittee<Self::ChainSpec>>;

    fn next_sync_committee(&self) -> Option<&SyncCommittee<Self::ChainSpec>>;

    fn fork_parameters(&self) -> &ForkParameters;

    fn tracking_checkpoint_root_index(&self) -> u64;
}
