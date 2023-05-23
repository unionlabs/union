use crate::bellatrix as spec;

use crate::state_transition::{invalid_operation_error, Context, InvalidExecutionPayload, Result};
use spec::{
    compute_timestamp_at_slot, get_current_epoch, get_randao_mix, is_execution_enabled,
    is_merge_transition_complete, process_block_header, process_eth1_data, process_operations,
    process_randao, process_sync_aggregate, BeaconBlock, BeaconState, ExecutionEngine,
    ExecutionPayload, ExecutionPayloadHeader,
};
use ssz_rs::prelude::*;

pub fn process_execution_payload<
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
    E: ExecutionEngine<
        BYTES_PER_LOGS_BLOOM,
        MAX_EXTRA_DATA_BYTES,
        MAX_BYTES_PER_TRANSACTION,
        MAX_TRANSACTIONS_PER_PAYLOAD,
    >,
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
    payload: &mut ExecutionPayload<
        BYTES_PER_LOGS_BLOOM,
        MAX_EXTRA_DATA_BYTES,
        MAX_BYTES_PER_TRANSACTION,
        MAX_TRANSACTIONS_PER_PAYLOAD,
    >,
    execution_engine: E,
    context: &Context,
) -> Result<()> {
    let parent_hash_invalid =
        payload.parent_hash != state.latest_execution_payload_header.block_hash;
    if is_merge_transition_complete(state) && parent_hash_invalid {
        return Err(invalid_operation_error(
            InvalidExecutionPayload::InvalidParentHash {
                provided: payload.parent_hash.clone(),
                expected: state.latest_execution_payload_header.block_hash.clone(),
            }
            .into(),
        ));
    }

    let current_epoch = get_current_epoch(state, context);
    let randao_mix = get_randao_mix(state, current_epoch);
    if &payload.prev_randao != randao_mix {
        return Err(invalid_operation_error(
            InvalidExecutionPayload::InvalidPrevRandao {
                provided: payload.prev_randao.clone(),
                expected: randao_mix.clone(),
            }
            .into(),
        ));
    }

    let timestamp = compute_timestamp_at_slot(state, state.slot, context)?;
    if payload.timestamp != timestamp {
        return Err(invalid_operation_error(
            InvalidExecutionPayload::InvalidTimestamp {
                provided: payload.timestamp,
                expected: timestamp,
            }
            .into(),
        ));
    }

    execution_engine.notify_new_payload(payload)?;

    state.latest_execution_payload_header = ExecutionPayloadHeader {
        parent_hash: payload.parent_hash.clone(),
        fee_recipient: payload.fee_recipient.clone(),
        state_root: payload.state_root.clone(),
        receipts_root: payload.receipts_root.clone(),
        logs_bloom: payload.logs_bloom.clone(),
        prev_randao: payload.prev_randao.clone(),
        block_number: payload.block_number,
        gas_limit: payload.gas_limit,
        gas_used: payload.gas_used,
        timestamp: payload.timestamp,
        extra_data: payload.extra_data.clone(),
        base_fee_per_gas: payload.base_fee_per_gas.clone(),
        block_hash: payload.block_hash.clone(),
        transactions_root: payload.transactions.hash_tree_root()?,
    };

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
    const SYNC_COMMITTEE_SIZE: usize,
    const MAX_PROPOSER_SLASHINGS: usize,
    const MAX_ATTESTER_SLASHINGS: usize,
    const MAX_ATTESTATIONS: usize,
    const MAX_DEPOSITS: usize,
    const MAX_VOLUNTARY_EXITS: usize,
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
    const MAX_BYTES_PER_TRANSACTION: usize,
    const MAX_TRANSACTIONS_PER_PAYLOAD: usize,
    E: ExecutionEngine<
        BYTES_PER_LOGS_BLOOM,
        MAX_EXTRA_DATA_BYTES,
        MAX_BYTES_PER_TRANSACTION,
        MAX_TRANSACTIONS_PER_PAYLOAD,
    >,
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
    block: &mut BeaconBlock<
        MAX_PROPOSER_SLASHINGS,
        MAX_VALIDATORS_PER_COMMITTEE,
        MAX_ATTESTER_SLASHINGS,
        MAX_ATTESTATIONS,
        MAX_DEPOSITS,
        MAX_VOLUNTARY_EXITS,
        SYNC_COMMITTEE_SIZE,
        BYTES_PER_LOGS_BLOOM,
        MAX_EXTRA_DATA_BYTES,
        MAX_BYTES_PER_TRANSACTION,
        MAX_TRANSACTIONS_PER_PAYLOAD,
    >,
    execution_engine: E,
    context: &Context,
) -> Result<()> {
    process_block_header(state, block, context)?;
    if is_execution_enabled(state, &block.body) {
        process_execution_payload(
            state,
            &mut block.body.execution_payload,
            execution_engine,
            context,
        )?;
    }
    process_randao(state, &block.body, context)?;
    process_eth1_data(state, &block.body, context);
    process_operations(state, &mut block.body, context)?;
    process_sync_aggregate(state, &block.body.sync_aggregate, context)?;
    Ok(())
}
