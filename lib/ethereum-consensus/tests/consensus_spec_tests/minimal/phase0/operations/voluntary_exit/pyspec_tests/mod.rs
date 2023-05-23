// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::operations::VoluntaryExitTestCase;
use ethereum_consensus::phase0::minimal as spec;
use ssz_rs::prelude::*;

#[test]
fn test_default_exit_epoch_subsequent_exit() {
    let mut test_case = VoluntaryExitTestCase::<spec::BeaconState, spec::SignedVoluntaryExit>::from("consensus-spec-tests/tests/minimal/phase0/operations/voluntary_exit/pyspec_tests/default_exit_epoch_subsequent_exit");

    test_case.execute(|state, operation, context| {
        spec::process_voluntary_exit(state, operation, context)
    });
}

#[test]
fn test_invalid_signature() {
    let mut test_case = VoluntaryExitTestCase::<spec::BeaconState, spec::SignedVoluntaryExit>::from("consensus-spec-tests/tests/minimal/phase0/operations/voluntary_exit/pyspec_tests/invalid_signature");

    test_case.execute(|state, operation, context| {
        spec::process_voluntary_exit(state, operation, context)
    });
}

#[test]
fn test_success() {
    let mut test_case = VoluntaryExitTestCase::<spec::BeaconState, spec::SignedVoluntaryExit>::from(
        "consensus-spec-tests/tests/minimal/phase0/operations/voluntary_exit/pyspec_tests/success",
    );

    test_case.execute(|state, operation, context| {
        spec::process_voluntary_exit(state, operation, context)
    });
}

#[test]
fn test_success_exit_queue_min_churn() {
    let mut test_case = VoluntaryExitTestCase::<spec::BeaconState, spec::SignedVoluntaryExit>::from("consensus-spec-tests/tests/minimal/phase0/operations/voluntary_exit/pyspec_tests/success_exit_queue__min_churn");

    test_case.execute(|state, operation, context| {
        spec::process_voluntary_exit(state, operation, context)
    });
}

#[test]
fn test_success_exit_queue_scaled_churn() {
    let mut test_case = VoluntaryExitTestCase::<spec::BeaconState, spec::SignedVoluntaryExit>::from("consensus-spec-tests/tests/minimal/phase0/operations/voluntary_exit/pyspec_tests/success_exit_queue__scaled_churn");

    test_case.execute(|state, operation, context| {
        spec::process_voluntary_exit(state, operation, context)
    });
}

#[test]
fn test_validator_already_exited() {
    let mut test_case = VoluntaryExitTestCase::<spec::BeaconState, spec::SignedVoluntaryExit>::from("consensus-spec-tests/tests/minimal/phase0/operations/voluntary_exit/pyspec_tests/validator_already_exited");

    test_case.execute(|state, operation, context| {
        spec::process_voluntary_exit(state, operation, context)
    });
}

#[test]
fn test_validator_exit_in_future() {
    let mut test_case = VoluntaryExitTestCase::<spec::BeaconState, spec::SignedVoluntaryExit>::from("consensus-spec-tests/tests/minimal/phase0/operations/voluntary_exit/pyspec_tests/validator_exit_in_future");

    test_case.execute(|state, operation, context| {
        spec::process_voluntary_exit(state, operation, context)
    });
}

#[test]
fn test_validator_invalid_validator_index() {
    let mut test_case = VoluntaryExitTestCase::<spec::BeaconState, spec::SignedVoluntaryExit>::from("consensus-spec-tests/tests/minimal/phase0/operations/voluntary_exit/pyspec_tests/validator_invalid_validator_index");

    test_case.execute(|state, operation, context| {
        spec::process_voluntary_exit(state, operation, context)
    });
}

#[test]
fn test_validator_not_active() {
    let mut test_case = VoluntaryExitTestCase::<spec::BeaconState, spec::SignedVoluntaryExit>::from("consensus-spec-tests/tests/minimal/phase0/operations/voluntary_exit/pyspec_tests/validator_not_active");

    test_case.execute(|state, operation, context| {
        spec::process_voluntary_exit(state, operation, context)
    });
}

#[test]
fn test_validator_not_active_long_enough() {
    let mut test_case = VoluntaryExitTestCase::<spec::BeaconState, spec::SignedVoluntaryExit>::from("consensus-spec-tests/tests/minimal/phase0/operations/voluntary_exit/pyspec_tests/validator_not_active_long_enough");

    test_case.execute(|state, operation, context| {
        spec::process_voluntary_exit(state, operation, context)
    });
}
