use crate::altair as spec;

use crate::primitives::{Gwei, ParticipationFlags, ValidatorIndex, GENESIS_EPOCH};
use crate::state_transition::{Context, Result};
use spec::{
    decrease_balance, get_base_reward_per_increment, get_current_epoch,
    get_eligible_validator_indices, get_flag_index_deltas, get_inactivity_penalty_deltas,
    get_next_sync_committee, get_previous_epoch, get_total_active_balance, get_total_balance,
    get_unslashed_participating_indices, increase_balance, is_in_inactivity_leak,
    process_effective_balance_updates, process_eth1_data_reset, process_historical_roots_update,
    process_randao_mixes_reset, process_registry_updates, process_slashings_reset,
    weigh_justification_and_finalization, BeaconState, PARTICIPATION_FLAG_WEIGHTS,
    TIMELY_TARGET_FLAG_INDEX,
};
use std::mem;

// Return the base reward for the validator defined by `index` with respect to the current `state`
pub fn get_base_reward<
    const SLOTS_PER_HISTORICAL_ROOT: usize,
    const HISTORICAL_ROOTS_LIMIT: usize,
    const ETH1_DATA_VOTES_BOUND: usize,
    const VALIDATOR_REGISTRY_LIMIT: usize,
    const EPOCHS_PER_HISTORICAL_VECTOR: usize,
    const EPOCHS_PER_SLASHINGS_VECTOR: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const SYNC_COMMITTEE_SIZE: usize,
>(
    state: &BeaconState<
        SLOTS_PER_HISTORICAL_ROOT,
        HISTORICAL_ROOTS_LIMIT,
        ETH1_DATA_VOTES_BOUND,
        VALIDATOR_REGISTRY_LIMIT,
        EPOCHS_PER_HISTORICAL_VECTOR,
        EPOCHS_PER_SLASHINGS_VECTOR,
        MAX_VALIDATORS_PER_COMMITTEE,
        SYNC_COMMITTEE_SIZE,
    >,
    index: ValidatorIndex,
    context: &Context,
) -> Result<Gwei> {
    let increments =
        state.validators[index].effective_balance / context.effective_balance_increment;
    Ok(increments * get_base_reward_per_increment(state, context)?)
}

pub fn process_justification_and_finalization<
    const SLOTS_PER_HISTORICAL_ROOT: usize,
    const HISTORICAL_ROOTS_LIMIT: usize,
    const ETH1_DATA_VOTES_BOUND: usize,
    const VALIDATOR_REGISTRY_LIMIT: usize,
    const EPOCHS_PER_HISTORICAL_VECTOR: usize,
    const EPOCHS_PER_SLASHINGS_VECTOR: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const SYNC_COMMITTEE_SIZE: usize,
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
    >,
    context: &Context,
) -> Result<()> {
    // Initial FFG checkpoint values have a `0x00` stub for `root`.
    // Skip FFG updates in the first two epochs to avoid corner cases that might result in modifying this stub.
    let current_epoch = get_current_epoch(state, context);
    if current_epoch <= GENESIS_EPOCH + 1 {
        return Ok(());
    }

    let previous_indices = get_unslashed_participating_indices(
        state,
        TIMELY_TARGET_FLAG_INDEX,
        get_previous_epoch(state, context),
        context,
    )?;
    let current_indices = get_unslashed_participating_indices(
        state,
        TIMELY_TARGET_FLAG_INDEX,
        current_epoch,
        context,
    )?;
    let total_active_balance = get_total_active_balance(state, context)?;
    let previous_target_balance = get_total_balance(state, &previous_indices, context)?;
    let current_target_balance = get_total_balance(state, &current_indices, context)?;
    weigh_justification_and_finalization(
        state,
        total_active_balance,
        previous_target_balance,
        current_target_balance,
        context,
    )
}

pub fn process_inactivity_updates<
    const SLOTS_PER_HISTORICAL_ROOT: usize,
    const HISTORICAL_ROOTS_LIMIT: usize,
    const ETH1_DATA_VOTES_BOUND: usize,
    const VALIDATOR_REGISTRY_LIMIT: usize,
    const EPOCHS_PER_HISTORICAL_VECTOR: usize,
    const EPOCHS_PER_SLASHINGS_VECTOR: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const SYNC_COMMITTEE_SIZE: usize,
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
    >,
    context: &Context,
) -> Result<()> {
    // Skip the genesis epoch as score updates are based on the previous epoch participation
    let current_epoch = get_current_epoch(state, context);
    if current_epoch == GENESIS_EPOCH {
        return Ok(());
    }

    let eligible_validator_indices =
        get_eligible_validator_indices(state, context).collect::<Vec<_>>();
    let unslashed_participating_indices = get_unslashed_participating_indices(
        state,
        TIMELY_TARGET_FLAG_INDEX,
        get_previous_epoch(state, context),
        context,
    )?;
    let not_is_leaking = !is_in_inactivity_leak(state, context);
    for index in eligible_validator_indices {
        // Increase the inactivity score of inactive validators
        if unslashed_participating_indices.contains(&index) {
            state.inactivity_scores[index] -= u64::min(1, state.inactivity_scores[index]);
        } else {
            state.inactivity_scores[index] += context.inactivity_score_bias;
        }
        // Decrease the inactivity score of all eligible validators during a leak-free epoch
        if not_is_leaking {
            state.inactivity_scores[index] -= u64::min(
                context.inactivity_score_recovery_rate,
                state.inactivity_scores[index],
            );
        }
    }
    Ok(())
}

pub fn process_rewards_and_penalties<
    const SLOTS_PER_HISTORICAL_ROOT: usize,
    const HISTORICAL_ROOTS_LIMIT: usize,
    const ETH1_DATA_VOTES_BOUND: usize,
    const VALIDATOR_REGISTRY_LIMIT: usize,
    const EPOCHS_PER_HISTORICAL_VECTOR: usize,
    const EPOCHS_PER_SLASHINGS_VECTOR: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const SYNC_COMMITTEE_SIZE: usize,
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
    >,
    context: &Context,
) -> Result<()> {
    // No rewards are applied at the end of `GENESIS_EPOCH` because rewards are for work done in the previous epoch
    let current_epoch = get_current_epoch(state, context);
    if current_epoch == GENESIS_EPOCH {
        return Ok(());
    }

    let mut deltas = Vec::new();
    for flag_index in 0..PARTICIPATION_FLAG_WEIGHTS.len() {
        let flag_index_delta = get_flag_index_deltas(state, flag_index, context)?;
        deltas.push(flag_index_delta);
    }
    deltas.push(get_inactivity_penalty_deltas(state, context)?);
    for (rewards, penalties) in deltas {
        for index in 0..state.validators.len() {
            increase_balance(state, index, rewards[index]);
            decrease_balance(state, index, penalties[index]);
        }
    }
    Ok(())
}

pub fn process_participation_flag_updates<
    const SLOTS_PER_HISTORICAL_ROOT: usize,
    const HISTORICAL_ROOTS_LIMIT: usize,
    const ETH1_DATA_VOTES_BOUND: usize,
    const VALIDATOR_REGISTRY_LIMIT: usize,
    const EPOCHS_PER_HISTORICAL_VECTOR: usize,
    const EPOCHS_PER_SLASHINGS_VECTOR: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const SYNC_COMMITTEE_SIZE: usize,
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
    >,
) -> Result<()> {
    let current_participation = mem::take(&mut state.current_epoch_participation);
    state.previous_epoch_participation = current_participation;
    let rotate_participation = vec![ParticipationFlags::default(); state.validators.len()];
    state.current_epoch_participation = rotate_participation
        .try_into()
        .expect("should convert from Vec to List");
    Ok(())
}

pub fn process_slashings<
    const SLOTS_PER_HISTORICAL_ROOT: usize,
    const HISTORICAL_ROOTS_LIMIT: usize,
    const ETH1_DATA_VOTES_BOUND: usize,
    const VALIDATOR_REGISTRY_LIMIT: usize,
    const EPOCHS_PER_HISTORICAL_VECTOR: usize,
    const EPOCHS_PER_SLASHINGS_VECTOR: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const SYNC_COMMITTEE_SIZE: usize,
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
    >,
    context: &Context,
) -> Result<()> {
    let epoch = get_current_epoch(state, context);
    let total_balance = get_total_active_balance(state, context)?;
    let adjusted_total_slashing_balance = Gwei::min(
        state.slashings.iter().sum::<Gwei>() * context.proportional_slashing_multiplier_altair,
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

pub fn process_sync_committee_updates<
    const SLOTS_PER_HISTORICAL_ROOT: usize,
    const HISTORICAL_ROOTS_LIMIT: usize,
    const ETH1_DATA_VOTES_BOUND: usize,
    const VALIDATOR_REGISTRY_LIMIT: usize,
    const EPOCHS_PER_HISTORICAL_VECTOR: usize,
    const EPOCHS_PER_SLASHINGS_VECTOR: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const SYNC_COMMITTEE_SIZE: usize,
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
    >,
    context: &Context,
) -> Result<()> {
    let next_epoch = get_current_epoch(state, context) + 1;
    if next_epoch % context.epochs_per_sync_committee_period == 0 {
        let next_sync_committee = get_next_sync_committee(state, context)?;
        let current_sync_committee =
            mem::replace(&mut state.next_sync_committee, next_sync_committee);
        state.current_sync_committee = current_sync_committee;
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
