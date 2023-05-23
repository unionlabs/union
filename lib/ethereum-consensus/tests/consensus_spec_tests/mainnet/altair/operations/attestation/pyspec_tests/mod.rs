// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::operations::AttestationTestCase;
use ethereum_consensus::altair::mainnet as spec;
use ssz_rs::prelude::*;

#[test]
fn test_after_epoch_slots() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/after_epoch_slots");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_bad_source_root() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/bad_source_root");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_before_inclusion_delay() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/before_inclusion_delay");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_correct_after_epoch_delay() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/correct_after_epoch_delay");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_correct_epoch_delay() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/correct_epoch_delay");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_correct_min_inclusion_delay() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/correct_min_inclusion_delay");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_correct_sqrt_epoch_delay() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/correct_sqrt_epoch_delay");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_empty_participants_seemingly_valid_sig() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/empty_participants_seemingly_valid_sig");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_empty_participants_zeroes_sig() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/empty_participants_zeroes_sig");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_future_target_epoch() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/future_target_epoch");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_incorrect_head_after_epoch_delay() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/incorrect_head_after_epoch_delay");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_incorrect_head_and_target_after_epoch_delay() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/incorrect_head_and_target_after_epoch_delay");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_incorrect_head_and_target_epoch_delay() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/incorrect_head_and_target_epoch_delay");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_incorrect_head_and_target_min_inclusion_delay() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/incorrect_head_and_target_min_inclusion_delay");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_incorrect_head_and_target_sqrt_epoch_delay() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/incorrect_head_and_target_sqrt_epoch_delay");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_incorrect_head_epoch_delay() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/incorrect_head_epoch_delay");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_incorrect_head_min_inclusion_delay() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/incorrect_head_min_inclusion_delay");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_incorrect_head_sqrt_epoch_delay() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/incorrect_head_sqrt_epoch_delay");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_incorrect_target_after_epoch_delay() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/incorrect_target_after_epoch_delay");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_incorrect_target_epoch_delay() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/incorrect_target_epoch_delay");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_incorrect_target_min_inclusion_delay() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/incorrect_target_min_inclusion_delay");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_incorrect_target_sqrt_epoch_delay() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/incorrect_target_sqrt_epoch_delay");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_invalid_attestation_signature() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/invalid_attestation_signature");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_invalid_current_source_root() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/invalid_current_source_root");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_invalid_index() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/invalid_index");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_invalid_previous_source_root() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/invalid_previous_source_root");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_mismatched_target_and_slot() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/mismatched_target_and_slot");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_new_source_epoch() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/new_source_epoch");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_old_source_epoch() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/old_source_epoch");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_old_target_epoch() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/old_target_epoch");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_source_root_is_target_root() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/source_root_is_target_root");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_success() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from(
        "consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/success",
    );

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_success_multi_proposer_index_iterations() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/success_multi_proposer_index_iterations");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_success_previous_epoch() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/success_previous_epoch");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_too_few_aggregation_bits() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/too_few_aggregation_bits");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_too_many_aggregation_bits() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/too_many_aggregation_bits");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_wrong_index_for_committee_signature() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/wrong_index_for_committee_signature");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_wrong_index_for_slot_0() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/wrong_index_for_slot_0");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}

#[test]
fn test_wrong_index_for_slot_1() {
    let mut test_case = AttestationTestCase::<spec::BeaconState, spec::Attestation>::from("consensus-spec-tests/tests/mainnet/altair/operations/attestation/pyspec_tests/wrong_index_for_slot_1");

    test_case
        .execute(|state, operation, context| spec::process_attestation(state, operation, context));
}
