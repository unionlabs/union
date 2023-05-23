use crate::phase0 as spec;

use crate::crypto::{hash, verify_signature};
use crate::primitives::{
    BlsPublicKey, Bytes32, DomainType, Gwei, ValidatorIndex, FAR_FUTURE_EPOCH,
};
use crate::signing::compute_signing_root;
use crate::ssz::ByteVector;
use crate::state_transition::{
    invalid_header_error, invalid_operation_error, Context, InvalidAttestation,
    InvalidAttesterSlashing, InvalidBeaconBlockHeader, InvalidDeposit, InvalidOperation,
    InvalidProposerSlashing, InvalidVoluntaryExit, Result,
};
use spec::{
    compute_domain, compute_epoch_at_slot, get_beacon_committee, get_beacon_proposer_index,
    get_committee_count_per_slot, get_current_epoch, get_domain, get_indexed_attestation,
    get_previous_epoch, get_randao_mix, increase_balance, initiate_validator_exit,
    is_active_validator, is_slashable_attestation_data, is_slashable_validator,
    is_valid_indexed_attestation, slash_validator, Attestation, AttesterSlashing, BeaconBlock,
    BeaconBlockBody, BeaconBlockHeader, BeaconState, Deposit, DepositMessage, PendingAttestation,
    ProposerSlashing, SignedVoluntaryExit, Validator, DEPOSIT_CONTRACT_TREE_DEPTH,
};
use ssz_rs::prelude::*;
use std::collections::HashSet;

pub fn process_proposer_slashing<
    const SLOTS_PER_HISTORICAL_ROOT: usize,
    const HISTORICAL_ROOTS_LIMIT: usize,
    const ETH1_DATA_VOTES_BOUND: usize,
    const VALIDATOR_REGISTRY_LIMIT: usize,
    const EPOCHS_PER_HISTORICAL_VECTOR: usize,
    const EPOCHS_PER_SLASHINGS_VECTOR: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const PENDING_ATTESTATIONS_BOUND: usize,
>(
    state: &mut BeaconState<
        SLOTS_PER_HISTORICAL_ROOT,
        HISTORICAL_ROOTS_LIMIT,
        ETH1_DATA_VOTES_BOUND,
        VALIDATOR_REGISTRY_LIMIT,
        EPOCHS_PER_HISTORICAL_VECTOR,
        EPOCHS_PER_SLASHINGS_VECTOR,
        MAX_VALIDATORS_PER_COMMITTEE,
        PENDING_ATTESTATIONS_BOUND,
    >,
    proposer_slashing: &mut ProposerSlashing,
    context: &Context,
) -> Result<()> {
    let header_1 = &proposer_slashing.signed_header_1.message;
    let header_2 = &proposer_slashing.signed_header_2.message;

    if header_1.slot != header_2.slot {
        return Err(invalid_operation_error(InvalidOperation::ProposerSlashing(
            InvalidProposerSlashing::SlotMismatch(header_1.slot, header_2.slot),
        )));
    }

    if header_1.proposer_index != header_2.proposer_index {
        return Err(invalid_operation_error(InvalidOperation::ProposerSlashing(
            InvalidProposerSlashing::ProposerMismatch(
                header_1.proposer_index,
                header_2.proposer_index,
            ),
        )));
    }

    if header_1 == header_2 {
        return Err(invalid_operation_error(InvalidOperation::ProposerSlashing(
            InvalidProposerSlashing::HeadersAreEqual(header_1.clone()),
        )));
    }

    let proposer_index = header_1.proposer_index;
    let proposer = state.validators.get(proposer_index).ok_or_else(|| {
        invalid_operation_error(InvalidOperation::ProposerSlashing(
            InvalidProposerSlashing::InvalidIndex(proposer_index),
        ))
    })?;
    if !is_slashable_validator(proposer, get_current_epoch(state, context)) {
        return Err(invalid_operation_error(InvalidOperation::ProposerSlashing(
            InvalidProposerSlashing::ProposerIsNotSlashable(header_1.proposer_index),
        )));
    }

    let epoch = compute_epoch_at_slot(header_1.slot, context);
    let domain = get_domain(state, DomainType::BeaconProposer, Some(epoch), context)?;
    for signed_header in [
        &mut proposer_slashing.signed_header_1,
        &mut proposer_slashing.signed_header_2,
    ] {
        let signing_root = compute_signing_root(&mut signed_header.message, domain)?;
        let public_key = &proposer.public_key;
        if verify_signature(public_key, signing_root.as_ref(), &signed_header.signature).is_err() {
            return Err(invalid_operation_error(InvalidOperation::ProposerSlashing(
                InvalidProposerSlashing::InvalidSignature(signed_header.signature.clone()),
            )));
        }
    }

    slash_validator(state, proposer_index, None, context)
}

pub fn process_attester_slashing<
    const SLOTS_PER_HISTORICAL_ROOT: usize,
    const HISTORICAL_ROOTS_LIMIT: usize,
    const ETH1_DATA_VOTES_BOUND: usize,
    const VALIDATOR_REGISTRY_LIMIT: usize,
    const EPOCHS_PER_HISTORICAL_VECTOR: usize,
    const EPOCHS_PER_SLASHINGS_VECTOR: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const PENDING_ATTESTATIONS_BOUND: usize,
>(
    state: &mut BeaconState<
        SLOTS_PER_HISTORICAL_ROOT,
        HISTORICAL_ROOTS_LIMIT,
        ETH1_DATA_VOTES_BOUND,
        VALIDATOR_REGISTRY_LIMIT,
        EPOCHS_PER_HISTORICAL_VECTOR,
        EPOCHS_PER_SLASHINGS_VECTOR,
        MAX_VALIDATORS_PER_COMMITTEE,
        PENDING_ATTESTATIONS_BOUND,
    >,
    attester_slashing: &mut AttesterSlashing<MAX_VALIDATORS_PER_COMMITTEE>,
    context: &Context,
) -> Result<()> {
    let attestation_1 = &mut attester_slashing.attestation_1;
    let attestation_2 = &mut attester_slashing.attestation_2;

    if !is_slashable_attestation_data(&attestation_1.data, &attestation_2.data) {
        return Err(invalid_operation_error(InvalidOperation::AttesterSlashing(
            InvalidAttesterSlashing::NotSlashable(
                Box::new(attestation_1.data.clone()),
                Box::new(attestation_2.data.clone()),
            ),
        )));
    }

    is_valid_indexed_attestation(state, attestation_1, context)?;

    is_valid_indexed_attestation(state, attestation_2, context)?;

    let indices_1: HashSet<ValidatorIndex> =
        HashSet::from_iter(attestation_1.attesting_indices.iter().cloned());
    let indices_2 = HashSet::from_iter(attestation_2.attesting_indices.iter().cloned());
    let mut indices = indices_1
        .intersection(&indices_2)
        .cloned()
        .collect::<Vec<_>>();
    indices.sort_unstable();

    let mut slashed_any = false;
    let current_epoch = get_current_epoch(state, context);
    for &index in &indices {
        if is_slashable_validator(&state.validators[index], current_epoch) {
            slash_validator(state, index, None, context)?;
            slashed_any = true;
        }
    }

    if !slashed_any {
        Err(invalid_operation_error(InvalidOperation::AttesterSlashing(
            InvalidAttesterSlashing::NoSlashings(indices),
        )))
    } else {
        Ok(())
    }
}

pub fn process_attestation<
    const SLOTS_PER_HISTORICAL_ROOT: usize,
    const HISTORICAL_ROOTS_LIMIT: usize,
    const ETH1_DATA_VOTES_BOUND: usize,
    const VALIDATOR_REGISTRY_LIMIT: usize,
    const EPOCHS_PER_HISTORICAL_VECTOR: usize,
    const EPOCHS_PER_SLASHINGS_VECTOR: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const PENDING_ATTESTATIONS_BOUND: usize,
>(
    state: &mut BeaconState<
        SLOTS_PER_HISTORICAL_ROOT,
        HISTORICAL_ROOTS_LIMIT,
        ETH1_DATA_VOTES_BOUND,
        VALIDATOR_REGISTRY_LIMIT,
        EPOCHS_PER_HISTORICAL_VECTOR,
        EPOCHS_PER_SLASHINGS_VECTOR,
        MAX_VALIDATORS_PER_COMMITTEE,
        PENDING_ATTESTATIONS_BOUND,
    >,
    attestation: &Attestation<MAX_VALIDATORS_PER_COMMITTEE>,
    context: &Context,
) -> Result<()> {
    let data = &attestation.data;

    let is_previous = data.target.epoch == get_previous_epoch(state, context);
    let current_epoch = get_current_epoch(state, context);
    let is_current = data.target.epoch == current_epoch;
    let valid_target_epoch = is_previous || is_current;
    if !valid_target_epoch {
        return Err(invalid_operation_error(InvalidOperation::Attestation(
            InvalidAttestation::InvalidTargetEpoch {
                target: data.target.epoch,
                current: current_epoch,
            },
        )));
    }

    let attestation_epoch = compute_epoch_at_slot(data.slot, context);
    if data.target.epoch != attestation_epoch {
        return Err(invalid_operation_error(InvalidOperation::Attestation(
            InvalidAttestation::InvalidSlot {
                slot: data.slot,
                epoch: attestation_epoch,
                target: data.target.epoch,
            },
        )));
    }

    let attestation_has_delay = data.slot + context.min_attestation_inclusion_delay <= state.slot;
    let attestation_is_recent = state.slot <= data.slot + context.slots_per_epoch;
    let attestation_is_timely = attestation_has_delay && attestation_is_recent;
    if !attestation_is_timely {
        return Err(invalid_operation_error(InvalidOperation::Attestation(
            InvalidAttestation::NotTimely {
                state_slot: state.slot,
                attestation_slot: data.slot,
                lower_bound: data.slot + context.slots_per_epoch,
                upper_bound: data.slot + context.min_attestation_inclusion_delay,
            },
        )));
    }

    let committee_count = get_committee_count_per_slot(state, data.target.epoch, context);
    if data.index >= committee_count {
        return Err(invalid_operation_error(InvalidOperation::Attestation(
            InvalidAttestation::InvalidIndex {
                index: data.index,
                upper_bound: committee_count,
            },
        )));
    }

    let committee = get_beacon_committee(state, data.slot, data.index, context)?;

    if attestation.aggregation_bits.len() != committee.len() {
        return Err(invalid_operation_error(InvalidOperation::Attestation(
            InvalidAttestation::Bitfield {
                expected_length: committee.len(),
                length: attestation.aggregation_bits.len(),
            },
        )));
    }

    // NOTE: swap order of these wrt the spec to avoid mutation
    // to the state that would need to be undone
    is_valid_indexed_attestation(
        state,
        &mut get_indexed_attestation(state, attestation, context)?,
        context,
    )?;

    let pending_attestation = PendingAttestation {
        aggregation_bits: attestation.aggregation_bits.clone(),
        data: data.clone(),
        inclusion_delay: state.slot - data.slot,
        proposer_index: get_beacon_proposer_index(state, context)?,
    };
    if is_current {
        if data.source != state.current_justified_checkpoint {
            return Err(invalid_operation_error(InvalidOperation::Attestation(
                InvalidAttestation::InvalidSource {
                    expected: state.current_justified_checkpoint.clone(),
                    source_checkpoint: data.source.clone(),
                    current: current_epoch,
                },
            )));
        }
        state.current_epoch_attestations.push(pending_attestation);
    } else {
        if data.source != state.previous_justified_checkpoint {
            return Err(invalid_operation_error(InvalidOperation::Attestation(
                InvalidAttestation::InvalidSource {
                    expected: state.previous_justified_checkpoint.clone(),
                    source_checkpoint: data.source.clone(),
                    current: current_epoch,
                },
            )));
        }
        state.previous_epoch_attestations.push(pending_attestation);
    }

    Ok(())
}

pub fn get_validator_from_deposit(deposit: &Deposit, context: &Context) -> Validator {
    let amount = deposit.data.amount;
    let effective_balance = Gwei::min(
        amount - amount % context.effective_balance_increment,
        context.max_effective_balance,
    );

    Validator {
        public_key: deposit.data.public_key.clone(),
        withdrawal_credentials: deposit.data.withdrawal_credentials.clone(),
        effective_balance,
        activation_eligibility_epoch: FAR_FUTURE_EPOCH,
        activation_epoch: FAR_FUTURE_EPOCH,
        exit_epoch: FAR_FUTURE_EPOCH,
        withdrawable_epoch: FAR_FUTURE_EPOCH,
        ..Default::default()
    }
}

pub fn process_deposit<
    const SLOTS_PER_HISTORICAL_ROOT: usize,
    const HISTORICAL_ROOTS_LIMIT: usize,
    const ETH1_DATA_VOTES_BOUND: usize,
    const VALIDATOR_REGISTRY_LIMIT: usize,
    const EPOCHS_PER_HISTORICAL_VECTOR: usize,
    const EPOCHS_PER_SLASHINGS_VECTOR: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const PENDING_ATTESTATIONS_BOUND: usize,
>(
    state: &mut BeaconState<
        SLOTS_PER_HISTORICAL_ROOT,
        HISTORICAL_ROOTS_LIMIT,
        ETH1_DATA_VOTES_BOUND,
        VALIDATOR_REGISTRY_LIMIT,
        EPOCHS_PER_HISTORICAL_VECTOR,
        EPOCHS_PER_SLASHINGS_VECTOR,
        MAX_VALIDATORS_PER_COMMITTEE,
        PENDING_ATTESTATIONS_BOUND,
    >,
    deposit: &mut Deposit,
    context: &Context,
) -> Result<()> {
    let branch = deposit
        .proof
        .iter()
        .map(|node| Node::try_from(node.as_ref()).expect("is valid instance"))
        .collect::<Vec<_>>();
    let leaf = deposit.data.hash_tree_root()?;
    let depth = DEPOSIT_CONTRACT_TREE_DEPTH + 1;
    let index = state.eth1_deposit_index as usize;
    let root = &state.eth1_data.deposit_root;
    if !is_valid_merkle_branch(&leaf, branch.iter(), depth, index, root) {
        return Err(invalid_operation_error(InvalidOperation::Deposit(
            InvalidDeposit::InvalidProof {
                leaf,
                branch,
                depth,
                index,
                root: *root,
            },
        )));
    }

    state.eth1_deposit_index += 1;

    let public_key = &deposit.data.public_key;
    let amount = deposit.data.amount;
    let validator_public_keys: HashSet<&BlsPublicKey> =
        HashSet::from_iter(state.validators.iter().map(|v| &v.public_key));
    if !validator_public_keys.contains(public_key) {
        let mut deposit_message = DepositMessage {
            public_key: public_key.clone(),
            withdrawal_credentials: deposit.data.withdrawal_credentials.clone(),
            amount,
        };
        let domain = compute_domain(DomainType::Deposit, None, None, context)?;
        let signing_root = compute_signing_root(&mut deposit_message, domain)?;

        if verify_signature(public_key, signing_root.as_ref(), &deposit.data.signature).is_err() {
            // NOTE: explicitly return with no error and also no further mutations to `state`
            return Ok(());
        }

        state
            .validators
            .push(get_validator_from_deposit(deposit, context));
        state.balances.push(amount);
    } else {
        let index = state
            .validators
            .iter()
            .position(|v| &v.public_key == public_key)
            .unwrap();

        increase_balance(state, index, amount);
    }

    Ok(())
}

pub fn process_voluntary_exit<
    const SLOTS_PER_HISTORICAL_ROOT: usize,
    const HISTORICAL_ROOTS_LIMIT: usize,
    const ETH1_DATA_VOTES_BOUND: usize,
    const VALIDATOR_REGISTRY_LIMIT: usize,
    const EPOCHS_PER_HISTORICAL_VECTOR: usize,
    const EPOCHS_PER_SLASHINGS_VECTOR: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const PENDING_ATTESTATIONS_BOUND: usize,
>(
    state: &mut BeaconState<
        SLOTS_PER_HISTORICAL_ROOT,
        HISTORICAL_ROOTS_LIMIT,
        ETH1_DATA_VOTES_BOUND,
        VALIDATOR_REGISTRY_LIMIT,
        EPOCHS_PER_HISTORICAL_VECTOR,
        EPOCHS_PER_SLASHINGS_VECTOR,
        MAX_VALIDATORS_PER_COMMITTEE,
        PENDING_ATTESTATIONS_BOUND,
    >,
    signed_voluntary_exit: &mut SignedVoluntaryExit,
    context: &Context,
) -> Result<()> {
    let voluntary_exit = &mut signed_voluntary_exit.message;
    let validator = state
        .validators
        .get(voluntary_exit.validator_index)
        .ok_or_else(|| {
            invalid_operation_error(InvalidOperation::VoluntaryExit(
                InvalidVoluntaryExit::InvalidIndex(voluntary_exit.validator_index),
            ))
        })?;
    let current_epoch = get_current_epoch(state, context);

    if !is_active_validator(validator, current_epoch) {
        return Err(invalid_operation_error(InvalidOperation::VoluntaryExit(
            InvalidVoluntaryExit::InactiveValidator(current_epoch),
        )));
    }

    if validator.exit_epoch != FAR_FUTURE_EPOCH {
        return Err(invalid_operation_error(InvalidOperation::VoluntaryExit(
            InvalidVoluntaryExit::ValidatorAlreadyExited {
                index: voluntary_exit.validator_index,
                epoch: validator.exit_epoch,
            },
        )));
    }

    if current_epoch < voluntary_exit.epoch {
        return Err(invalid_operation_error(InvalidOperation::VoluntaryExit(
            InvalidVoluntaryExit::EarlyExit {
                current_epoch,
                exit_epoch: voluntary_exit.epoch,
            },
        )));
    }

    let minimum_time_active =
        validator.activation_eligibility_epoch + context.shard_committee_period;
    if current_epoch < minimum_time_active {
        return Err(invalid_operation_error(InvalidOperation::VoluntaryExit(
            InvalidVoluntaryExit::ValidatorIsNotActiveForLongEnough {
                current_epoch,
                minimum_time_active,
            },
        )));
    }

    let domain = get_domain(
        state,
        DomainType::VoluntaryExit,
        Some(voluntary_exit.epoch),
        context,
    )?;
    let signing_root = compute_signing_root(voluntary_exit, domain)?;

    let public_key = &validator.public_key;
    if verify_signature(
        public_key,
        signing_root.as_ref(),
        &signed_voluntary_exit.signature,
    )
    .is_err()
    {
        return Err(invalid_operation_error(InvalidOperation::VoluntaryExit(
            InvalidVoluntaryExit::InvalidSignature(signed_voluntary_exit.signature.clone()),
        )));
    }

    initiate_validator_exit(state, voluntary_exit.validator_index, context);

    Ok(())
}

pub fn process_block_header<
    const SLOTS_PER_HISTORICAL_ROOT: usize,
    const HISTORICAL_ROOTS_LIMIT: usize,
    const ETH1_DATA_VOTES_BOUND: usize,
    const VALIDATOR_REGISTRY_LIMIT: usize,
    const EPOCHS_PER_HISTORICAL_VECTOR: usize,
    const EPOCHS_PER_SLASHINGS_VECTOR: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const PENDING_ATTESTATIONS_BOUND: usize,
    const MAX_PROPOSER_SLASHINGS: usize,
    const MAX_ATTESTER_SLASHINGS: usize,
    const MAX_ATTESTATIONS: usize,
    const MAX_DEPOSITS: usize,
    const MAX_VOLUNTARY_EXITS: usize,
>(
    state: &mut BeaconState<
        SLOTS_PER_HISTORICAL_ROOT,
        HISTORICAL_ROOTS_LIMIT,
        ETH1_DATA_VOTES_BOUND,
        VALIDATOR_REGISTRY_LIMIT,
        EPOCHS_PER_HISTORICAL_VECTOR,
        EPOCHS_PER_SLASHINGS_VECTOR,
        MAX_VALIDATORS_PER_COMMITTEE,
        PENDING_ATTESTATIONS_BOUND,
    >,
    block: &mut BeaconBlock<
        MAX_PROPOSER_SLASHINGS,
        MAX_VALIDATORS_PER_COMMITTEE,
        MAX_ATTESTER_SLASHINGS,
        MAX_ATTESTATIONS,
        MAX_DEPOSITS,
        MAX_VOLUNTARY_EXITS,
    >,
    context: &Context,
) -> Result<()> {
    if block.slot != state.slot {
        return Err(invalid_header_error(
            InvalidBeaconBlockHeader::StateSlotMismatch {
                state_slot: state.slot,
                block_slot: block.slot,
            },
        ));
    }

    if block.slot <= state.latest_block_header.slot {
        return Err(invalid_header_error(
            InvalidBeaconBlockHeader::OlderThanLatestBlockHeader {
                block_slot: block.slot,
                latest_block_header_slot: state.latest_block_header.slot,
            },
        ));
    }

    let proposer_index = get_beacon_proposer_index(state, context)?;
    if block.proposer_index != proposer_index {
        return Err(invalid_header_error(
            InvalidBeaconBlockHeader::ProposerIndexMismatch {
                block_proposer_index: block.proposer_index,
                proposer_index,
            },
        ));
    }

    let expected_parent_root = state.latest_block_header.hash_tree_root()?;
    if block.parent_root != expected_parent_root {
        return Err(invalid_header_error(
            InvalidBeaconBlockHeader::ParentBlockRootMismatch {
                expected: expected_parent_root,
                provided: block.parent_root,
            },
        ));
    }

    state.latest_block_header = BeaconBlockHeader {
        slot: block.slot,
        proposer_index: block.proposer_index,
        parent_root: block.parent_root,
        body_root: block.body.hash_tree_root()?,
        ..Default::default()
    };

    let proposer = &state.validators[block.proposer_index];
    if proposer.slashed {
        return Err(invalid_header_error(
            InvalidBeaconBlockHeader::ProposerSlashed(proposer_index),
        ));
    }

    Ok(())
}

pub fn xor(a: &Bytes32, b: &Bytes32) -> Bytes32 {
    let inner = a
        .iter()
        .zip(b.iter())
        .map(|(a, b)| a ^ b)
        .collect::<Vec<_>>();
    ByteVector::<32>::try_from(inner.as_ref()).unwrap()
}

pub fn process_randao<
    const SLOTS_PER_HISTORICAL_ROOT: usize,
    const HISTORICAL_ROOTS_LIMIT: usize,
    const ETH1_DATA_VOTES_BOUND: usize,
    const VALIDATOR_REGISTRY_LIMIT: usize,
    const EPOCHS_PER_HISTORICAL_VECTOR: usize,
    const EPOCHS_PER_SLASHINGS_VECTOR: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const PENDING_ATTESTATIONS_BOUND: usize,
    const MAX_PROPOSER_SLASHINGS: usize,
    const MAX_ATTESTER_SLASHINGS: usize,
    const MAX_ATTESTATIONS: usize,
    const MAX_DEPOSITS: usize,
    const MAX_VOLUNTARY_EXITS: usize,
>(
    state: &mut BeaconState<
        SLOTS_PER_HISTORICAL_ROOT,
        HISTORICAL_ROOTS_LIMIT,
        ETH1_DATA_VOTES_BOUND,
        VALIDATOR_REGISTRY_LIMIT,
        EPOCHS_PER_HISTORICAL_VECTOR,
        EPOCHS_PER_SLASHINGS_VECTOR,
        MAX_VALIDATORS_PER_COMMITTEE,
        PENDING_ATTESTATIONS_BOUND,
    >,
    body: &BeaconBlockBody<
        MAX_PROPOSER_SLASHINGS,
        MAX_VALIDATORS_PER_COMMITTEE,
        MAX_ATTESTER_SLASHINGS,
        MAX_ATTESTATIONS,
        MAX_DEPOSITS,
        MAX_VOLUNTARY_EXITS,
    >,
    context: &Context,
) -> Result<()> {
    let mut epoch = get_current_epoch(state, context);

    let proposer_index = get_beacon_proposer_index(state, context)?;
    let proposer = &state.validators[proposer_index];

    let domain = get_domain(state, DomainType::Randao, Some(epoch), context)?;
    let signing_root = compute_signing_root(&mut epoch, domain)?;

    if verify_signature(
        &proposer.public_key,
        signing_root.as_ref(),
        &body.randao_reveal,
    )
    .is_err()
    {
        return Err(invalid_operation_error(InvalidOperation::Randao(
            body.randao_reveal.clone(),
        )));
    }

    let mix = xor(
        get_randao_mix(state, epoch),
        &hash(body.randao_reveal.as_ref()),
    );
    let mix_index = epoch % context.epochs_per_historical_vector;
    state.randao_mixes[mix_index as usize] = mix;
    Ok(())
}

pub fn process_eth1_data<
    const SLOTS_PER_HISTORICAL_ROOT: usize,
    const HISTORICAL_ROOTS_LIMIT: usize,
    const ETH1_DATA_VOTES_BOUND: usize,
    const VALIDATOR_REGISTRY_LIMIT: usize,
    const EPOCHS_PER_HISTORICAL_VECTOR: usize,
    const EPOCHS_PER_SLASHINGS_VECTOR: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const PENDING_ATTESTATIONS_BOUND: usize,
    const MAX_PROPOSER_SLASHINGS: usize,
    const MAX_ATTESTER_SLASHINGS: usize,
    const MAX_ATTESTATIONS: usize,
    const MAX_DEPOSITS: usize,
    const MAX_VOLUNTARY_EXITS: usize,
>(
    state: &mut BeaconState<
        SLOTS_PER_HISTORICAL_ROOT,
        HISTORICAL_ROOTS_LIMIT,
        ETH1_DATA_VOTES_BOUND,
        VALIDATOR_REGISTRY_LIMIT,
        EPOCHS_PER_HISTORICAL_VECTOR,
        EPOCHS_PER_SLASHINGS_VECTOR,
        MAX_VALIDATORS_PER_COMMITTEE,
        PENDING_ATTESTATIONS_BOUND,
    >,
    body: &BeaconBlockBody<
        MAX_PROPOSER_SLASHINGS,
        MAX_VALIDATORS_PER_COMMITTEE,
        MAX_ATTESTER_SLASHINGS,
        MAX_ATTESTATIONS,
        MAX_DEPOSITS,
        MAX_VOLUNTARY_EXITS,
    >,
    context: &Context,
) {
    state.eth1_data_votes.push(body.eth1_data.clone());

    let votes_count = state
        .eth1_data_votes
        .iter()
        .filter(|&vote| *vote == body.eth1_data)
        .count() as u64;

    if votes_count * 2 > context.epochs_per_eth1_voting_period * context.slots_per_epoch {
        state.eth1_data = body.eth1_data.clone();
    }
}

pub fn process_operations<
    const SLOTS_PER_HISTORICAL_ROOT: usize,
    const HISTORICAL_ROOTS_LIMIT: usize,
    const ETH1_DATA_VOTES_BOUND: usize,
    const VALIDATOR_REGISTRY_LIMIT: usize,
    const EPOCHS_PER_HISTORICAL_VECTOR: usize,
    const EPOCHS_PER_SLASHINGS_VECTOR: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const PENDING_ATTESTATIONS_BOUND: usize,
    const MAX_PROPOSER_SLASHINGS: usize,
    const MAX_ATTESTER_SLASHINGS: usize,
    const MAX_ATTESTATIONS: usize,
    const MAX_DEPOSITS: usize,
    const MAX_VOLUNTARY_EXITS: usize,
>(
    state: &mut BeaconState<
        SLOTS_PER_HISTORICAL_ROOT,
        HISTORICAL_ROOTS_LIMIT,
        ETH1_DATA_VOTES_BOUND,
        VALIDATOR_REGISTRY_LIMIT,
        EPOCHS_PER_HISTORICAL_VECTOR,
        EPOCHS_PER_SLASHINGS_VECTOR,
        MAX_VALIDATORS_PER_COMMITTEE,
        PENDING_ATTESTATIONS_BOUND,
    >,
    body: &mut BeaconBlockBody<
        MAX_PROPOSER_SLASHINGS,
        MAX_VALIDATORS_PER_COMMITTEE,
        MAX_ATTESTER_SLASHINGS,
        MAX_ATTESTATIONS,
        MAX_DEPOSITS,
        MAX_VOLUNTARY_EXITS,
    >,
    context: &Context,
) -> Result<()> {
    let expected_deposit_count = usize::min(
        context.max_deposits,
        (state.eth1_data.deposit_count - state.eth1_deposit_index) as usize,
    );

    if body.deposits.len() != expected_deposit_count {
        return Err(invalid_operation_error(InvalidOperation::Deposit(
            InvalidDeposit::IncorrectCount {
                expected: expected_deposit_count,
                count: body.deposits.len(),
            },
        )));
    }

    body.proposer_slashings
        .iter_mut()
        .try_for_each(|op| process_proposer_slashing(state, op, context))?;
    body.attester_slashings
        .iter_mut()
        .try_for_each(|op| process_attester_slashing(state, op, context))?;
    body.attestations
        .iter()
        .try_for_each(|op| process_attestation(state, op, context))?;
    body.deposits
        .iter_mut()
        .try_for_each(|op| process_deposit(state, op, context))?;
    body.voluntary_exits
        .iter_mut()
        .try_for_each(|op| process_voluntary_exit(state, op, context))?;
    Ok(())
}

pub fn process_block<
    const SLOTS_PER_HISTORICAL_ROOT: usize,
    const HISTORICAL_ROOTS_LIMIT: usize,
    const ETH1_DATA_VOTES_BOUND: usize,
    const VALIDATOR_REGISTRY_LIMIT: usize,
    const EPOCHS_PER_HISTORICAL_VECTOR: usize,
    const EPOCHS_PER_SLASHINGS_VECTOR: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const PENDING_ATTESTATIONS_BOUND: usize,
    const MAX_PROPOSER_SLASHINGS: usize,
    const MAX_ATTESTER_SLASHINGS: usize,
    const MAX_ATTESTATIONS: usize,
    const MAX_DEPOSITS: usize,
    const MAX_VOLUNTARY_EXITS: usize,
>(
    state: &mut BeaconState<
        SLOTS_PER_HISTORICAL_ROOT,
        HISTORICAL_ROOTS_LIMIT,
        ETH1_DATA_VOTES_BOUND,
        VALIDATOR_REGISTRY_LIMIT,
        EPOCHS_PER_HISTORICAL_VECTOR,
        EPOCHS_PER_SLASHINGS_VECTOR,
        MAX_VALIDATORS_PER_COMMITTEE,
        PENDING_ATTESTATIONS_BOUND,
    >,
    block: &mut BeaconBlock<
        MAX_PROPOSER_SLASHINGS,
        MAX_VALIDATORS_PER_COMMITTEE,
        MAX_ATTESTER_SLASHINGS,
        MAX_ATTESTATIONS,
        MAX_DEPOSITS,
        MAX_VOLUNTARY_EXITS,
    >,
    context: &Context,
) -> Result<()> {
    process_block_header(state, block, context)?;
    process_randao(state, &block.body, context)?;
    process_eth1_data(state, &block.body, context);
    process_operations(state, &mut block.body, context)?;
    Ok(())
}
