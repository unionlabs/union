#[allow(non_snake_case)]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Preset {
    /// Misc
    /// ---------------------------------------------------------------
    pub MAX_COMMITTEES_PER_SLOT: usize,
    pub DEPOSIT_CONTRACT_TREE_DEPTH: usize,
    pub MAX_VALIDATORS_PER_COMMITTEE: usize,

    /// Time parameters
    /// ---------------------------------------------------------------
    pub SECONDS_PER_SLOT: usize,
    pub SLOTS_PER_EPOCH: usize,

    /// Max operations per block
    /// ---------------------------------------------------------------
    pub MAX_PROPOSER_SLASHINGS: usize,
    pub MAX_ATTESTER_SLASHINGS: usize,
    pub MAX_ATTESTATIONS: usize,
    pub MAX_DEPOSITS: usize,
    pub MAX_VOLUNTARY_EXITS: usize,
    pub MAX_BLS_TO_EXECUTION_CHANGES: usize,
    pub MAX_BLOB_COMMITMENTS_PER_BLOCK: usize,

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
    pub EPOCHS_PER_SYNC_COMMITTEE_PERIOD: usize,

    /// Sync protocol
    /// ---------------------------------------------------------------
    pub MIN_SYNC_COMMITTEE_PARTICIPANTS: usize,
    pub UPDATE_TIMEOUT: usize,

    pub MAX_DEPOSIT_REQUESTS_PER_PAYLOAD: usize,
    pub MAX_WITHDRAWAL_REQUESTS_PER_PAYLOAD: usize,
    pub MAX_CONSOLIDATION_REQUESTS_PER_PAYLOAD: usize,
}

/// <https://github.com/ethereum/consensus-specs/blob/dev/presets/mainnet>
pub const MAINNET: Preset = Preset {
    MAX_COMMITTEES_PER_SLOT: 64,
    DEPOSIT_CONTRACT_TREE_DEPTH: 32,
    MAX_VALIDATORS_PER_COMMITTEE: 2048,

    SECONDS_PER_SLOT: 12,
    SLOTS_PER_EPOCH: 32,

    MAX_PROPOSER_SLASHINGS: 16,
    MAX_ATTESTER_SLASHINGS: 2,
    MAX_ATTESTATIONS: 128,
    MAX_DEPOSITS: 16,
    MAX_VOLUNTARY_EXITS: 16,
    MAX_BLS_TO_EXECUTION_CHANGES: 16,
    MAX_BLOB_COMMITMENTS_PER_BLOCK: 4096,
    SYNC_COMMITTEE_SIZE: 512,
    EPOCHS_PER_SYNC_COMMITTEE_PERIOD: 256,
    MIN_SYNC_COMMITTEE_PARTICIPANTS: 1,
    UPDATE_TIMEOUT: 8192,

    MAX_BYTES_PER_TRANSACTION: 1_073_741_824,
    MAX_TRANSACTIONS_PER_PAYLOAD: 1_048_576,
    BYTES_PER_LOGS_BLOOM: 256,
    MAX_EXTRA_DATA_BYTES: 32,
    MAX_WITHDRAWALS_PER_PAYLOAD: 16,

    MAX_DEPOSIT_REQUESTS_PER_PAYLOAD: 8192,
    MAX_WITHDRAWAL_REQUESTS_PER_PAYLOAD: 16,
    MAX_CONSOLIDATION_REQUESTS_PER_PAYLOAD: 2,
};

/// <https://github.com/ethereum/consensus-specs/blob/dev/presets/minimal>
pub const MINIMAL: Preset = Preset {
    MAX_COMMITTEES_PER_SLOT: 4,
    DEPOSIT_CONTRACT_TREE_DEPTH: 32,
    MAX_VALIDATORS_PER_COMMITTEE: 2048,

    SECONDS_PER_SLOT: 6,
    SLOTS_PER_EPOCH: 8,

    MAX_PROPOSER_SLASHINGS: 16,
    MAX_ATTESTER_SLASHINGS: 2,
    MAX_ATTESTATIONS: 128,
    MAX_DEPOSITS: 16,
    MAX_VOLUNTARY_EXITS: 16,
    MAX_BLS_TO_EXECUTION_CHANGES: 16,
    MAX_BLOB_COMMITMENTS_PER_BLOCK: 16,

    SYNC_COMMITTEE_SIZE: 32,
    EPOCHS_PER_SYNC_COMMITTEE_PERIOD: 8,
    MIN_SYNC_COMMITTEE_PARTICIPANTS: 1,
    UPDATE_TIMEOUT: 64,

    MAX_BYTES_PER_TRANSACTION: 1_073_741_824,
    MAX_TRANSACTIONS_PER_PAYLOAD: 1_048_576,
    BYTES_PER_LOGS_BLOOM: 256,
    MAX_EXTRA_DATA_BYTES: 32,
    MAX_WITHDRAWALS_PER_PAYLOAD: 4,

    MAX_CONSOLIDATION_REQUESTS_PER_PAYLOAD: 2,
    MAX_DEPOSIT_REQUESTS_PER_PAYLOAD: 4,
    MAX_WITHDRAWAL_REQUESTS_PER_PAYLOAD: 2,
};
