// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::operations::DepositTestCase;
use ethereum_consensus::bellatrix::mainnet as spec;
use ssz_rs::prelude::*;

#[test]
fn test_bad_merkle_proof() {
    let mut test_case = DepositTestCase::<spec::BeaconState, spec::Deposit>::from("consensus-spec-tests/tests/mainnet/bellatrix/operations/deposit/pyspec_tests/bad_merkle_proof");

    test_case.execute(|state, operation, context| spec::process_deposit(state, operation, context));
}

#[test]
fn test_invalid_sig_new_deposit() {
    let mut test_case = DepositTestCase::<spec::BeaconState, spec::Deposit>::from("consensus-spec-tests/tests/mainnet/bellatrix/operations/deposit/pyspec_tests/invalid_sig_new_deposit");

    test_case.execute(|state, operation, context| spec::process_deposit(state, operation, context));
}

#[test]
fn test_invalid_sig_other_version() {
    let mut test_case = DepositTestCase::<spec::BeaconState, spec::Deposit>::from("consensus-spec-tests/tests/mainnet/bellatrix/operations/deposit/pyspec_tests/invalid_sig_other_version");

    test_case.execute(|state, operation, context| spec::process_deposit(state, operation, context));
}

#[test]
fn test_invalid_sig_top_up() {
    let mut test_case = DepositTestCase::<spec::BeaconState, spec::Deposit>::from("consensus-spec-tests/tests/mainnet/bellatrix/operations/deposit/pyspec_tests/invalid_sig_top_up");

    test_case.execute(|state, operation, context| spec::process_deposit(state, operation, context));
}

#[test]
fn test_invalid_withdrawal_credentials_top_up() {
    let mut test_case = DepositTestCase::<spec::BeaconState, spec::Deposit>::from("consensus-spec-tests/tests/mainnet/bellatrix/operations/deposit/pyspec_tests/invalid_withdrawal_credentials_top_up");

    test_case.execute(|state, operation, context| spec::process_deposit(state, operation, context));
}

#[test]
fn test_new_deposit_eth_1_withdrawal_credentials() {
    let mut test_case = DepositTestCase::<spec::BeaconState, spec::Deposit>::from("consensus-spec-tests/tests/mainnet/bellatrix/operations/deposit/pyspec_tests/new_deposit_eth1_withdrawal_credentials");

    test_case.execute(|state, operation, context| spec::process_deposit(state, operation, context));
}

#[test]
fn test_new_deposit_max() {
    let mut test_case = DepositTestCase::<spec::BeaconState, spec::Deposit>::from("consensus-spec-tests/tests/mainnet/bellatrix/operations/deposit/pyspec_tests/new_deposit_max");

    test_case.execute(|state, operation, context| spec::process_deposit(state, operation, context));
}

#[test]
fn test_new_deposit_non_versioned_withdrawal_credentials() {
    let mut test_case = DepositTestCase::<spec::BeaconState, spec::Deposit>::from("consensus-spec-tests/tests/mainnet/bellatrix/operations/deposit/pyspec_tests/new_deposit_non_versioned_withdrawal_credentials");

    test_case.execute(|state, operation, context| spec::process_deposit(state, operation, context));
}

#[test]
fn test_new_deposit_over_max() {
    let mut test_case = DepositTestCase::<spec::BeaconState, spec::Deposit>::from("consensus-spec-tests/tests/mainnet/bellatrix/operations/deposit/pyspec_tests/new_deposit_over_max");

    test_case.execute(|state, operation, context| spec::process_deposit(state, operation, context));
}

#[test]
fn test_new_deposit_under_max() {
    let mut test_case = DepositTestCase::<spec::BeaconState, spec::Deposit>::from("consensus-spec-tests/tests/mainnet/bellatrix/operations/deposit/pyspec_tests/new_deposit_under_max");

    test_case.execute(|state, operation, context| spec::process_deposit(state, operation, context));
}

#[test]
fn test_success_top_up() {
    let mut test_case = DepositTestCase::<spec::BeaconState, spec::Deposit>::from("consensus-spec-tests/tests/mainnet/bellatrix/operations/deposit/pyspec_tests/success_top_up");

    test_case.execute(|state, operation, context| spec::process_deposit(state, operation, context));
}

#[test]
fn test_valid_sig_but_forked_state() {
    let mut test_case = DepositTestCase::<spec::BeaconState, spec::Deposit>::from("consensus-spec-tests/tests/mainnet/bellatrix/operations/deposit/pyspec_tests/valid_sig_but_forked_state");

    test_case.execute(|state, operation, context| spec::process_deposit(state, operation, context));
}

#[test]
fn test_wrong_deposit_for_deposit_count() {
    let mut test_case = DepositTestCase::<spec::BeaconState, spec::Deposit>::from("consensus-spec-tests/tests/mainnet/bellatrix/operations/deposit/pyspec_tests/wrong_deposit_for_deposit_count");

    test_case.execute(|state, operation, context| spec::process_deposit(state, operation, context));
}
