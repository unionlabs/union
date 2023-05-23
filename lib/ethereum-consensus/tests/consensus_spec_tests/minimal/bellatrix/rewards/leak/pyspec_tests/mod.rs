// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::rewards::LeakTestCase;
use ethereum_consensus::bellatrix::minimal as spec;
use ssz_rs::prelude::*;

#[test]
fn test_empty_leak() {
    let test_case = LeakTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/minimal/bellatrix/rewards/leak/pyspec_tests/empty_leak",
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
fn test_full_but_partial_participation_leak() {
    let  test_case = LeakTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/rewards/leak/pyspec_tests/full_but_partial_participation_leak");

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
fn test_full_leak() {
    let test_case = LeakTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/minimal/bellatrix/rewards/leak/pyspec_tests/full_leak",
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
fn test_full_random_leak() {
    let test_case = LeakTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/minimal/bellatrix/rewards/leak/pyspec_tests/full_random_leak",
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
fn test_full_random_seven_epoch_leak() {
    let  test_case = LeakTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/rewards/leak/pyspec_tests/full_random_seven_epoch_leak");

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
fn test_full_random_ten_epoch_leak() {
    let  test_case = LeakTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/rewards/leak/pyspec_tests/full_random_ten_epoch_leak");

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
fn test_half_full_leak() {
    let test_case = LeakTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/minimal/bellatrix/rewards/leak/pyspec_tests/half_full_leak",
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
fn test_quarter_full_leak() {
    let test_case = LeakTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/minimal/bellatrix/rewards/leak/pyspec_tests/quarter_full_leak",
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
fn test_some_very_low_effective_balances_that_attested_leak() {
    let  test_case = LeakTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/rewards/leak/pyspec_tests/some_very_low_effective_balances_that_attested_leak");

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
fn test_some_very_low_effective_balances_that_did_not_attest_leak() {
    let  test_case = LeakTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/rewards/leak/pyspec_tests/some_very_low_effective_balances_that_did_not_attest_leak");

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
fn test_with_exited_validators_leak() {
    let  test_case = LeakTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/rewards/leak/pyspec_tests/with_exited_validators_leak");

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
fn test_with_not_yet_activated_validators_leak() {
    let  test_case = LeakTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/rewards/leak/pyspec_tests/with_not_yet_activated_validators_leak");

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
fn test_with_slashed_validators_leak() {
    let  test_case = LeakTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/rewards/leak/pyspec_tests/with_slashed_validators_leak");

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
