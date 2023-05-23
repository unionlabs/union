// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::rewards::LeakTestCase;
use ethereum_consensus::phase0::mainnet as spec;
use ssz_rs::prelude::*;

#[test]
fn test_empty_leak() {
    let test_case = LeakTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/mainnet/phase0/rewards/leak/pyspec_tests/empty_leak",
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
fn test_full_but_partial_participation_leak() {
    let  test_case = LeakTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/phase0/rewards/leak/pyspec_tests/full_but_partial_participation_leak");

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
fn test_full_correct_target_incorrect_head_leak() {
    let  test_case = LeakTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/phase0/rewards/leak/pyspec_tests/full_correct_target_incorrect_head_leak");

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
fn test_full_half_correct_target_incorrect_head_leak() {
    let  test_case = LeakTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/phase0/rewards/leak/pyspec_tests/full_half_correct_target_incorrect_head_leak");

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
fn test_full_half_incorrect_target_correct_head_leak() {
    let  test_case = LeakTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/phase0/rewards/leak/pyspec_tests/full_half_incorrect_target_correct_head_leak");

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
fn test_full_half_incorrect_target_incorrect_head_leak() {
    let  test_case = LeakTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/phase0/rewards/leak/pyspec_tests/full_half_incorrect_target_incorrect_head_leak");

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
fn test_full_leak() {
    let test_case = LeakTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/mainnet/phase0/rewards/leak/pyspec_tests/full_leak",
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
fn test_full_random_leak() {
    let test_case = LeakTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/mainnet/phase0/rewards/leak/pyspec_tests/full_random_leak",
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
fn test_full_random_seven_epoch_leak() {
    let  test_case = LeakTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/phase0/rewards/leak/pyspec_tests/full_random_seven_epoch_leak");

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
fn test_full_random_ten_epoch_leak() {
    let  test_case = LeakTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/phase0/rewards/leak/pyspec_tests/full_random_ten_epoch_leak");

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
fn test_half_full_leak() {
    let test_case = LeakTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/mainnet/phase0/rewards/leak/pyspec_tests/half_full_leak",
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
fn test_one_attestation_one_correct_leak() {
    let  test_case = LeakTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/phase0/rewards/leak/pyspec_tests/one_attestation_one_correct_leak");

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
fn test_quarter_full_leak() {
    let test_case = LeakTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/mainnet/phase0/rewards/leak/pyspec_tests/quarter_full_leak",
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
fn test_some_very_low_effective_balances_that_attested_leak() {
    let  test_case = LeakTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/phase0/rewards/leak/pyspec_tests/some_very_low_effective_balances_that_attested_leak");

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
fn test_some_very_low_effective_balances_that_did_not_attest_leak() {
    let  test_case = LeakTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/phase0/rewards/leak/pyspec_tests/some_very_low_effective_balances_that_did_not_attest_leak");

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
fn test_with_exited_validators_leak() {
    let  test_case = LeakTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/phase0/rewards/leak/pyspec_tests/with_exited_validators_leak");

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
fn test_with_not_yet_activated_validators_leak() {
    let  test_case = LeakTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/phase0/rewards/leak/pyspec_tests/with_not_yet_activated_validators_leak");

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
fn test_with_slashed_validators_leak() {
    let  test_case = LeakTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/phase0/rewards/leak/pyspec_tests/with_slashed_validators_leak");

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
