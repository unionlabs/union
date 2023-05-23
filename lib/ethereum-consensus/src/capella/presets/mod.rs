pub mod mainnet;
pub mod minimal;

pub struct Preset {
    pub max_bls_to_execution_changes: usize,
    pub max_withdrawals_per_payload: usize,
    pub max_validators_per_withdrawals_sweep: usize,
}
