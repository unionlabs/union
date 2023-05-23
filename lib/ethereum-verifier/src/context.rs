use super::SyncCommittee;
use ethereum_consensus::primitives::{Epoch, Slot, Version};

pub trait LightClientContext {
    fn finalized_slot(&self) -> Slot;

    fn current_sync_committee(&self) -> Option<&SyncCommittee>;

    fn next_sync_committee(&self) -> Option<&SyncCommittee>;
    // # Max number of active participants in a sync committee (used to calculate safety threshold)
    // previous_max_active_participants: uint64
    // current_max_active_participants: uint64
    fn fork_parameters(&self) -> &ForkParameters;
}

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ForkParameters {
    pub genesis_fork_version: Version,
    pub genesis_slot: Slot,

    pub altair_fork_version: Version,
    pub altair_fork_epoch: Epoch,

    pub bellatrix_fork_version: Version,
    pub bellatrix_fork_epoch: Epoch,

    pub capella_fork_version: Version,
    pub capella_fork_epoch: Epoch,

    pub eip4844_fork_version: Version,
    pub eip4844_fork_epoch: Epoch,
}
