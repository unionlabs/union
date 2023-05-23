pub mod mainnet;
pub mod minimal;

use crate::primitives::Epoch;

pub struct Preset {
    pub inactivity_penalty_quotient_altair: u64,
    pub min_slashing_penalty_quotient_altair: u64,
    pub proportional_slashing_multiplier_altair: u64,
    pub sync_committee_size: usize,
    pub epochs_per_sync_committee_period: Epoch,
    pub min_sync_committee_participants: usize,
    pub update_timeout: usize,
}
