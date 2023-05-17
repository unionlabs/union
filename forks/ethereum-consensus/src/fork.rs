use crate::beacon::{Epoch, Slot, Version};

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
