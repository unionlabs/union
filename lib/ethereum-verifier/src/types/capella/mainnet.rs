use crate::primitives::{Epoch, Version};

pub const SECONDS_PER_SLOT: u64 = 12;
pub const SLOTS_PER_EPOCH: u64 = 32;
pub const SYNC_COMMITTEE_SIZE: usize = 512;
pub const EPOCHS_PER_SYNC_COMMITTEE_PERIOD: Epoch = 256;
pub const BYTES_PER_LOGS_BLOOM: usize = 256;
pub const MAX_EXTRA_DATA_BYTES: usize = 32;
pub const MIN_SYNC_COMMITTEE_PARTICIPANTS: usize = 1;
pub const GENESIS_FORK_VERSION: Version = [0u8; 4];

pub type SyncCommittee = super::SyncCommittee<SYNC_COMMITTEE_SIZE>;
pub type SyncAggregate = super::SyncAggregate<SYNC_COMMITTEE_SIZE>;

pub type LightClientHeader = super::LightClientHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>;
pub type LightClientUpdate =
    super::LightClientUpdate<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>;

pub type ExecutionPayloadHeader =
    super::ExecutionPayloadHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>;
