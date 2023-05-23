// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::rewards::BasicTestCase;
use ethereum_consensus::altair::minimal as spec;
use ssz_rs::prelude::*;

#[test]
fn test_all_balances_too_low_for_reward() {
    let  test_case = BasicTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/rewards/basic/pyspec_tests/all_balances_too_low_for_reward");

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
fn test_empty() {
    let test_case = BasicTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/minimal/altair/rewards/basic/pyspec_tests/empty",
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
fn test_full_all_correct() {
    let test_case = BasicTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/minimal/altair/rewards/basic/pyspec_tests/full_all_correct",
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
fn test_full_but_partial_participation() {
    let  test_case = BasicTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/rewards/basic/pyspec_tests/full_but_partial_participation");

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
fn test_half_full() {
    let test_case = BasicTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/minimal/altair/rewards/basic/pyspec_tests/half_full",
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
fn test_quarter_full() {
    let test_case = BasicTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/minimal/altair/rewards/basic/pyspec_tests/quarter_full",
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
fn test_some_very_low_effective_balances_that_attested() {
    let  test_case = BasicTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/rewards/basic/pyspec_tests/some_very_low_effective_balances_that_attested");

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
fn test_some_very_low_effective_balances_that_did_not_attest() {
    let  test_case = BasicTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/rewards/basic/pyspec_tests/some_very_low_effective_balances_that_did_not_attest");

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
fn test_with_exited_validators() {
    let  test_case = BasicTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/rewards/basic/pyspec_tests/with_exited_validators");

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
fn test_with_not_yet_activated_validators() {
    let  test_case = BasicTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/rewards/basic/pyspec_tests/with_not_yet_activated_validators");

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
fn test_with_slashed_validators() {
    let  test_case = BasicTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/rewards/basic/pyspec_tests/with_slashed_validators");

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
