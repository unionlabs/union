use crate::phase0;
use crate::phase0::presets::Preset;
use crate::primitives::{Epoch, Gwei, Slot};

pub use phase0::*;

pub const MAX_COMMITTEES_PER_SLOT: u64 = 64;
pub const TARGET_COMMITTEE_SIZE: u64 = 128;
pub const MAX_VALIDATORS_PER_COMMITTEE: usize = 2048;
pub const SHUFFLE_ROUND_COUNT: u64 = 90;
pub const HYSTERESIS_QUOTIENT: u64 = 4;
pub const HYSTERESIS_DOWNWARD_MULTIPLIER: u64 = 1;
pub const HYSTERESIS_UPWARD_MULTIPLIER: u64 = 5;
pub const MIN_DEPOSIT_AMOUNT: Gwei = 10u64.pow(9);
pub const MAX_EFFECTIVE_BALANCE: Gwei = 32 * 10u64.pow(9);
pub const EFFECTIVE_BALANCE_INCREMENT: Gwei = 10u64.pow(9);
pub const MIN_ATTESTATION_INCLUSION_DELAY: Slot = 1;
pub const SLOTS_PER_EPOCH: Slot = 32;
pub const MIN_SEED_LOOKAHEAD: Epoch = 1;
pub const MAX_SEED_LOOKAHEAD: Epoch = 4;
pub const MIN_EPOCHS_TO_INACTIVITY_PENALTY: Epoch = 4;
pub const EPOCHS_PER_ETH1_VOTING_PERIOD: Epoch = 64;
pub const SLOTS_PER_HISTORICAL_ROOT: usize = 8192;
pub const EPOCHS_PER_HISTORICAL_VECTOR: usize = 65536;
pub const EPOCHS_PER_SLASHINGS_VECTOR: usize = 8192;
pub const HISTORICAL_ROOTS_LIMIT: usize = 16_777_216;
pub const VALIDATOR_REGISTRY_LIMIT: usize = 2usize.pow(40);
pub const BASE_REWARD_FACTOR: u64 = 64;
pub const WHISTLEBLOWER_REWARD_QUOTIENT: u64 = 512;
pub const PROPOSER_REWARD_QUOTIENT: u64 = 8;
pub const INACTIVITY_PENALTY_QUOTIENT: u64 = 2u64.pow(26);
pub const MIN_SLASHING_PENALTY_QUOTIENT: u64 = 128;
pub const PROPORTIONAL_SLASHING_MULTIPLIER: u64 = 1;
pub const MAX_PROPOSER_SLASHINGS: usize = 16;
pub const MAX_ATTESTER_SLASHINGS: usize = 2;
pub const MAX_ATTESTATIONS: usize = 128;
pub const MAX_DEPOSITS: usize = 16;
pub const MAX_VOLUNTARY_EXITS: usize = 16;

pub const TARGET_AGGREGATORS_PER_COMMITTEE: usize = 16;
pub const RANDOM_SUBNETS_PER_VALIDATOR: usize = 1;
pub const EPOCHS_PER_RANDOM_SUBNET_SUBSCRIPTION: Epoch = 256;

pub const PRESET: Preset = Preset {
    max_committees_per_slot: MAX_COMMITTEES_PER_SLOT,
    target_committee_size: TARGET_COMMITTEE_SIZE,
    max_validators_per_committee: MAX_VALIDATORS_PER_COMMITTEE,
    shuffle_round_count: SHUFFLE_ROUND_COUNT,
    hysteresis_quotient: HYSTERESIS_QUOTIENT,
    hysteresis_downward_multiplier: HYSTERESIS_DOWNWARD_MULTIPLIER,
    hysteresis_upward_multiplier: HYSTERESIS_UPWARD_MULTIPLIER,
    min_deposit_amount: MIN_DEPOSIT_AMOUNT,
    max_effective_balance: MAX_EFFECTIVE_BALANCE,
    effective_balance_increment: EFFECTIVE_BALANCE_INCREMENT,
    min_attestation_inclusion_delay: MIN_ATTESTATION_INCLUSION_DELAY,
    slots_per_epoch: SLOTS_PER_EPOCH,
    min_seed_lookahead: MIN_SEED_LOOKAHEAD,
    max_seed_lookahead: MAX_SEED_LOOKAHEAD,
    min_epochs_to_inactivity_penalty: MIN_EPOCHS_TO_INACTIVITY_PENALTY,
    epochs_per_eth1_voting_period: EPOCHS_PER_ETH1_VOTING_PERIOD,
    slots_per_historical_root: SLOTS_PER_HISTORICAL_ROOT as Slot,
    epochs_per_historical_vector: EPOCHS_PER_HISTORICAL_VECTOR as Epoch,
    epochs_per_slashings_vector: EPOCHS_PER_SLASHINGS_VECTOR as Epoch,
    historical_roots_limit: HISTORICAL_ROOTS_LIMIT,
    validator_registry_limit: VALIDATOR_REGISTRY_LIMIT,
    base_reward_factor: BASE_REWARD_FACTOR,
    whistleblower_reward_quotient: WHISTLEBLOWER_REWARD_QUOTIENT,
    proposer_reward_quotient: PROPOSER_REWARD_QUOTIENT,
    inactivity_penalty_quotient: INACTIVITY_PENALTY_QUOTIENT,
    min_slashing_penalty_quotient: MIN_SLASHING_PENALTY_QUOTIENT,
    proportional_slashing_multiplier: PROPORTIONAL_SLASHING_MULTIPLIER,
    max_proposer_slashings: MAX_PROPOSER_SLASHINGS,
    max_attester_slashings: MAX_ATTESTER_SLASHINGS,
    max_attestations: MAX_ATTESTATIONS,
    max_deposits: MAX_DEPOSITS,
    max_voluntary_exits: MAX_VOLUNTARY_EXITS,
};

pub type IndexedAttestation = phase0::IndexedAttestation<MAX_VALIDATORS_PER_COMMITTEE>;
pub type PendingAttestation = phase0::PendingAttestation<MAX_VALIDATORS_PER_COMMITTEE>;
pub type AttesterSlashing = phase0::AttesterSlashing<MAX_VALIDATORS_PER_COMMITTEE>;
pub type Attestation = phase0::Attestation<MAX_VALIDATORS_PER_COMMITTEE>;

pub const ETH1_DATA_VOTES_BOUND: usize =
    phase0::get_eth1_data_votes_bound(EPOCHS_PER_ETH1_VOTING_PERIOD, SLOTS_PER_EPOCH as usize);
pub const PENDING_ATTESTATIONS_BOUND: usize =
    phase0::get_pending_attestations_bound(MAX_ATTESTATIONS, SLOTS_PER_EPOCH as usize);

pub type HistoricalBatch = phase0::HistoricalBatch<SLOTS_PER_HISTORICAL_ROOT>;

pub type BeaconState = phase0::BeaconState<
    SLOTS_PER_HISTORICAL_ROOT,
    HISTORICAL_ROOTS_LIMIT,
    ETH1_DATA_VOTES_BOUND,
    VALIDATOR_REGISTRY_LIMIT,
    EPOCHS_PER_HISTORICAL_VECTOR,
    EPOCHS_PER_SLASHINGS_VECTOR,
    MAX_VALIDATORS_PER_COMMITTEE,
    PENDING_ATTESTATIONS_BOUND,
>;

pub type BeaconBlockBody = phase0::BeaconBlockBody<
    MAX_PROPOSER_SLASHINGS,
    MAX_VALIDATORS_PER_COMMITTEE,
    MAX_ATTESTER_SLASHINGS,
    MAX_ATTESTATIONS,
    MAX_DEPOSITS,
    MAX_VOLUNTARY_EXITS,
>;

pub type BeaconBlock = phase0::BeaconBlock<
    MAX_PROPOSER_SLASHINGS,
    MAX_VALIDATORS_PER_COMMITTEE,
    MAX_ATTESTER_SLASHINGS,
    MAX_ATTESTATIONS,
    MAX_DEPOSITS,
    MAX_VOLUNTARY_EXITS,
>;

pub type SignedBeaconBlock = phase0::SignedBeaconBlock<
    MAX_PROPOSER_SLASHINGS,
    MAX_VALIDATORS_PER_COMMITTEE,
    MAX_ATTESTER_SLASHINGS,
    MAX_ATTESTATIONS,
    MAX_DEPOSITS,
    MAX_VOLUNTARY_EXITS,
>;

pub type AggregateAndProof = phase0::AggregateAndProof<MAX_VALIDATORS_PER_COMMITTEE>;
pub type SignedAggregateAndProof = phase0::SignedAggregateAndProof<MAX_VALIDATORS_PER_COMMITTEE>;
