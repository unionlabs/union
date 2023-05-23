// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::fork::ForkTestCase;
use ethereum_consensus::altair::minimal as spec;
use ethereum_consensus::phase0::minimal as phase0;
use ssz_rs::prelude::*;

#[test]
fn test_altair_fork_random_0() {
    let test_case = ForkTestCase::from(
        "consensus-spec-tests/tests/minimal/altair/fork/fork/pyspec_tests/altair_fork_random_0",
    );

    test_case.execute(
        |state: &phase0::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_altair(state, context).unwrap()
        },
    );
}

#[test]
fn test_altair_fork_random_1() {
    let test_case = ForkTestCase::from(
        "consensus-spec-tests/tests/minimal/altair/fork/fork/pyspec_tests/altair_fork_random_1",
    );

    test_case.execute(
        |state: &phase0::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_altair(state, context).unwrap()
        },
    );
}

#[test]
fn test_altair_fork_random_2() {
    let test_case = ForkTestCase::from(
        "consensus-spec-tests/tests/minimal/altair/fork/fork/pyspec_tests/altair_fork_random_2",
    );

    test_case.execute(
        |state: &phase0::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_altair(state, context).unwrap()
        },
    );
}

#[test]
fn test_altair_fork_random_3() {
    let test_case = ForkTestCase::from(
        "consensus-spec-tests/tests/minimal/altair/fork/fork/pyspec_tests/altair_fork_random_3",
    );

    test_case.execute(
        |state: &phase0::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_altair(state, context).unwrap()
        },
    );
}

#[test]
fn test_altair_fork_random_duplicate_attestations() {
    let  test_case = ForkTestCase::<>::from("consensus-spec-tests/tests/minimal/altair/fork/fork/pyspec_tests/altair_fork_random_duplicate_attestations");

    test_case.execute(
        |state: &phase0::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_altair(state, context).unwrap()
        },
    );
}

#[test]
fn test_altair_fork_random_large_validator_set() {
    let  test_case = ForkTestCase::<>::from("consensus-spec-tests/tests/minimal/altair/fork/fork/pyspec_tests/altair_fork_random_large_validator_set");

    test_case.execute(
        |state: &phase0::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_altair(state, context).unwrap()
        },
    );
}

#[test]
fn test_altair_fork_random_low_balances() {
    let  test_case = ForkTestCase::<>::from("consensus-spec-tests/tests/minimal/altair/fork/fork/pyspec_tests/altair_fork_random_low_balances");

    test_case.execute(
        |state: &phase0::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_altair(state, context).unwrap()
        },
    );
}

#[test]
fn test_altair_fork_random_misc_balances() {
    let  test_case = ForkTestCase::<>::from("consensus-spec-tests/tests/minimal/altair/fork/fork/pyspec_tests/altair_fork_random_misc_balances");

    test_case.execute(
        |state: &phase0::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_altair(state, context).unwrap()
        },
    );
}

#[test]
fn test_altair_fork_random_mismatched_attestations() {
    let  test_case = ForkTestCase::<>::from("consensus-spec-tests/tests/minimal/altair/fork/fork/pyspec_tests/altair_fork_random_mismatched_attestations");

    test_case.execute(
        |state: &phase0::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_altair(state, context).unwrap()
        },
    );
}

#[test]
fn test_fork_base_state() {
    let test_case = ForkTestCase::from(
        "consensus-spec-tests/tests/minimal/altair/fork/fork/pyspec_tests/fork_base_state",
    );

    test_case.execute(
        |state: &phase0::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_altair(state, context).unwrap()
        },
    );
}

#[test]
fn test_fork_many_next_epoch() {
    let test_case = ForkTestCase::from(
        "consensus-spec-tests/tests/minimal/altair/fork/fork/pyspec_tests/fork_many_next_epoch",
    );

    test_case.execute(
        |state: &phase0::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_altair(state, context).unwrap()
        },
    );
}

#[test]
fn test_fork_next_epoch() {
    let test_case = ForkTestCase::from(
        "consensus-spec-tests/tests/minimal/altair/fork/fork/pyspec_tests/fork_next_epoch",
    );

    test_case.execute(
        |state: &phase0::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_altair(state, context).unwrap()
        },
    );
}

#[test]
fn test_fork_next_epoch_with_block() {
    let  test_case = ForkTestCase::<>::from("consensus-spec-tests/tests/minimal/altair/fork/fork/pyspec_tests/fork_next_epoch_with_block");

    test_case.execute(
        |state: &phase0::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_altair(state, context).unwrap()
        },
    );
}

#[test]
fn test_fork_random_large_validator_set() {
    let  test_case = ForkTestCase::<>::from("consensus-spec-tests/tests/minimal/altair/fork/fork/pyspec_tests/fork_random_large_validator_set");

    test_case.execute(
        |state: &phase0::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_altair(state, context).unwrap()
        },
    );
}

#[test]
fn test_fork_random_low_balances() {
    let test_case = ForkTestCase::from(
        "consensus-spec-tests/tests/minimal/altair/fork/fork/pyspec_tests/fork_random_low_balances",
    );

    test_case.execute(
        |state: &phase0::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_altair(state, context).unwrap()
        },
    );
}

#[test]
fn test_fork_random_misc_balances() {
    let  test_case = ForkTestCase::<>::from("consensus-spec-tests/tests/minimal/altair/fork/fork/pyspec_tests/fork_random_misc_balances");

    test_case.execute(
        |state: &phase0::BeaconState, context| -> spec::BeaconState {
            spec::upgrade_to_altair(state, context).unwrap()
        },
    );
}
