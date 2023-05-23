// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::epoch_processing::RewardsAndPenaltiesTestCase;
use ethereum_consensus::altair::minimal as spec;
use ssz_rs::prelude::*;

#[test]
fn test_almost_empty_attestations() {
    let mut test_case = RewardsAndPenaltiesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/rewards_and_penalties/pyspec_tests/almost_empty_attestations");

    test_case.execute(|state, context| spec::process_rewards_and_penalties(state, context));
}

#[test]
fn test_almost_empty_attestations_with_leak() {
    let mut test_case = RewardsAndPenaltiesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/rewards_and_penalties/pyspec_tests/almost_empty_attestations_with_leak");

    test_case.execute(|state, context| spec::process_rewards_and_penalties(state, context));
}

#[test]
fn test_almost_full_attestations() {
    let mut test_case = RewardsAndPenaltiesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/rewards_and_penalties/pyspec_tests/almost_full_attestations");

    test_case.execute(|state, context| spec::process_rewards_and_penalties(state, context));
}

#[test]
fn test_almost_full_attestations_with_leak() {
    let mut test_case = RewardsAndPenaltiesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/rewards_and_penalties/pyspec_tests/almost_full_attestations_with_leak");

    test_case.execute(|state, context| spec::process_rewards_and_penalties(state, context));
}

#[test]
fn test_attestations_some_slashed() {
    let mut test_case = RewardsAndPenaltiesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/rewards_and_penalties/pyspec_tests/attestations_some_slashed");

    test_case.execute(|state, context| spec::process_rewards_and_penalties(state, context));
}

#[test]
fn test_duplicate_attestation() {
    let mut test_case = RewardsAndPenaltiesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/rewards_and_penalties/pyspec_tests/duplicate_attestation");

    test_case.execute(|state, context| spec::process_rewards_and_penalties(state, context));
}

#[test]
fn test_full_attestation_participation() {
    let mut test_case = RewardsAndPenaltiesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/rewards_and_penalties/pyspec_tests/full_attestation_participation");

    test_case.execute(|state, context| spec::process_rewards_and_penalties(state, context));
}

#[test]
fn test_full_attestation_participation_with_leak() {
    let mut test_case = RewardsAndPenaltiesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/rewards_and_penalties/pyspec_tests/full_attestation_participation_with_leak");

    test_case.execute(|state, context| spec::process_rewards_and_penalties(state, context));
}

#[test]
fn test_full_attestations_misc_balances() {
    let mut test_case = RewardsAndPenaltiesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/rewards_and_penalties/pyspec_tests/full_attestations_misc_balances");

    test_case.execute(|state, context| spec::process_rewards_and_penalties(state, context));
}

#[test]
fn test_full_attestations_one_validaor_one_gwei() {
    let mut test_case = RewardsAndPenaltiesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/rewards_and_penalties/pyspec_tests/full_attestations_one_validaor_one_gwei");

    test_case.execute(|state, context| spec::process_rewards_and_penalties(state, context));
}

#[test]
fn test_genesis_epoch_full_attestations_no_rewards() {
    let mut test_case = RewardsAndPenaltiesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/rewards_and_penalties/pyspec_tests/genesis_epoch_full_attestations_no_rewards");

    test_case.execute(|state, context| spec::process_rewards_and_penalties(state, context));
}

#[test]
fn test_genesis_epoch_no_attestations_no_penalties() {
    let mut test_case = RewardsAndPenaltiesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/rewards_and_penalties/pyspec_tests/genesis_epoch_no_attestations_no_penalties");

    test_case.execute(|state, context| spec::process_rewards_and_penalties(state, context));
}

#[test]
fn test_no_attestations_all_penalties() {
    let mut test_case = RewardsAndPenaltiesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/rewards_and_penalties/pyspec_tests/no_attestations_all_penalties");

    test_case.execute(|state, context| spec::process_rewards_and_penalties(state, context));
}

#[test]
fn test_random_fill_attestations() {
    let mut test_case = RewardsAndPenaltiesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/rewards_and_penalties/pyspec_tests/random_fill_attestations");

    test_case.execute(|state, context| spec::process_rewards_and_penalties(state, context));
}

#[test]
fn test_random_fill_attestations_with_leak() {
    let mut test_case = RewardsAndPenaltiesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/rewards_and_penalties/pyspec_tests/random_fill_attestations_with_leak");

    test_case.execute(|state, context| spec::process_rewards_and_penalties(state, context));
}
