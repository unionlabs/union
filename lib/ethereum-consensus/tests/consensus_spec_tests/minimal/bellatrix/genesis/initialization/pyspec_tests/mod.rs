// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::genesis::InitializationTestCase;
use ethereum_consensus::bellatrix::minimal as spec;
use ssz_rs::prelude::*;

#[test]
fn test_initialize_beacon_state_from_eth_1() {
    let mut test_case = InitializationTestCase::<spec::BeaconState, spec::Deposit, spec::ExecutionPayloadHeader>::from("consensus-spec-tests/tests/minimal/bellatrix/genesis/initialization/pyspec_tests/initialize_beacon_state_from_eth1");

    test_case.execute(
        |eth1_block_hash, eth1_timestamp, deposits, execution_payload_header, context| {
            spec::initialize_beacon_state_from_eth1::<
                { spec::SLOTS_PER_HISTORICAL_ROOT },
                { spec::HISTORICAL_ROOTS_LIMIT },
                { spec::ETH1_DATA_VOTES_BOUND },
                { spec::VALIDATOR_REGISTRY_LIMIT },
                { spec::EPOCHS_PER_HISTORICAL_VECTOR },
                { spec::EPOCHS_PER_SLASHINGS_VECTOR },
                { spec::MAX_VALIDATORS_PER_COMMITTEE },
                { spec::SYNC_COMMITTEE_SIZE },
                { spec::MAX_PROPOSER_SLASHINGS },
                { spec::MAX_ATTESTER_SLASHINGS },
                { spec::MAX_ATTESTATIONS },
                { spec::MAX_DEPOSITS },
                { spec::MAX_VOLUNTARY_EXITS },
                { spec::BYTES_PER_LOGS_BLOOM },
                { spec::MAX_EXTRA_DATA_BYTES },
                { spec::MAX_BYTES_PER_TRANSACTION },
                { spec::MAX_TRANSACTIONS_PER_PAYLOAD },
            >(
                eth1_block_hash,
                eth1_timestamp,
                deposits,
                execution_payload_header,
                context,
            )
        },
    );
}

#[test]
fn test_initialize_beacon_state_one_topup_activation() {
    let mut test_case = InitializationTestCase::<spec::BeaconState, spec::Deposit, spec::ExecutionPayloadHeader>::from("consensus-spec-tests/tests/minimal/bellatrix/genesis/initialization/pyspec_tests/initialize_beacon_state_one_topup_activation");

    test_case.execute(
        |eth1_block_hash, eth1_timestamp, deposits, execution_payload_header, context| {
            spec::initialize_beacon_state_from_eth1::<
                { spec::SLOTS_PER_HISTORICAL_ROOT },
                { spec::HISTORICAL_ROOTS_LIMIT },
                { spec::ETH1_DATA_VOTES_BOUND },
                { spec::VALIDATOR_REGISTRY_LIMIT },
                { spec::EPOCHS_PER_HISTORICAL_VECTOR },
                { spec::EPOCHS_PER_SLASHINGS_VECTOR },
                { spec::MAX_VALIDATORS_PER_COMMITTEE },
                { spec::SYNC_COMMITTEE_SIZE },
                { spec::MAX_PROPOSER_SLASHINGS },
                { spec::MAX_ATTESTER_SLASHINGS },
                { spec::MAX_ATTESTATIONS },
                { spec::MAX_DEPOSITS },
                { spec::MAX_VOLUNTARY_EXITS },
                { spec::BYTES_PER_LOGS_BLOOM },
                { spec::MAX_EXTRA_DATA_BYTES },
                { spec::MAX_BYTES_PER_TRANSACTION },
                { spec::MAX_TRANSACTIONS_PER_PAYLOAD },
            >(
                eth1_block_hash,
                eth1_timestamp,
                deposits,
                execution_payload_header,
                context,
            )
        },
    );
}

#[test]
fn test_initialize_beacon_state_random_invalid_genesis() {
    let mut test_case = InitializationTestCase::<spec::BeaconState, spec::Deposit, spec::ExecutionPayloadHeader>::from("consensus-spec-tests/tests/minimal/bellatrix/genesis/initialization/pyspec_tests/initialize_beacon_state_random_invalid_genesis");

    test_case.execute(
        |eth1_block_hash, eth1_timestamp, deposits, execution_payload_header, context| {
            spec::initialize_beacon_state_from_eth1::<
                { spec::SLOTS_PER_HISTORICAL_ROOT },
                { spec::HISTORICAL_ROOTS_LIMIT },
                { spec::ETH1_DATA_VOTES_BOUND },
                { spec::VALIDATOR_REGISTRY_LIMIT },
                { spec::EPOCHS_PER_HISTORICAL_VECTOR },
                { spec::EPOCHS_PER_SLASHINGS_VECTOR },
                { spec::MAX_VALIDATORS_PER_COMMITTEE },
                { spec::SYNC_COMMITTEE_SIZE },
                { spec::MAX_PROPOSER_SLASHINGS },
                { spec::MAX_ATTESTER_SLASHINGS },
                { spec::MAX_ATTESTATIONS },
                { spec::MAX_DEPOSITS },
                { spec::MAX_VOLUNTARY_EXITS },
                { spec::BYTES_PER_LOGS_BLOOM },
                { spec::MAX_EXTRA_DATA_BYTES },
                { spec::MAX_BYTES_PER_TRANSACTION },
                { spec::MAX_TRANSACTIONS_PER_PAYLOAD },
            >(
                eth1_block_hash,
                eth1_timestamp,
                deposits,
                execution_payload_header,
                context,
            )
        },
    );
}

#[test]
fn test_initialize_beacon_state_random_valid_genesis() {
    let mut test_case = InitializationTestCase::<spec::BeaconState, spec::Deposit, spec::ExecutionPayloadHeader>::from("consensus-spec-tests/tests/minimal/bellatrix/genesis/initialization/pyspec_tests/initialize_beacon_state_random_valid_genesis");

    test_case.execute(
        |eth1_block_hash, eth1_timestamp, deposits, execution_payload_header, context| {
            spec::initialize_beacon_state_from_eth1::<
                { spec::SLOTS_PER_HISTORICAL_ROOT },
                { spec::HISTORICAL_ROOTS_LIMIT },
                { spec::ETH1_DATA_VOTES_BOUND },
                { spec::VALIDATOR_REGISTRY_LIMIT },
                { spec::EPOCHS_PER_HISTORICAL_VECTOR },
                { spec::EPOCHS_PER_SLASHINGS_VECTOR },
                { spec::MAX_VALIDATORS_PER_COMMITTEE },
                { spec::SYNC_COMMITTEE_SIZE },
                { spec::MAX_PROPOSER_SLASHINGS },
                { spec::MAX_ATTESTER_SLASHINGS },
                { spec::MAX_ATTESTATIONS },
                { spec::MAX_DEPOSITS },
                { spec::MAX_VOLUNTARY_EXITS },
                { spec::BYTES_PER_LOGS_BLOOM },
                { spec::MAX_EXTRA_DATA_BYTES },
                { spec::MAX_BYTES_PER_TRANSACTION },
                { spec::MAX_TRANSACTIONS_PER_PAYLOAD },
            >(
                eth1_block_hash,
                eth1_timestamp,
                deposits,
                execution_payload_header,
                context,
            )
        },
    );
}

#[test]
fn test_initialize_beacon_state_some_small_balances() {
    let mut test_case = InitializationTestCase::<spec::BeaconState, spec::Deposit, spec::ExecutionPayloadHeader>::from("consensus-spec-tests/tests/minimal/bellatrix/genesis/initialization/pyspec_tests/initialize_beacon_state_some_small_balances");

    test_case.execute(
        |eth1_block_hash, eth1_timestamp, deposits, execution_payload_header, context| {
            spec::initialize_beacon_state_from_eth1::<
                { spec::SLOTS_PER_HISTORICAL_ROOT },
                { spec::HISTORICAL_ROOTS_LIMIT },
                { spec::ETH1_DATA_VOTES_BOUND },
                { spec::VALIDATOR_REGISTRY_LIMIT },
                { spec::EPOCHS_PER_HISTORICAL_VECTOR },
                { spec::EPOCHS_PER_SLASHINGS_VECTOR },
                { spec::MAX_VALIDATORS_PER_COMMITTEE },
                { spec::SYNC_COMMITTEE_SIZE },
                { spec::MAX_PROPOSER_SLASHINGS },
                { spec::MAX_ATTESTER_SLASHINGS },
                { spec::MAX_ATTESTATIONS },
                { spec::MAX_DEPOSITS },
                { spec::MAX_VOLUNTARY_EXITS },
                { spec::BYTES_PER_LOGS_BLOOM },
                { spec::MAX_EXTRA_DATA_BYTES },
                { spec::MAX_BYTES_PER_TRANSACTION },
                { spec::MAX_TRANSACTIONS_PER_PAYLOAD },
            >(
                eth1_block_hash,
                eth1_timestamp,
                deposits,
                execution_payload_header,
                context,
            )
        },
    );
}

#[test]
fn test_initialize_post_transition() {
    let mut test_case = InitializationTestCase::<spec::BeaconState, spec::Deposit, spec::ExecutionPayloadHeader>::from("consensus-spec-tests/tests/minimal/bellatrix/genesis/initialization/pyspec_tests/initialize_post_transition");

    test_case.execute(
        |eth1_block_hash, eth1_timestamp, deposits, execution_payload_header, context| {
            spec::initialize_beacon_state_from_eth1::<
                { spec::SLOTS_PER_HISTORICAL_ROOT },
                { spec::HISTORICAL_ROOTS_LIMIT },
                { spec::ETH1_DATA_VOTES_BOUND },
                { spec::VALIDATOR_REGISTRY_LIMIT },
                { spec::EPOCHS_PER_HISTORICAL_VECTOR },
                { spec::EPOCHS_PER_SLASHINGS_VECTOR },
                { spec::MAX_VALIDATORS_PER_COMMITTEE },
                { spec::SYNC_COMMITTEE_SIZE },
                { spec::MAX_PROPOSER_SLASHINGS },
                { spec::MAX_ATTESTER_SLASHINGS },
                { spec::MAX_ATTESTATIONS },
                { spec::MAX_DEPOSITS },
                { spec::MAX_VOLUNTARY_EXITS },
                { spec::BYTES_PER_LOGS_BLOOM },
                { spec::MAX_EXTRA_DATA_BYTES },
                { spec::MAX_BYTES_PER_TRANSACTION },
                { spec::MAX_TRANSACTIONS_PER_PAYLOAD },
            >(
                eth1_block_hash,
                eth1_timestamp,
                deposits,
                execution_payload_header,
                context,
            )
        },
    );
}

#[test]
fn test_initialize_pre_transition_empty_payload() {
    let mut test_case = InitializationTestCase::<spec::BeaconState, spec::Deposit, spec::ExecutionPayloadHeader>::from("consensus-spec-tests/tests/minimal/bellatrix/genesis/initialization/pyspec_tests/initialize_pre_transition_empty_payload");

    test_case.execute(
        |eth1_block_hash, eth1_timestamp, deposits, execution_payload_header, context| {
            spec::initialize_beacon_state_from_eth1::<
                { spec::SLOTS_PER_HISTORICAL_ROOT },
                { spec::HISTORICAL_ROOTS_LIMIT },
                { spec::ETH1_DATA_VOTES_BOUND },
                { spec::VALIDATOR_REGISTRY_LIMIT },
                { spec::EPOCHS_PER_HISTORICAL_VECTOR },
                { spec::EPOCHS_PER_SLASHINGS_VECTOR },
                { spec::MAX_VALIDATORS_PER_COMMITTEE },
                { spec::SYNC_COMMITTEE_SIZE },
                { spec::MAX_PROPOSER_SLASHINGS },
                { spec::MAX_ATTESTER_SLASHINGS },
                { spec::MAX_ATTESTATIONS },
                { spec::MAX_DEPOSITS },
                { spec::MAX_VOLUNTARY_EXITS },
                { spec::BYTES_PER_LOGS_BLOOM },
                { spec::MAX_EXTRA_DATA_BYTES },
                { spec::MAX_BYTES_PER_TRANSACTION },
                { spec::MAX_TRANSACTIONS_PER_PAYLOAD },
            >(
                eth1_block_hash,
                eth1_timestamp,
                deposits,
                execution_payload_header,
                context,
            )
        },
    );
}

#[test]
fn test_initialize_pre_transition_no_param() {
    let mut test_case = InitializationTestCase::<spec::BeaconState, spec::Deposit, spec::ExecutionPayloadHeader>::from("consensus-spec-tests/tests/minimal/bellatrix/genesis/initialization/pyspec_tests/initialize_pre_transition_no_param");

    test_case.execute(
        |eth1_block_hash, eth1_timestamp, deposits, execution_payload_header, context| {
            spec::initialize_beacon_state_from_eth1::<
                { spec::SLOTS_PER_HISTORICAL_ROOT },
                { spec::HISTORICAL_ROOTS_LIMIT },
                { spec::ETH1_DATA_VOTES_BOUND },
                { spec::VALIDATOR_REGISTRY_LIMIT },
                { spec::EPOCHS_PER_HISTORICAL_VECTOR },
                { spec::EPOCHS_PER_SLASHINGS_VECTOR },
                { spec::MAX_VALIDATORS_PER_COMMITTEE },
                { spec::SYNC_COMMITTEE_SIZE },
                { spec::MAX_PROPOSER_SLASHINGS },
                { spec::MAX_ATTESTER_SLASHINGS },
                { spec::MAX_ATTESTATIONS },
                { spec::MAX_DEPOSITS },
                { spec::MAX_VOLUNTARY_EXITS },
                { spec::BYTES_PER_LOGS_BLOOM },
                { spec::MAX_EXTRA_DATA_BYTES },
                { spec::MAX_BYTES_PER_TRANSACTION },
                { spec::MAX_TRANSACTIONS_PER_PAYLOAD },
            >(
                eth1_block_hash,
                eth1_timestamp,
                deposits,
                execution_payload_header,
                context,
            )
        },
    );
}
