use beacon_api_types::{
    fork::Fork, light_client_update::LightClientUpdate, BeaconBlockHeader, ForkParameters,
    PresetBaseKind,
};
use serde::{Deserialize, Serialize};
use unionlabs::{
    bls::BlsSignature,
    hash::{hash_v2::Hash, H256},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BeaconHeaderData {
    pub root: H256,
    pub canonical: bool,
    pub header: BeaconHeaderSignature,
}

// REVIEW: This is just SignedBeaconHeader?
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BeaconHeaderSignature {
    pub message: BeaconBlockHeader,
    pub signature: BlsSignature,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct LightClientUpdatesResponse(pub Vec<LightClientUpdateResponse>);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct LightClientUpdateResponse {
    pub version: String,
    pub data: LightClientUpdate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Spec {
    pub preset_base: PresetBaseKind,
    pub config_name: String,
    // TERMINAL_TOTAL_DIFFICULTY: 17000000000000000,
    // TERMINAL_BLOCK_HASH: 0x0000000000000000000000000000000000000000000000000000000000000000,
    // TERMINAL_BLOCK_HASH_ACTIVATION_EPOCH: 18446744073709551615,
    // MIN_GENESIS_ACTIVE_VALIDATOR_COUNT: 1300,
    // MIN_GENESIS_TIME: 1655647200,
    pub genesis_fork_version: Hash<4>,
    #[serde(with = "::serde_utils::string")]
    pub genesis_delay: u64,
    pub altair_fork_version: Hash<4>,
    #[serde(with = "::serde_utils::string")]
    pub altair_fork_epoch: u64,
    pub bellatrix_fork_version: Hash<4>,
    #[serde(with = "::serde_utils::string")]
    pub bellatrix_fork_epoch: u64,
    pub capella_fork_version: Hash<4>,
    #[serde(with = "::serde_utils::string")]
    pub capella_fork_epoch: u64,
    pub deneb_fork_version: Hash<4>,
    #[serde(with = "::serde_utils::string")]
    pub deneb_fork_epoch: u64,
    #[serde(with = "::serde_utils::string")]
    pub seconds_per_slot: u64,
    // SECONDS_PER_ETH1_BLOCK: 14,
    // MIN_VALIDATOR_WITHDRAWABILITY_DELAY: 256,
    // SHARD_COMMITTEE_PERIOD: 256,
    // ETH1_FOLLOW_DISTANCE: 2048,
    // INACTIVITY_SCORE_BIAS: 4,
    // INACTIVITY_SCORE_RECOVERY_RATE: 16,
    // EJECTION_BALANCE: 16000000000,
    // MIN_PER_EPOCH_CHURN_LIMIT: 4,
    // CHURN_LIMIT_QUOTIENT: 65536,
    // PROPOSER_SCORE_BOOST: 40,
    // DEPOSIT_CHAIN_ID: 11155111,
    // DEPOSIT_NETWORK_ID: 11155111,
    // DEPOSIT_CONTRACT_ADDRESS: 0x7f02c3e3c98b133055b8b348b2ac625669ed295d,
    // MAX_COMMITTEES_PER_SLOT: 64,
    // TARGET_COMMITTEE_SIZE: 128,
    // MAX_VALIDATORS_PER_COMMITTEE: 2048,
    // SHUFFLE_ROUND_COUNT: 90,
    // HYSTERESIS_QUOTIENT: 4,
    // HYSTERESIS_DOWNWARD_MULTIPLIER: 1,
    // HYSTERESIS_UPWARD_MULTIPLIER: 5,
    // MIN_DEPOSIT_AMOUNT: 1000000000,
    // MAX_EFFECTIVE_BALANCE: 32000000000,
    // EFFECTIVE_BALANCE_INCREMENT: 1000000000,
    // MIN_ATTESTATION_INCLUSION_DELAY: 1,
    #[serde(with = "::serde_utils::string")]
    pub slots_per_epoch: u64,
    // MIN_SEED_LOOKAHEAD: 1,
    // MAX_SEED_LOOKAHEAD: 4,
    // EPOCHS_PER_ETH1_VOTING_PERIOD: 64,
    // SLOTS_PER_HISTORICAL_ROOT: 8192,
    // MIN_EPOCHS_TO_INACTIVITY_PENALTY: 4,
    // EPOCHS_PER_HISTORICAL_VECTOR: 65536,
    // EPOCHS_PER_SLASHINGS_VECTOR: 8192,
    // HISTORICAL_ROOTS_LIMIT: 16777216,
    // VALIDATOR_REGISTRY_LIMIT: 1099511627776,
    // BASE_REWARD_FACTOR: 64,
    // WHISTLEBLOWER_REWARD_QUOTIENT: 512,
    // PROPOSER_REWARD_QUOTIENT: 8,
    // INACTIVITY_PENALTY_QUOTIENT: 67108864,
    // MIN_SLASHING_PENALTY_QUOTIENT: 128,
    // PROPORTIONAL_SLASHING_MULTIPLIER: 1,
    // MAX_PROPOSER_SLASHINGS: 16,
    // MAX_ATTESTER_SLASHINGS: 2,
    // MAX_ATTESTATIONS: 128,
    // MAX_DEPOSITS: 16,
    // MAX_VOLUNTARY_EXITS: 16,
    #[serde(with = "::serde_utils::string")]
    pub sync_committee_size: u64,
    #[serde(with = "::serde_utils::string")]
    pub epochs_per_sync_committee_period: u64,
    // INACTIVITY_PENALTY_QUOTIENT_ALTAIR: 50331648,
    // MIN_SLASHING_PENALTY_QUOTIENT_ALTAIR: 64,
    // PROPORTIONAL_SLASHING_MULTIPLIER_ALTAIR: 2,
    // MIN_SYNC_COMMITTEE_PARTICIPANTS: 1,
    // UPDATE_TIMEOUT: 8192,
    // INACTIVITY_PENALTY_QUOTIENT_BELLATRIX: 16777216,
    // MIN_SLASHING_PENALTY_QUOTIENT_BELLATRIX: 32,
    // PROPORTIONAL_SLASHING_MULTIPLIER_BELLATRIX: 3,
    // MAX_BYTES_PER_TRANSACTION: 1073741824,
    // MAX_TRANSACTIONS_PER_PAYLOAD: 1048576,
    // BYTES_PER_LOGS_BLOOM: 256,
    // MAX_EXTRA_DATA_BYTES: 32,
    // MAX_BLS_TO_EXECUTION_CHANGES: 16,
    // MAX_WITHDRAWALS_PER_PAYLOAD: 16,
    // MAX_VALIDATORS_PER_WITHDRAWALS_SWEEP: 16384,
    // FIELD_ELEMENTS_PER_BLOB: 4096,
    // MAX_BLOBS_PER_BLOCK: 4,
    #[serde(with = "::serde_utils::string")]
    pub genesis_slot: u64,
    // GENESIS_EPOCH: 0,
    // FAR_FUTURE_EPOCH: 18446744073709551615,
    // BASE_REWARDS_PER_EPOCH: 4,
    // DEPOSIT_CONTRACT_TREE_DEPTH: 32,
    // JUSTIFICATION_BITS_LENGTH: 4,
    // ENDIANNESS: little,
    // BLS_WITHDRAWAL_PREFIX: 0,
    // ETH1_ADDRESS_WITHDRAWAL_PREFIX: 1,
    // DOMAIN_BEACON_PROPOSER: 0x00000000,
    // DOMAIN_BEACON_ATTESTER: 0x01000000,
    // DOMAIN_RANDAO: 0x02000000,
    // DOMAIN_DEPOSIT: 0x03000000,
    // DOMAIN_VOLUNTARY_EXIT: 0x04000000,
    // DOMAIN_SELECTION_PROOF: 0x05000000,
    // DOMAIN_AGGREGATE_AND_PROOF: 0x06000000,
    // DOMAIN_APPLICATION_BUILDER: 0x00000001,
    // TARGET_AGGREGATORS_PER_COMMITTEE: 16,
    // RANDOM_SUBNETS_PER_VALIDATOR: 1,
    // EPOCHS_PER_RANDOM_SUBNET_SUBSCRIPTION: 256,
    // ATTESTATION_SUBNET_COUNT: 64,
    // TIMELY_SOURCE_FLAG_INDEX: 0,
    // TIMELY_TARGET_FLAG_INDEX: 1,
    // TIMELY_HEAD_FLAG_INDEX: 2,
    // TIMELY_SOURCE_WEIGHT: 14,
    // TIMELY_TARGET_WEIGHT: 26,
    // TIMELY_HEAD_WEIGHT: 14,
    // SYNC_REWARD_WEIGHT: 2,
    // PROPOSER_WEIGHT: 8,
    // WEIGHT_DENOMINATOR: 64,
    // DOMAIN_SYNC_COMMITTEE: 0x07000000,
    // DOMAIN_SYNC_COMMITTEE_SELECTION_PROOF: 0x08000000,
    // DOMAIN_CONTRIBUTION_AND_PROOF: 0x09000000,
    // TARGET_AGGREGATORS_PER_SYNC_SUBCOMMITTEE: 16,
    // SYNC_COMMITTEE_SUBNET_COUNT: 4,
    // DOMAIN_BLS_TO_EXECUTION_CHANGE: 0x0a000000,
    // BLOB_TX_TYPE: 3,
    // VERSIONED_HASH_VERSION_KZG: 1
}

impl Spec {
    pub fn to_fork_parameters(&self) -> ForkParameters {
        ForkParameters {
            genesis_fork_version: self.genesis_fork_version,
            genesis_slot: self.genesis_slot,
            altair: Fork {
                version: self.altair_fork_version,
                epoch: self.altair_fork_epoch,
            },
            bellatrix: Fork {
                version: self.bellatrix_fork_version,
                epoch: self.bellatrix_fork_epoch,
            },
            capella: Fork {
                version: self.capella_fork_version,
                epoch: self.capella_fork_epoch,
            },
            deneb: Fork {
                version: self.deneb_fork_version,
                epoch: self.deneb_fork_epoch,
            },
        }
    }

    pub fn period(&self) -> u64 {
        self.epochs_per_sync_committee_period * self.slots_per_epoch
    }
}
