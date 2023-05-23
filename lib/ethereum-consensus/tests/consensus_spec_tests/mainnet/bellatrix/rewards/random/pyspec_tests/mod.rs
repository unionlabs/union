// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::rewards::RandomTestCase;
use ethereum_consensus::bellatrix::mainnet as spec;
use ssz_rs::prelude::*;

#[test]
fn test_full_random_0() {
    let test_case = RandomTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/rewards/random/pyspec_tests/full_random_0",
    );

    test_case.execute(|state, context| {
        let flag_index = spec::TIMELY_SOURCE_FLAG_INDEX;
        let source_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let flag_index = spec::TIMELY_TARGET_FLAG_INDEX;
        let target_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let flag_index = spec::TIMELY_HEAD_FLAG_INDEX;
        let head_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let inactivity_penalty_deltas =
            spec::get_inactivity_penalty_deltas(state, context).unwrap();
        (
            source_deltas,
            target_deltas,
            head_deltas,
            None,
            inactivity_penalty_deltas,
        )
    });
}

#[test]
fn test_full_random_1() {
    let test_case = RandomTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/rewards/random/pyspec_tests/full_random_1",
    );

    test_case.execute(|state, context| {
        let flag_index = spec::TIMELY_SOURCE_FLAG_INDEX;
        let source_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let flag_index = spec::TIMELY_TARGET_FLAG_INDEX;
        let target_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let flag_index = spec::TIMELY_HEAD_FLAG_INDEX;
        let head_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let inactivity_penalty_deltas =
            spec::get_inactivity_penalty_deltas(state, context).unwrap();
        (
            source_deltas,
            target_deltas,
            head_deltas,
            None,
            inactivity_penalty_deltas,
        )
    });
}

#[test]
fn test_full_random_2() {
    let test_case = RandomTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/rewards/random/pyspec_tests/full_random_2",
    );

    test_case.execute(|state, context| {
        let flag_index = spec::TIMELY_SOURCE_FLAG_INDEX;
        let source_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let flag_index = spec::TIMELY_TARGET_FLAG_INDEX;
        let target_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let flag_index = spec::TIMELY_HEAD_FLAG_INDEX;
        let head_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let inactivity_penalty_deltas =
            spec::get_inactivity_penalty_deltas(state, context).unwrap();
        (
            source_deltas,
            target_deltas,
            head_deltas,
            None,
            inactivity_penalty_deltas,
        )
    });
}

#[test]
fn test_full_random_3() {
    let test_case = RandomTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/rewards/random/pyspec_tests/full_random_3",
    );

    test_case.execute(|state, context| {
        let flag_index = spec::TIMELY_SOURCE_FLAG_INDEX;
        let source_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let flag_index = spec::TIMELY_TARGET_FLAG_INDEX;
        let target_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let flag_index = spec::TIMELY_HEAD_FLAG_INDEX;
        let head_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let inactivity_penalty_deltas =
            spec::get_inactivity_penalty_deltas(state, context).unwrap();
        (
            source_deltas,
            target_deltas,
            head_deltas,
            None,
            inactivity_penalty_deltas,
        )
    });
}

#[test]
fn test_full_random_4() {
    let test_case = RandomTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/rewards/random/pyspec_tests/full_random_4",
    );

    test_case.execute(|state, context| {
        let flag_index = spec::TIMELY_SOURCE_FLAG_INDEX;
        let source_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let flag_index = spec::TIMELY_TARGET_FLAG_INDEX;
        let target_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let flag_index = spec::TIMELY_HEAD_FLAG_INDEX;
        let head_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let inactivity_penalty_deltas =
            spec::get_inactivity_penalty_deltas(state, context).unwrap();
        (
            source_deltas,
            target_deltas,
            head_deltas,
            None,
            inactivity_penalty_deltas,
        )
    });
}

#[test]
fn test_full_random_low_balances_0() {
    let  test_case = RandomTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/bellatrix/rewards/random/pyspec_tests/full_random_low_balances_0");

    test_case.execute(|state, context| {
        let flag_index = spec::TIMELY_SOURCE_FLAG_INDEX;
        let source_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let flag_index = spec::TIMELY_TARGET_FLAG_INDEX;
        let target_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let flag_index = spec::TIMELY_HEAD_FLAG_INDEX;
        let head_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let inactivity_penalty_deltas =
            spec::get_inactivity_penalty_deltas(state, context).unwrap();
        (
            source_deltas,
            target_deltas,
            head_deltas,
            None,
            inactivity_penalty_deltas,
        )
    });
}

#[test]
fn test_full_random_low_balances_1() {
    let  test_case = RandomTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/bellatrix/rewards/random/pyspec_tests/full_random_low_balances_1");

    test_case.execute(|state, context| {
        let flag_index = spec::TIMELY_SOURCE_FLAG_INDEX;
        let source_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let flag_index = spec::TIMELY_TARGET_FLAG_INDEX;
        let target_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let flag_index = spec::TIMELY_HEAD_FLAG_INDEX;
        let head_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let inactivity_penalty_deltas =
            spec::get_inactivity_penalty_deltas(state, context).unwrap();
        (
            source_deltas,
            target_deltas,
            head_deltas,
            None,
            inactivity_penalty_deltas,
        )
    });
}

#[test]
fn test_full_random_misc_balances() {
    let  test_case = RandomTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/bellatrix/rewards/random/pyspec_tests/full_random_misc_balances");

    test_case.execute(|state, context| {
        let flag_index = spec::TIMELY_SOURCE_FLAG_INDEX;
        let source_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let flag_index = spec::TIMELY_TARGET_FLAG_INDEX;
        let target_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let flag_index = spec::TIMELY_HEAD_FLAG_INDEX;
        let head_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let inactivity_penalty_deltas =
            spec::get_inactivity_penalty_deltas(state, context).unwrap();
        (
            source_deltas,
            target_deltas,
            head_deltas,
            None,
            inactivity_penalty_deltas,
        )
    });
}

#[test]
fn test_full_random_without_leak_0() {
    let  test_case = RandomTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/bellatrix/rewards/random/pyspec_tests/full_random_without_leak_0");

    test_case.execute(|state, context| {
        let flag_index = spec::TIMELY_SOURCE_FLAG_INDEX;
        let source_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let flag_index = spec::TIMELY_TARGET_FLAG_INDEX;
        let target_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let flag_index = spec::TIMELY_HEAD_FLAG_INDEX;
        let head_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let inactivity_penalty_deltas =
            spec::get_inactivity_penalty_deltas(state, context).unwrap();
        (
            source_deltas,
            target_deltas,
            head_deltas,
            None,
            inactivity_penalty_deltas,
        )
    });
}

#[test]
fn test_full_random_without_leak_and_current_exit_0() {
    let  test_case = RandomTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/bellatrix/rewards/random/pyspec_tests/full_random_without_leak_and_current_exit_0");

    test_case.execute(|state, context| {
        let flag_index = spec::TIMELY_SOURCE_FLAG_INDEX;
        let source_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let flag_index = spec::TIMELY_TARGET_FLAG_INDEX;
        let target_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let flag_index = spec::TIMELY_HEAD_FLAG_INDEX;
        let head_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
        let inactivity_penalty_deltas =
            spec::get_inactivity_penalty_deltas(state, context).unwrap();
        (
            source_deltas,
            target_deltas,
            head_deltas,
            None,
            inactivity_penalty_deltas,
        )
    });
}
