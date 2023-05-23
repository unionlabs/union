pub mod mainnet;
pub mod minimal;

use crate::primitives::{Epoch, Gwei, Slot};

pub struct Preset {
    pub max_committees_per_slot: u64,
    pub target_committee_size: u64,
    pub max_validators_per_committee: usize,
    pub shuffle_round_count: u64,
    pub hysteresis_quotient: u64,
    pub hysteresis_downward_multiplier: u64,
    pub hysteresis_upward_multiplier: u64,
    pub min_deposit_amount: Gwei,
    pub max_effective_balance: Gwei,
    pub effective_balance_increment: Gwei,
    pub min_attestation_inclusion_delay: Slot,
    pub slots_per_epoch: Slot,
    pub min_seed_lookahead: Epoch,
    pub max_seed_lookahead: Epoch,
    pub min_epochs_to_inactivity_penalty: Epoch,
    pub epochs_per_eth1_voting_period: Epoch,
    pub slots_per_historical_root: Slot,
    pub epochs_per_historical_vector: Epoch,
    pub epochs_per_slashings_vector: Epoch,
    pub historical_roots_limit: usize,
    pub validator_registry_limit: usize,
    pub base_reward_factor: u64,
    pub whistleblower_reward_quotient: u64,
    pub proposer_reward_quotient: u64,
    pub inactivity_penalty_quotient: u64,
    pub min_slashing_penalty_quotient: u64,
    pub proportional_slashing_multiplier: u64,
    pub max_proposer_slashings: usize,
    pub max_attester_slashings: usize,
    pub max_attestations: usize,
    pub max_deposits: usize,
    pub max_voluntary_exits: usize,
}
