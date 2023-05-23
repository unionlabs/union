// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::fork::ForkTestCase;
use ethereum_consensus::altair::mainnet as altair;
use ethereum_consensus::bellatrix::mainnet as spec;
use ssz_rs::prelude::*;

#[test]
fn test_bellatrix_fork_random_0() {
    let  test_case = ForkTestCase::<>::from("consensus-spec-tests/tests/mainnet/bellatrix/fork/fork/pyspec_tests/bellatrix_fork_random_0");

    test_case.execute(
        |state: &altair::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_bellatrix(state, context)
        },
    );
}

#[test]
fn test_bellatrix_fork_random_1() {
    let  test_case = ForkTestCase::<>::from("consensus-spec-tests/tests/mainnet/bellatrix/fork/fork/pyspec_tests/bellatrix_fork_random_1");

    test_case.execute(
        |state: &altair::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_bellatrix(state, context)
        },
    );
}

#[test]
fn test_bellatrix_fork_random_2() {
    let  test_case = ForkTestCase::<>::from("consensus-spec-tests/tests/mainnet/bellatrix/fork/fork/pyspec_tests/bellatrix_fork_random_2");

    test_case.execute(
        |state: &altair::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_bellatrix(state, context)
        },
    );
}

#[test]
fn test_bellatrix_fork_random_3() {
    let  test_case = ForkTestCase::<>::from("consensus-spec-tests/tests/mainnet/bellatrix/fork/fork/pyspec_tests/bellatrix_fork_random_3");

    test_case.execute(
        |state: &altair::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_bellatrix(state, context)
        },
    );
}

#[test]
fn test_bellatrix_fork_random_low_balances() {
    let  test_case = ForkTestCase::<>::from("consensus-spec-tests/tests/mainnet/bellatrix/fork/fork/pyspec_tests/bellatrix_fork_random_low_balances");

    test_case.execute(
        |state: &altair::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_bellatrix(state, context)
        },
    );
}

#[test]
fn test_bellatrix_fork_random_misc_balances() {
    let  test_case = ForkTestCase::<>::from("consensus-spec-tests/tests/mainnet/bellatrix/fork/fork/pyspec_tests/bellatrix_fork_random_misc_balances");

    test_case.execute(
        |state: &altair::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_bellatrix(state, context)
        },
    );
}

#[test]
fn test_fork_base_state() {
    let test_case = ForkTestCase::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/fork/fork/pyspec_tests/fork_base_state",
    );

    test_case.execute(
        |state: &altair::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_bellatrix(state, context)
        },
    );
}

#[test]
fn test_fork_many_next_epoch() {
    let test_case = ForkTestCase::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/fork/fork/pyspec_tests/fork_many_next_epoch",
    );

    test_case.execute(
        |state: &altair::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_bellatrix(state, context)
        },
    );
}

#[test]
fn test_fork_next_epoch() {
    let test_case = ForkTestCase::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/fork/fork/pyspec_tests/fork_next_epoch",
    );

    test_case.execute(
        |state: &altair::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_bellatrix(state, context)
        },
    );
}

#[test]
fn test_fork_next_epoch_with_block() {
    let  test_case = ForkTestCase::<>::from("consensus-spec-tests/tests/mainnet/bellatrix/fork/fork/pyspec_tests/fork_next_epoch_with_block");

    test_case.execute(
        |state: &altair::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_bellatrix(state, context)
        },
    );
}

#[test]
fn test_fork_random_low_balances() {
    let  test_case = ForkTestCase::<>::from("consensus-spec-tests/tests/mainnet/bellatrix/fork/fork/pyspec_tests/fork_random_low_balances");

    test_case.execute(
        |state: &altair::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_bellatrix(state, context)
        },
    );
}

#[test]
fn test_fork_random_misc_balances() {
    let  test_case = ForkTestCase::<>::from("consensus-spec-tests/tests/mainnet/bellatrix/fork/fork/pyspec_tests/fork_random_misc_balances");

    test_case.execute(
        |state: &altair::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_bellatrix(state, context)
        },
    );
}
