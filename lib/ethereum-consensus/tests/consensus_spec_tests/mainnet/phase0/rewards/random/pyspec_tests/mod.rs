// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::rewards::RandomTestCase;
use ethereum_consensus::phase0::mainnet as spec;
use ssz_rs::prelude::*;

#[test]
fn test_full_random_0() {
    let test_case = RandomTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/mainnet/phase0/rewards/random/pyspec_tests/full_random_0",
    );

    test_case.execute(|state, context| {
        let source_deltas = spec::get_source_deltas(state, context).unwrap();
        let target_deltas = spec::get_target_deltas(state, context).unwrap();
        let head_deltas = spec::get_head_deltas(state, context).unwrap();
        let inclusion_penalty_deltas = spec::get_inclusion_delay_deltas(state, context).unwrap();
        let inactivity_penalty_deltas =
            spec::get_inactivity_penalty_deltas(state, context).unwrap();
        (
            source_deltas,
            target_deltas,
            head_deltas,
            Some(inclusion_penalty_deltas),
            inactivity_penalty_deltas,
        )
    });
}

#[test]
fn test_full_random_1() {
    let test_case = RandomTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/mainnet/phase0/rewards/random/pyspec_tests/full_random_1",
    );

    test_case.execute(|state, context| {
        let source_deltas = spec::get_source_deltas(state, context).unwrap();
        let target_deltas = spec::get_target_deltas(state, context).unwrap();
        let head_deltas = spec::get_head_deltas(state, context).unwrap();
        let inclusion_penalty_deltas = spec::get_inclusion_delay_deltas(state, context).unwrap();
        let inactivity_penalty_deltas =
            spec::get_inactivity_penalty_deltas(state, context).unwrap();
        (
            source_deltas,
            target_deltas,
            head_deltas,
            Some(inclusion_penalty_deltas),
            inactivity_penalty_deltas,
        )
    });
}

#[test]
fn test_full_random_2() {
    let test_case = RandomTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/mainnet/phase0/rewards/random/pyspec_tests/full_random_2",
    );

    test_case.execute(|state, context| {
        let source_deltas = spec::get_source_deltas(state, context).unwrap();
        let target_deltas = spec::get_target_deltas(state, context).unwrap();
        let head_deltas = spec::get_head_deltas(state, context).unwrap();
        let inclusion_penalty_deltas = spec::get_inclusion_delay_deltas(state, context).unwrap();
        let inactivity_penalty_deltas =
            spec::get_inactivity_penalty_deltas(state, context).unwrap();
        (
            source_deltas,
            target_deltas,
            head_deltas,
            Some(inclusion_penalty_deltas),
            inactivity_penalty_deltas,
        )
    });
}

#[test]
fn test_full_random_3() {
    let test_case = RandomTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/mainnet/phase0/rewards/random/pyspec_tests/full_random_3",
    );

    test_case.execute(|state, context| {
        let source_deltas = spec::get_source_deltas(state, context).unwrap();
        let target_deltas = spec::get_target_deltas(state, context).unwrap();
        let head_deltas = spec::get_head_deltas(state, context).unwrap();
        let inclusion_penalty_deltas = spec::get_inclusion_delay_deltas(state, context).unwrap();
        let inactivity_penalty_deltas =
            spec::get_inactivity_penalty_deltas(state, context).unwrap();
        (
            source_deltas,
            target_deltas,
            head_deltas,
            Some(inclusion_penalty_deltas),
            inactivity_penalty_deltas,
        )
    });
}

#[test]
fn test_full_random_4() {
    let test_case = RandomTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/mainnet/phase0/rewards/random/pyspec_tests/full_random_4",
    );

    test_case.execute(|state, context| {
        let source_deltas = spec::get_source_deltas(state, context).unwrap();
        let target_deltas = spec::get_target_deltas(state, context).unwrap();
        let head_deltas = spec::get_head_deltas(state, context).unwrap();
        let inclusion_penalty_deltas = spec::get_inclusion_delay_deltas(state, context).unwrap();
        let inactivity_penalty_deltas =
            spec::get_inactivity_penalty_deltas(state, context).unwrap();
        (
            source_deltas,
            target_deltas,
            head_deltas,
            Some(inclusion_penalty_deltas),
            inactivity_penalty_deltas,
        )
    });
}

#[test]
fn test_full_random_low_balances_0() {
    let  test_case = RandomTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/phase0/rewards/random/pyspec_tests/full_random_low_balances_0");

    test_case.execute(|state, context| {
        let source_deltas = spec::get_source_deltas(state, context).unwrap();
        let target_deltas = spec::get_target_deltas(state, context).unwrap();
        let head_deltas = spec::get_head_deltas(state, context).unwrap();
        let inclusion_penalty_deltas = spec::get_inclusion_delay_deltas(state, context).unwrap();
        let inactivity_penalty_deltas =
            spec::get_inactivity_penalty_deltas(state, context).unwrap();
        (
            source_deltas,
            target_deltas,
            head_deltas,
            Some(inclusion_penalty_deltas),
            inactivity_penalty_deltas,
        )
    });
}

#[test]
fn test_full_random_low_balances_1() {
    let  test_case = RandomTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/phase0/rewards/random/pyspec_tests/full_random_low_balances_1");

    test_case.execute(|state, context| {
        let source_deltas = spec::get_source_deltas(state, context).unwrap();
        let target_deltas = spec::get_target_deltas(state, context).unwrap();
        let head_deltas = spec::get_head_deltas(state, context).unwrap();
        let inclusion_penalty_deltas = spec::get_inclusion_delay_deltas(state, context).unwrap();
        let inactivity_penalty_deltas =
            spec::get_inactivity_penalty_deltas(state, context).unwrap();
        (
            source_deltas,
            target_deltas,
            head_deltas,
            Some(inclusion_penalty_deltas),
            inactivity_penalty_deltas,
        )
    });
}

#[test]
fn test_full_random_misc_balances() {
    let  test_case = RandomTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/phase0/rewards/random/pyspec_tests/full_random_misc_balances");

    test_case.execute(|state, context| {
        let source_deltas = spec::get_source_deltas(state, context).unwrap();
        let target_deltas = spec::get_target_deltas(state, context).unwrap();
        let head_deltas = spec::get_head_deltas(state, context).unwrap();
        let inclusion_penalty_deltas = spec::get_inclusion_delay_deltas(state, context).unwrap();
        let inactivity_penalty_deltas =
            spec::get_inactivity_penalty_deltas(state, context).unwrap();
        (
            source_deltas,
            target_deltas,
            head_deltas,
            Some(inclusion_penalty_deltas),
            inactivity_penalty_deltas,
        )
    });
}

#[test]
fn test_full_random_without_leak_0() {
    let  test_case = RandomTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/phase0/rewards/random/pyspec_tests/full_random_without_leak_0");

    test_case.execute(|state, context| {
        let source_deltas = spec::get_source_deltas(state, context).unwrap();
        let target_deltas = spec::get_target_deltas(state, context).unwrap();
        let head_deltas = spec::get_head_deltas(state, context).unwrap();
        let inclusion_penalty_deltas = spec::get_inclusion_delay_deltas(state, context).unwrap();
        let inactivity_penalty_deltas =
            spec::get_inactivity_penalty_deltas(state, context).unwrap();
        (
            source_deltas,
            target_deltas,
            head_deltas,
            Some(inclusion_penalty_deltas),
            inactivity_penalty_deltas,
        )
    });
}

#[test]
fn test_full_random_without_leak_and_current_exit_0() {
    let  test_case = RandomTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/phase0/rewards/random/pyspec_tests/full_random_without_leak_and_current_exit_0");

    test_case.execute(|state, context| {
        let source_deltas = spec::get_source_deltas(state, context).unwrap();
        let target_deltas = spec::get_target_deltas(state, context).unwrap();
        let head_deltas = spec::get_head_deltas(state, context).unwrap();
        let inclusion_penalty_deltas = spec::get_inclusion_delay_deltas(state, context).unwrap();
        let inactivity_penalty_deltas =
            spec::get_inactivity_penalty_deltas(state, context).unwrap();
        (
            source_deltas,
            target_deltas,
            head_deltas,
            Some(inclusion_penalty_deltas),
            inactivity_penalty_deltas,
        )
    });
}
