use crate::capella::{self, SyncCommittee};
use crate::primitives::{Epoch, Slot, Version};

pub trait LightClientContext {
    type Config: ChainConfig;

    fn finalized_slot(&self) -> Slot;

    fn current_sync_committee(
        &self,
    ) -> Option<&SyncCommittee<{ Self::Config::SYNC_COMMITTEE_SIZE }>>;

    fn next_sync_committee(&self) -> Option<&SyncCommittee<{ Self::Config::SYNC_COMMITTEE_SIZE }>>;
    // # Max number of active participants in a sync committee (used to calculate safety threshold)
    // previous_max_active_participants: uint64
    // current_max_active_participants: uint64
    fn fork_parameters(&self) -> &ForkParameters;
}

pub trait ChainConfig {
    const SECONDS_PER_SLOT: u64;
    const SLOTS_PER_EPOCH: u64;
    const SYNC_COMMITTEE_SIZE: usize;
    const EPOCHS_PER_SYNC_COMMITTEE_PERIOD: Epoch;
    const BYTES_PER_LOGS_BLOOM: usize;
    const MAX_EXTRA_DATA_BYTES: usize;
    const MIN_SYNC_COMMITTEE_PARTICIPANTS: usize;
    const GENESIS_FORK_VERSION: Version;
}

pub struct MinimalConfig;

impl ChainConfig for MinimalConfig {
    const SECONDS_PER_SLOT: u64 = capella::minimal::SECONDS_PER_SLOT;

    const SLOTS_PER_EPOCH: u64 = capella::minimal::SLOTS_PER_EPOCH;

    const SYNC_COMMITTEE_SIZE: usize = capella::minimal::SYNC_COMMITTEE_SIZE;

    const EPOCHS_PER_SYNC_COMMITTEE_PERIOD: Epoch =
        capella::minimal::EPOCHS_PER_SYNC_COMMITTEE_PERIOD;

    const BYTES_PER_LOGS_BLOOM: usize = capella::minimal::BYTES_PER_LOGS_BLOOM;

    const MAX_EXTRA_DATA_BYTES: usize = capella::minimal::MAX_EXTRA_DATA_BYTES;

    const MIN_SYNC_COMMITTEE_PARTICIPANTS: usize =
        capella::minimal::MIN_SYNC_COMMITTEE_PARTICIPANTS;

    const GENESIS_FORK_VERSION: Version = capella::minimal::GENESIS_FORK_VERSION;
}

pub struct MainnetConfig;

impl ChainConfig for MainnetConfig {
    const SECONDS_PER_SLOT: u64 = capella::mainnet::SECONDS_PER_SLOT;

    const SLOTS_PER_EPOCH: u64 = capella::mainnet::SLOTS_PER_EPOCH;

    const SYNC_COMMITTEE_SIZE: usize = capella::mainnet::SYNC_COMMITTEE_SIZE;

    const EPOCHS_PER_SYNC_COMMITTEE_PERIOD: Epoch =
        capella::mainnet::EPOCHS_PER_SYNC_COMMITTEE_PERIOD;

    const BYTES_PER_LOGS_BLOOM: usize = capella::mainnet::BYTES_PER_LOGS_BLOOM;

    const MAX_EXTRA_DATA_BYTES: usize = capella::mainnet::MAX_EXTRA_DATA_BYTES;

    const MIN_SYNC_COMMITTEE_PARTICIPANTS: usize =
        capella::mainnet::MIN_SYNC_COMMITTEE_PARTICIPANTS;

    const GENESIS_FORK_VERSION: Version = capella::mainnet::GENESIS_FORK_VERSION;
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
