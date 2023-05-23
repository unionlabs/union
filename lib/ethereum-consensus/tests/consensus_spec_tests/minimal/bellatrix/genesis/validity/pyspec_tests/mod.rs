// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::genesis::ValidityTestCase;
use ethereum_consensus::bellatrix::minimal as spec;
use ssz_rs::prelude::*;

#[test]
fn test_is_valid_genesis_state_false_invalid_timestamp() {
    let mut test_case = ValidityTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/genesis/validity/pyspec_tests/is_valid_genesis_state_false_invalid_timestamp");

    test_case.execute(|state, context| spec::is_valid_genesis_state(state, context));
}

#[test]
fn test_is_valid_genesis_state_false_not_enough_validator() {
    let mut test_case = ValidityTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/genesis/validity/pyspec_tests/is_valid_genesis_state_false_not_enough_validator");

    test_case.execute(|state, context| spec::is_valid_genesis_state(state, context));
}

#[test]
fn test_is_valid_genesis_state_true() {
    let mut test_case = ValidityTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/genesis/validity/pyspec_tests/is_valid_genesis_state_true");

    test_case.execute(|state, context| spec::is_valid_genesis_state(state, context));
}

#[test]
fn test_is_valid_genesis_state_true_more_balance() {
    let mut test_case = ValidityTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/genesis/validity/pyspec_tests/is_valid_genesis_state_true_more_balance");

    test_case.execute(|state, context| spec::is_valid_genesis_state(state, context));
}

#[test]
fn test_is_valid_genesis_state_true_one_more_validator() {
    let mut test_case = ValidityTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/genesis/validity/pyspec_tests/is_valid_genesis_state_true_one_more_validator");

    test_case.execute(|state, context| spec::is_valid_genesis_state(state, context));
}
