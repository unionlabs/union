use crate::bellatrix as spec;

use crate::primitives::Gwei;
use crate::state_transition::{Context, Result};
use spec::{
    decrease_balance, get_current_epoch, get_total_active_balance,
    process_effective_balance_updates, process_eth1_data_reset, process_historical_roots_update,
    process_inactivity_updates, process_justification_and_finalization,
    process_participation_flag_updates, process_randao_mixes_reset, process_registry_updates,
    process_rewards_and_penalties, process_slashings_reset, process_sync_committee_updates,
    BeaconState,
};

pub fn process_slashings<
    const SLOTS_PER_HISTORICAL_ROOT: usize,
    const HISTORICAL_ROOTS_LIMIT: usize,
    const ETH1_DATA_VOTES_BOUND: usize,
    const VALIDATOR_REGISTRY_LIMIT: usize,
    const EPOCHS_PER_HISTORICAL_VECTOR: usize,
    const EPOCHS_PER_SLASHINGS_VECTOR: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const SYNC_COMMITTEE_SIZE: usize,
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
    const MAX_BYTES_PER_TRANSACTION: usize,
    const MAX_TRANSACTIONS_PER_PAYLOAD: usize,
>(
    state: &mut BeaconState<
        SLOTS_PER_HISTORICAL_ROOT,
        HISTORICAL_ROOTS_LIMIT,
        ETH1_DATA_VOTES_BOUND,
        VALIDATOR_REGISTRY_LIMIT,
        EPOCHS_PER_HISTORICAL_VECTOR,
        EPOCHS_PER_SLASHINGS_VECTOR,
        MAX_VALIDATORS_PER_COMMITTEE,
        SYNC_COMMITTEE_SIZE,
        BYTES_PER_LOGS_BLOOM,
        MAX_EXTRA_DATA_BYTES,
        MAX_BYTES_PER_TRANSACTION,
        MAX_TRANSACTIONS_PER_PAYLOAD,
    >,
    context: &Context,
) -> Result<()> {
    let epoch = get_current_epoch(state, context);
    let total_balance = get_total_active_balance(state, context)?;
    let adjusted_total_slashing_balance = Gwei::min(
        state.slashings.iter().sum::<Gwei>() * context.proportional_slashing_multiplier_bellatrix,
        total_balance,
    );
    for i in 0..state.validators.len() {
        let validator = &state.validators[i];
        if validator.slashed
            && (epoch + context.epochs_per_slashings_vector / 2) == validator.withdrawable_epoch
        {
            let increment = context.effective_balance_increment;
            let penalty_numerator =
                validator.effective_balance / increment * adjusted_total_slashing_balance;
            let penalty = penalty_numerator / total_balance * increment;
            decrease_balance(state, i, penalty);
        }
    }
    Ok(())
}

pub fn process_epoch<
    const SLOTS_PER_HISTORICAL_ROOT: usize,
    const HISTORICAL_ROOTS_LIMIT: usize,
    const ETH1_DATA_VOTES_BOUND: usize,
    const VALIDATOR_REGISTRY_LIMIT: usize,
    const EPOCHS_PER_HISTORICAL_VECTOR: usize,
    const EPOCHS_PER_SLASHINGS_VECTOR: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const SYNC_COMMITTEE_SIZE: usize,
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
    const MAX_BYTES_PER_TRANSACTION: usize,
    const MAX_TRANSACTIONS_PER_PAYLOAD: usize,
>(
    state: &mut BeaconState<
        SLOTS_PER_HISTORICAL_ROOT,
        HISTORICAL_ROOTS_LIMIT,
        ETH1_DATA_VOTES_BOUND,
        VALIDATOR_REGISTRY_LIMIT,
        EPOCHS_PER_HISTORICAL_VECTOR,
        EPOCHS_PER_SLASHINGS_VECTOR,
        MAX_VALIDATORS_PER_COMMITTEE,
        SYNC_COMMITTEE_SIZE,
        BYTES_PER_LOGS_BLOOM,
        MAX_EXTRA_DATA_BYTES,
        MAX_BYTES_PER_TRANSACTION,
        MAX_TRANSACTIONS_PER_PAYLOAD,
    >,
    context: &Context,
) -> Result<()> {
    process_justification_and_finalization(state, context)?;
    process_inactivity_updates(state, context)?;
    process_rewards_and_penalties(state, context)?;
    process_registry_updates(state, context);
    process_slashings(state, context)?;
    process_eth1_data_reset(state, context);
    process_effective_balance_updates(state, context);
    process_slashings_reset(state, context);
    process_randao_mixes_reset(state, context);
    process_historical_roots_update(state, context)?;
    process_participation_flag_updates(state)?;
    process_sync_committee_updates(state, context)?;
    Ok(())
}
