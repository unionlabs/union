pub mod mainnet;
pub mod minimal;

use crate::{
    beacon::{Epoch, Slot},
    types::U64,
};

#[allow(non_snake_case)]
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Preset {
    /// Misc
    /// ---------------------------------------------------------------
    pub DEPOSIT_CONTRACT_TREE_DEPTH: usize,
    pub MAX_VALIDATORS_PER_COMMITTEE: usize,

    /// Time parameters
    /// ---------------------------------------------------------------
    pub SECONDS_PER_SLOT: Slot,
    pub SLOTS_PER_EPOCH: Slot,

    /// Max operations per block
    /// ---------------------------------------------------------------
    pub MAX_PROPOSER_SLASHINGS: usize,
    pub MAX_ATTESTER_SLASHINGS: usize,
    pub MAX_ATTESTATIONS: usize,
    pub MAX_DEPOSITS: usize,
    pub MAX_VOLUNTARY_EXITS: usize,
    pub MAX_BLS_TO_EXECUTION_CHANGES: usize,

    /// Execution
    /// ---------------------------------------------------------------
    pub MAX_BYTES_PER_TRANSACTION: usize,
    pub MAX_TRANSACTIONS_PER_PAYLOAD: usize,
    pub BYTES_PER_LOGS_BLOOM: usize,
    pub MAX_EXTRA_DATA_BYTES: usize,
    pub MAX_WITHDRAWALS_PER_PAYLOAD: usize,

    /// Sync committee
    /// ---------------------------------------------------------------
    pub SYNC_COMMITTEE_SIZE: usize,
    pub EPOCHS_PER_SYNC_COMMITTEE_PERIOD: Epoch,

    /// Sync protocol
    /// ---------------------------------------------------------------
    pub MIN_SYNC_COMMITTEE_PARTICIPANTS: usize,
    pub UPDATE_TIMEOUT: U64,
}
