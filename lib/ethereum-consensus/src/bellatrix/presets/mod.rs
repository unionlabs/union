pub mod mainnet;
pub mod minimal;

pub struct Preset {
    pub inactivity_penalty_quotient_bellatrix: u64,
    pub min_slashing_penalty_quotient_bellatrix: u64,
    pub proportional_slashing_multiplier_bellatrix: u64,
    pub max_bytes_per_transaction: usize,
    pub max_transactions_per_payload: usize,
    pub bytes_per_logs_bloom: usize,
    pub max_extra_data_bytes: usize,
}
