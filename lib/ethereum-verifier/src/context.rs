use crate::capella::SyncCommittee;
use crate::primitives::{Epoch, Slot, Version};

#[cfg(not(feature = "config-minimal"))]
pub use crate::capella::mainnet::*;
#[cfg(feature = "config-minimal")]
pub use crate::capella::minimal::*;

pub trait LightClientContext {
    fn finalized_slot(&self) -> Slot;

    fn current_sync_committee(&self) -> Option<&SyncCommittee<SYNC_COMMITTEE_SIZE>>;

    fn next_sync_committee(&self) -> Option<&SyncCommittee<SYNC_COMMITTEE_SIZE>>;

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
