use crate::altair::{
    add_flag, get_attestation_participation_flag_indices, get_attesting_indices,
    get_next_sync_committee, BeaconState, Fork,
};
use crate::phase0;
use crate::state_transition::{Context, Result};
use ssz_rs::prelude::*;

fn translate_participation<
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
    pending_attestations: &[phase0::PendingAttestation<MAX_VALIDATORS_PER_COMMITTEE>],
    context: &Context,
) -> Result<()> {
    for attestation in pending_attestations {
        let data = &attestation.data;
        let inclusion_delay = attestation.inclusion_delay;
        let participation_flag_indices =
            get_attestation_participation_flag_indices(state, data, inclusion_delay, context)?;

        for index in get_attesting_indices(state, data, &attestation.aggregation_bits, context)? {
            for flag_index in &participation_flag_indices {
                let flags = state.previous_epoch_participation[index];
                state.previous_epoch_participation[index] = add_flag(flags, *flag_index);
            }
        }
    }
    Ok(())
}

pub fn upgrade_to_altair<
    const SLOTS_PER_HISTORICAL_ROOT: usize,
    const HISTORICAL_ROOTS_LIMIT: usize,
    const ETH1_DATA_VOTES_BOUND: usize,
    const VALIDATOR_REGISTRY_LIMIT: usize,
    const EPOCHS_PER_HISTORICAL_VECTOR: usize,
    const EPOCHS_PER_SLASHINGS_VECTOR: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const PENDING_ATTESTATIONS_BOUND: usize,
    const SYNC_COMMITTEE_SIZE: usize,
>(
    state: &phase0::BeaconState<
        SLOTS_PER_HISTORICAL_ROOT,
        HISTORICAL_ROOTS_LIMIT,
        ETH1_DATA_VOTES_BOUND,
        VALIDATOR_REGISTRY_LIMIT,
        EPOCHS_PER_HISTORICAL_VECTOR,
        EPOCHS_PER_SLASHINGS_VECTOR,
        MAX_VALIDATORS_PER_COMMITTEE,
        PENDING_ATTESTATIONS_BOUND,
    >,
    context: &Context,
) -> Result<
    BeaconState<
        SLOTS_PER_HISTORICAL_ROOT,
        HISTORICAL_ROOTS_LIMIT,
        ETH1_DATA_VOTES_BOUND,
        VALIDATOR_REGISTRY_LIMIT,
        EPOCHS_PER_HISTORICAL_VECTOR,
        EPOCHS_PER_SLASHINGS_VECTOR,
        MAX_VALIDATORS_PER_COMMITTEE,
        SYNC_COMMITTEE_SIZE,
    >,
> {
    let epoch = phase0::get_current_epoch(state, context);
    let previous_epoch_participation = vec![0u8; state.validators.len()]
        .try_into()
        .map_err(|(_, err)| err)?;
    let current_epoch_participation = vec![0u8; state.validators.len()]
        .try_into()
        .map_err(|(_, err)| err)?;
    let inactivity_scores = vec![0u64; state.validators.len()]
        .try_into()
        .map_err(|(_, err)| err)?;
    let mut post_state = BeaconState {
        genesis_time: state.genesis_time,
        genesis_validators_root: state.genesis_validators_root,
        slot: state.slot,
        fork: Fork {
            previous_version: state.fork.current_version,
            current_version: context.altair_fork_version,
            epoch,
        },
        latest_block_header: state.latest_block_header.clone(),
        block_roots: state.block_roots.clone(),
        state_roots: state.state_roots.clone(),
        historical_roots: state.historical_roots.clone(),
        eth1_data: state.eth1_data.clone(),
        eth1_data_votes: state.eth1_data_votes.clone(),
        eth1_deposit_index: state.eth1_deposit_index,
        validators: state.validators.clone(),
        balances: state.balances.clone(),
        randao_mixes: state.randao_mixes.clone(),
        slashings: state.slashings.clone(),
        previous_epoch_participation,
        current_epoch_participation,
        justification_bits: state.justification_bits.clone(),
        previous_justified_checkpoint: state.previous_justified_checkpoint.clone(),
        current_justified_checkpoint: state.current_justified_checkpoint.clone(),
        finalized_checkpoint: state.finalized_checkpoint.clone(),
        inactivity_scores,
        current_sync_committee: Default::default(),
        next_sync_committee: Default::default(),
    };

    translate_participation(&mut post_state, &state.previous_epoch_attestations, context)?;

    let sync_committee = get_next_sync_committee(&post_state, context)?;
    post_state.current_sync_committee = sync_committee.clone();
    post_state.next_sync_committee = sync_committee;

    Ok(post_state)
}
