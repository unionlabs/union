// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::epoch_processing::InactivityUpdatesTestCase;
use ethereum_consensus::bellatrix::minimal as spec;
use ssz_rs::prelude::*;

#[test]
fn test_all_zero_inactivity_scores_empty_participation() {
    let mut test_case = InactivityUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/inactivity_updates/pyspec_tests/all_zero_inactivity_scores_empty_participation");

    test_case.execute(|state, context| spec::process_inactivity_updates(state, context));
}

#[test]
fn test_all_zero_inactivity_scores_empty_participation_leaking() {
    let mut test_case = InactivityUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/inactivity_updates/pyspec_tests/all_zero_inactivity_scores_empty_participation_leaking");

    test_case.execute(|state, context| spec::process_inactivity_updates(state, context));
}

#[test]
fn test_all_zero_inactivity_scores_full_participation() {
    let mut test_case = InactivityUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/inactivity_updates/pyspec_tests/all_zero_inactivity_scores_full_participation");

    test_case.execute(|state, context| spec::process_inactivity_updates(state, context));
}

#[test]
fn test_all_zero_inactivity_scores_full_participation_leaking() {
    let mut test_case = InactivityUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/inactivity_updates/pyspec_tests/all_zero_inactivity_scores_full_participation_leaking");

    test_case.execute(|state, context| spec::process_inactivity_updates(state, context));
}

#[test]
fn test_all_zero_inactivity_scores_random_participation() {
    let mut test_case = InactivityUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/inactivity_updates/pyspec_tests/all_zero_inactivity_scores_random_participation");

    test_case.execute(|state, context| spec::process_inactivity_updates(state, context));
}

#[test]
fn test_all_zero_inactivity_scores_random_participation_leaking() {
    let mut test_case = InactivityUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/inactivity_updates/pyspec_tests/all_zero_inactivity_scores_random_participation_leaking");

    test_case.execute(|state, context| spec::process_inactivity_updates(state, context));
}

#[test]
fn test_genesis() {
    let mut test_case = InactivityUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/inactivity_updates/pyspec_tests/genesis");

    test_case.execute(|state, context| spec::process_inactivity_updates(state, context));
}

#[test]
fn test_genesis_random_scores() {
    let mut test_case = InactivityUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/inactivity_updates/pyspec_tests/genesis_random_scores");

    test_case.execute(|state, context| spec::process_inactivity_updates(state, context));
}

#[test]
fn test_random_inactivity_scores_empty_participation() {
    let mut test_case = InactivityUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/inactivity_updates/pyspec_tests/random_inactivity_scores_empty_participation");

    test_case.execute(|state, context| spec::process_inactivity_updates(state, context));
}

#[test]
fn test_random_inactivity_scores_empty_participation_leaking() {
    let mut test_case = InactivityUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/inactivity_updates/pyspec_tests/random_inactivity_scores_empty_participation_leaking");

    test_case.execute(|state, context| spec::process_inactivity_updates(state, context));
}

#[test]
fn test_random_inactivity_scores_full_participation() {
    let mut test_case = InactivityUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/inactivity_updates/pyspec_tests/random_inactivity_scores_full_participation");

    test_case.execute(|state, context| spec::process_inactivity_updates(state, context));
}

#[test]
fn test_random_inactivity_scores_full_participation_leaking() {
    let mut test_case = InactivityUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/inactivity_updates/pyspec_tests/random_inactivity_scores_full_participation_leaking");

    test_case.execute(|state, context| spec::process_inactivity_updates(state, context));
}

#[test]
fn test_random_inactivity_scores_random_participation() {
    let mut test_case = InactivityUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/inactivity_updates/pyspec_tests/random_inactivity_scores_random_participation");

    test_case.execute(|state, context| spec::process_inactivity_updates(state, context));
}

#[test]
fn test_random_inactivity_scores_random_participation_leaking() {
    let mut test_case = InactivityUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/inactivity_updates/pyspec_tests/random_inactivity_scores_random_participation_leaking");

    test_case.execute(|state, context| spec::process_inactivity_updates(state, context));
}

#[test]
fn test_some_exited_full_random_leaking() {
    let mut test_case = InactivityUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/inactivity_updates/pyspec_tests/some_exited_full_random_leaking");

    test_case.execute(|state, context| spec::process_inactivity_updates(state, context));
}

#[test]
fn test_some_slashed_full_random() {
    let mut test_case = InactivityUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/inactivity_updates/pyspec_tests/some_slashed_full_random");

    test_case.execute(|state, context| spec::process_inactivity_updates(state, context));
}

#[test]
fn test_some_slashed_full_random_leaking() {
    let mut test_case = InactivityUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/inactivity_updates/pyspec_tests/some_slashed_full_random_leaking");

    test_case.execute(|state, context| spec::process_inactivity_updates(state, context));
}

#[test]
fn test_some_slashed_zero_scores_full_participation() {
    let mut test_case = InactivityUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/inactivity_updates/pyspec_tests/some_slashed_zero_scores_full_participation");

    test_case.execute(|state, context| spec::process_inactivity_updates(state, context));
}

#[test]
fn test_some_slashed_zero_scores_full_participation_leaking() {
    let mut test_case = InactivityUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/inactivity_updates/pyspec_tests/some_slashed_zero_scores_full_participation_leaking");

    test_case.execute(|state, context| spec::process_inactivity_updates(state, context));
}
