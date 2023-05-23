// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::operations::ProposerSlashingTestCase;
use ethereum_consensus::altair::minimal as spec;
use ssz_rs::prelude::*;

#[test]
fn test_epochs_are_different() {
    let mut test_case = ProposerSlashingTestCase::<spec::BeaconState, spec::ProposerSlashing>::from("consensus-spec-tests/tests/minimal/altair/operations/proposer_slashing/pyspec_tests/epochs_are_different");

    test_case.execute(|state, operation, context| {
        spec::process_proposer_slashing(state, operation, context)
    });
}

#[test]
fn test_headers_are_same_sigs_are_different() {
    let mut test_case = ProposerSlashingTestCase::<spec::BeaconState, spec::ProposerSlashing>::from("consensus-spec-tests/tests/minimal/altair/operations/proposer_slashing/pyspec_tests/headers_are_same_sigs_are_different");

    test_case.execute(|state, operation, context| {
        spec::process_proposer_slashing(state, operation, context)
    });
}

#[test]
fn test_headers_are_same_sigs_are_same() {
    let mut test_case = ProposerSlashingTestCase::<spec::BeaconState, spec::ProposerSlashing>::from("consensus-spec-tests/tests/minimal/altair/operations/proposer_slashing/pyspec_tests/headers_are_same_sigs_are_same");

    test_case.execute(|state, operation, context| {
        spec::process_proposer_slashing(state, operation, context)
    });
}

#[test]
fn test_invalid_different_proposer_indices() {
    let mut test_case = ProposerSlashingTestCase::<spec::BeaconState, spec::ProposerSlashing>::from("consensus-spec-tests/tests/minimal/altair/operations/proposer_slashing/pyspec_tests/invalid_different_proposer_indices");

    test_case.execute(|state, operation, context| {
        spec::process_proposer_slashing(state, operation, context)
    });
}

#[test]
fn test_invalid_proposer_index() {
    let mut test_case = ProposerSlashingTestCase::<spec::BeaconState, spec::ProposerSlashing>::from("consensus-spec-tests/tests/minimal/altair/operations/proposer_slashing/pyspec_tests/invalid_proposer_index");

    test_case.execute(|state, operation, context| {
        spec::process_proposer_slashing(state, operation, context)
    });
}

#[test]
fn test_invalid_sig_1() {
    let mut test_case = ProposerSlashingTestCase::<spec::BeaconState, spec::ProposerSlashing>::from("consensus-spec-tests/tests/minimal/altair/operations/proposer_slashing/pyspec_tests/invalid_sig_1");

    test_case.execute(|state, operation, context| {
        spec::process_proposer_slashing(state, operation, context)
    });
}

#[test]
fn test_invalid_sig_1_and_2() {
    let mut test_case = ProposerSlashingTestCase::<spec::BeaconState, spec::ProposerSlashing>::from("consensus-spec-tests/tests/minimal/altair/operations/proposer_slashing/pyspec_tests/invalid_sig_1_and_2");

    test_case.execute(|state, operation, context| {
        spec::process_proposer_slashing(state, operation, context)
    });
}

#[test]
fn test_invalid_sig_1_and_2_swap() {
    let mut test_case = ProposerSlashingTestCase::<spec::BeaconState, spec::ProposerSlashing>::from("consensus-spec-tests/tests/minimal/altair/operations/proposer_slashing/pyspec_tests/invalid_sig_1_and_2_swap");

    test_case.execute(|state, operation, context| {
        spec::process_proposer_slashing(state, operation, context)
    });
}

#[test]
fn test_invalid_sig_2() {
    let mut test_case = ProposerSlashingTestCase::<spec::BeaconState, spec::ProposerSlashing>::from("consensus-spec-tests/tests/minimal/altair/operations/proposer_slashing/pyspec_tests/invalid_sig_2");

    test_case.execute(|state, operation, context| {
        spec::process_proposer_slashing(state, operation, context)
    });
}

#[test]
fn test_proposer_is_not_activated() {
    let mut test_case = ProposerSlashingTestCase::<spec::BeaconState, spec::ProposerSlashing>::from("consensus-spec-tests/tests/minimal/altair/operations/proposer_slashing/pyspec_tests/proposer_is_not_activated");

    test_case.execute(|state, operation, context| {
        spec::process_proposer_slashing(state, operation, context)
    });
}

#[test]
fn test_proposer_is_slashed() {
    let mut test_case = ProposerSlashingTestCase::<spec::BeaconState, spec::ProposerSlashing>::from("consensus-spec-tests/tests/minimal/altair/operations/proposer_slashing/pyspec_tests/proposer_is_slashed");

    test_case.execute(|state, operation, context| {
        spec::process_proposer_slashing(state, operation, context)
    });
}

#[test]
fn test_proposer_is_withdrawn() {
    let mut test_case = ProposerSlashingTestCase::<spec::BeaconState, spec::ProposerSlashing>::from("consensus-spec-tests/tests/minimal/altair/operations/proposer_slashing/pyspec_tests/proposer_is_withdrawn");

    test_case.execute(|state, operation, context| {
        spec::process_proposer_slashing(state, operation, context)
    });
}

#[test]
fn test_success() {
    let mut test_case = ProposerSlashingTestCase::<spec::BeaconState, spec::ProposerSlashing>::from("consensus-spec-tests/tests/minimal/altair/operations/proposer_slashing/pyspec_tests/success");

    test_case.execute(|state, operation, context| {
        spec::process_proposer_slashing(state, operation, context)
    });
}

#[test]
fn test_success_block_header_from_future() {
    let mut test_case = ProposerSlashingTestCase::<spec::BeaconState, spec::ProposerSlashing>::from("consensus-spec-tests/tests/minimal/altair/operations/proposer_slashing/pyspec_tests/success_block_header_from_future");

    test_case.execute(|state, operation, context| {
        spec::process_proposer_slashing(state, operation, context)
    });
}

#[test]
fn test_success_slashed_and_proposer_index_the_same() {
    let mut test_case = ProposerSlashingTestCase::<spec::BeaconState, spec::ProposerSlashing>::from("consensus-spec-tests/tests/minimal/altair/operations/proposer_slashing/pyspec_tests/success_slashed_and_proposer_index_the_same");

    test_case.execute(|state, operation, context| {
        spec::process_proposer_slashing(state, operation, context)
    });
}
