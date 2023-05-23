use crate::altair as spec;

use crate::crypto::{eth_aggregate_public_keys, hash};
use crate::domains::DomainType;
use crate::primitives::{BlsPublicKey, Epoch, Gwei, ParticipationFlags, ValidatorIndex};
use crate::state_transition::{
    invalid_operation_error, Context, Error, InvalidAttestation, InvalidOperation, Result,
};
use integer_sqrt::IntegerSquareRoot;
use spec::{
    compute_shuffled_index, decrease_balance, get_active_validator_indices, get_base_reward,
    get_beacon_proposer_index, get_block_root, get_block_root_at_slot, get_current_epoch,
    get_eligible_validator_indices, get_previous_epoch, get_seed, get_total_active_balance,
    get_total_balance, increase_balance, initiate_validator_exit, is_in_inactivity_leak,
    AttestationData, BeaconState, SyncCommittee, PARTICIPATION_FLAG_WEIGHTS, PROPOSER_WEIGHT,
    TIMELY_HEAD_FLAG_INDEX, TIMELY_SOURCE_FLAG_INDEX, TIMELY_TARGET_FLAG_INDEX, WEIGHT_DENOMINATOR,
};
use ssz_rs::Vector;
use std::collections::HashSet;

// Return a new ``ParticipationFlags`` adding ``flag_index`` to ``flags``
pub fn add_flag(flags: ParticipationFlags, flag_index: usize) -> ParticipationFlags {
    let flag = 2u8.pow(flag_index as u32);
    flags | flag
}

// Return whether ``flags`` has ``flag_index`` set
pub fn has_flag(flags: ParticipationFlags, flag_index: usize) -> bool {
    let flag = 2u8.pow(flag_index as u32);
    flags & flag == flag
}

// Return the sync committee indices, with possible duplicates, for the next sync committee.
pub fn get_next_sync_committee_indices<
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
    context: &Context,
) -> Result<Vec<ValidatorIndex>> {
    let epoch = get_current_epoch(state, context) + 1;
    let max_random_byte = u8::MAX as u64;
    let active_validator_indices = get_active_validator_indices(state, epoch);
    let active_validator_count = active_validator_indices.len();
    let seed = get_seed(state, epoch, DomainType::SyncCommittee, context);
    let mut i: usize = 0;
    let mut sync_committee_indices = vec![];
    let mut hash_input = [0u8; 40];
    hash_input[..32].copy_from_slice(seed.as_ref());
    while sync_committee_indices.len() < context.sync_committee_size {
        let shuffled_index = compute_shuffled_index(
            i % active_validator_count,
            active_validator_count,
            &seed,
            context,
        )?;
        let candidate_index = active_validator_indices[shuffled_index];

        let i_bytes: [u8; 8] = ((i / 32) as u64).to_le_bytes();
        hash_input[32..].copy_from_slice(&i_bytes);
        let random_byte = hash(hash_input).as_ref()[i % 32] as u64;
        let effective_balance = state.validators[candidate_index].effective_balance;

        if effective_balance * max_random_byte >= context.max_effective_balance * random_byte {
            sync_committee_indices.push(candidate_index);
        }
        i += 1;
    }
    Ok(sync_committee_indices)
}

// Return the next sync committee, with possible pubkey duplicates.
pub fn get_next_sync_committee<
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
    context: &Context,
) -> Result<SyncCommittee<SYNC_COMMITTEE_SIZE>> {
    let indices = get_next_sync_committee_indices(state, context)?;
    let public_keys = indices
        .into_iter()
        .map(|i| state.validators[i].public_key.clone())
        .collect::<Vec<_>>();
    let public_keys = Vector::<BlsPublicKey, SYNC_COMMITTEE_SIZE>::try_from(public_keys)
        .map_err(|(_, err)| err)?;
    let aggregate_public_key = eth_aggregate_public_keys(&public_keys)?;

    Ok(SyncCommittee::<SYNC_COMMITTEE_SIZE> {
        public_keys,
        aggregate_public_key,
    })
}

pub fn get_base_reward_per_increment<
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
    context: &Context,
) -> Result<Gwei> {
    Ok(
        context.effective_balance_increment * context.base_reward_factor
            / get_total_active_balance(state, context)?.integer_sqrt(),
    )
}

// Return the set of validator indices that are both active and unslashed for the given ``flag_index`` and ``epoch``
pub fn get_unslashed_participating_indices<
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
    flag_index: usize,
    epoch: Epoch,
    context: &Context,
) -> Result<HashSet<ValidatorIndex>> {
    let previous_epoch = get_previous_epoch(state, context);
    let current_epoch = get_current_epoch(state, context);
    let is_current = epoch == current_epoch;
    if previous_epoch != epoch && current_epoch != epoch {
        return Err(Error::InvalidEpoch {
            requested: epoch,
            previous: previous_epoch,
            current: current_epoch,
        });
    }

    let epoch_participation = if is_current {
        &state.current_epoch_participation
    } else {
        &state.previous_epoch_participation
    };

    Ok(get_active_validator_indices(state, epoch)
        .into_iter()
        .filter(|&i| {
            let did_participate = has_flag(epoch_participation[i], flag_index);
            let not_slashed = !state.validators[i].slashed;
            did_participate && not_slashed
        })
        .collect::<HashSet<_>>())
}

// Return the flag indices that are satisfied by an attestation.
pub fn get_attestation_participation_flag_indices<
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
    data: &AttestationData,
    inclusion_delay: u64,
    context: &Context,
) -> Result<Vec<usize>> {
    let justified_checkpoint = if data.target.epoch == get_current_epoch(state, context) {
        &state.current_justified_checkpoint
    } else {
        &state.previous_justified_checkpoint
    };

    let is_matching_source = data.source == *justified_checkpoint;
    if !is_matching_source {
        return Err(invalid_operation_error(InvalidOperation::Attestation(
            InvalidAttestation::InvalidSource {
                expected: justified_checkpoint.clone(),
                source_checkpoint: data.source.clone(),
                current: get_current_epoch(state, context),
            },
        )));
    }
    let is_matching_target = is_matching_source
        && (data.target.root == *get_block_root(state, data.target.epoch, context)?);
    let is_matching_head = is_matching_target
        && (data.beacon_block_root == *get_block_root_at_slot(state, data.slot)?);

    let mut participation_flag_indices = Vec::new();
    if is_matching_source && inclusion_delay <= context.slots_per_epoch.integer_sqrt() {
        participation_flag_indices.push(TIMELY_SOURCE_FLAG_INDEX);
    }
    if is_matching_target && inclusion_delay <= context.slots_per_epoch {
        participation_flag_indices.push(TIMELY_TARGET_FLAG_INDEX);
    }
    if is_matching_head && inclusion_delay == context.min_attestation_inclusion_delay {
        participation_flag_indices.push(TIMELY_HEAD_FLAG_INDEX);
    }

    Ok(participation_flag_indices)
}

// Return the deltas for a given ``flag_index`` by scanning through the participation flags.
pub fn get_flag_index_deltas<
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
    flag_index: usize,
    context: &Context,
) -> Result<(Vec<Gwei>, Vec<Gwei>)> {
    let validator_count = state.validators.len();
    let mut rewards = vec![0; validator_count];
    let mut penalties = vec![0; validator_count];
    let previous_epoch = get_previous_epoch(state, context);
    let unslashed_participating_indices =
        get_unslashed_participating_indices(state, flag_index, previous_epoch, context)?;
    let weight = PARTICIPATION_FLAG_WEIGHTS[flag_index];
    let unslashed_participating_balance =
        get_total_balance(state, &unslashed_participating_indices, context)?;
    let unslashed_participating_increments =
        unslashed_participating_balance / context.effective_balance_increment;
    let active_increments =
        get_total_active_balance(state, context)? / context.effective_balance_increment;
    let not_leaking = !is_in_inactivity_leak(state, context);
    for index in get_eligible_validator_indices(state, context) {
        let base_reward = get_base_reward(state, index, context)?;
        if unslashed_participating_indices.contains(&index) {
            if not_leaking {
                let reward_numerator = base_reward * weight * unslashed_participating_increments;
                rewards[index] += reward_numerator / (active_increments * WEIGHT_DENOMINATOR);
            }
        } else if flag_index != TIMELY_HEAD_FLAG_INDEX {
            penalties[index] += base_reward * weight / WEIGHT_DENOMINATOR;
        }
    }
    Ok((rewards, penalties))
}

pub fn get_inactivity_penalty_deltas<
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
    context: &Context,
) -> Result<(Vec<Gwei>, Vec<Gwei>)> {
    let validator_count = state.validators.len();
    let rewards = vec![0; validator_count];
    let mut penalties = vec![0; validator_count];
    let previous_epoch = get_previous_epoch(state, context);
    let matching_target_indices = get_unslashed_participating_indices(
        state,
        TIMELY_TARGET_FLAG_INDEX,
        previous_epoch,
        context,
    )?;
    for i in get_eligible_validator_indices(state, context) {
        if !matching_target_indices.contains(&i) {
            let penalty_numerator =
                state.validators[i].effective_balance * state.inactivity_scores[i];
            let penalty_denominator =
                context.inactivity_score_bias * context.inactivity_penalty_quotient_altair;
            penalties[i] += penalty_numerator / penalty_denominator;
        }
    }
    Ok((rewards, penalties))
}

pub fn slash_validator<
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
    slashed_index: ValidatorIndex,
    whistleblower_index: Option<ValidatorIndex>,
    context: &Context,
) -> Result<()> {
    let epoch = get_current_epoch(state, context);
    initiate_validator_exit(state, slashed_index, context);
    state.validators[slashed_index].slashed = true;
    state.validators[slashed_index].withdrawable_epoch = u64::max(
        state.validators[slashed_index].withdrawable_epoch,
        epoch + context.epochs_per_slashings_vector,
    );
    let slashings_index = epoch as usize % EPOCHS_PER_SLASHINGS_VECTOR;
    state.slashings[slashings_index] += state.validators[slashed_index].effective_balance;
    decrease_balance(
        state,
        slashed_index,
        state.validators[slashed_index].effective_balance
            / context.min_slashing_penalty_quotient_altair,
    );

    let proposer_index = get_beacon_proposer_index(state, context)?;

    let whistleblower_index = whistleblower_index.unwrap_or(proposer_index);

    let whistleblower_reward =
        state.validators[slashed_index].effective_balance / context.whistleblower_reward_quotient;
    let proposer_reward_scaling_factor = PROPOSER_WEIGHT / WEIGHT_DENOMINATOR;
    let proposer_reward = whistleblower_reward * proposer_reward_scaling_factor;
    increase_balance(state, proposer_index, proposer_reward);
    increase_balance(
        state,
        whistleblower_index,
        whistleblower_reward - proposer_reward,
    );
    Ok(())
}
