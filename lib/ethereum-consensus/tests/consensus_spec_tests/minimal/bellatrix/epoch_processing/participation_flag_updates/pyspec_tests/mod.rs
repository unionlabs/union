// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::epoch_processing::ParticipationFlagUpdatesTestCase;
use ethereum_consensus::bellatrix::minimal as spec;
use ssz_rs::prelude::*;

#[test]
fn test_all_zeroed() {
    let mut test_case = ParticipationFlagUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/participation_flag_updates/pyspec_tests/all_zeroed");

    test_case.execute(|state, context| spec::process_participation_flag_updates(state));
}

#[test]
fn test_current_epoch_zeroed() {
    let mut test_case = ParticipationFlagUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/participation_flag_updates/pyspec_tests/current_epoch_zeroed");

    test_case.execute(|state, context| spec::process_participation_flag_updates(state));
}

#[test]
fn test_current_filled() {
    let mut test_case = ParticipationFlagUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/participation_flag_updates/pyspec_tests/current_filled");

    test_case.execute(|state, context| spec::process_participation_flag_updates(state));
}

#[test]
fn test_filled() {
    let mut test_case = ParticipationFlagUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/participation_flag_updates/pyspec_tests/filled");

    test_case.execute(|state, context| spec::process_participation_flag_updates(state));
}

#[test]
fn test_large_random() {
    let mut test_case = ParticipationFlagUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/participation_flag_updates/pyspec_tests/large_random");

    test_case.execute(|state, context| spec::process_participation_flag_updates(state));
}

#[test]
fn test_previous_epoch_zeroed() {
    let mut test_case = ParticipationFlagUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/participation_flag_updates/pyspec_tests/previous_epoch_zeroed");

    test_case.execute(|state, context| spec::process_participation_flag_updates(state));
}

#[test]
fn test_previous_filled() {
    let mut test_case = ParticipationFlagUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/participation_flag_updates/pyspec_tests/previous_filled");

    test_case.execute(|state, context| spec::process_participation_flag_updates(state));
}

#[test]
fn test_random_0() {
    let mut test_case = ParticipationFlagUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/participation_flag_updates/pyspec_tests/random_0");

    test_case.execute(|state, context| spec::process_participation_flag_updates(state));
}

#[test]
fn test_random_1() {
    let mut test_case = ParticipationFlagUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/participation_flag_updates/pyspec_tests/random_1");

    test_case.execute(|state, context| spec::process_participation_flag_updates(state));
}

#[test]
fn test_random_2() {
    let mut test_case = ParticipationFlagUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/participation_flag_updates/pyspec_tests/random_2");

    test_case.execute(|state, context| spec::process_participation_flag_updates(state));
}

#[test]
fn test_random_genesis() {
    let mut test_case = ParticipationFlagUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/participation_flag_updates/pyspec_tests/random_genesis");

    test_case.execute(|state, context| spec::process_participation_flag_updates(state));
}

#[test]
fn test_slightly_larger_random() {
    let mut test_case = ParticipationFlagUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/participation_flag_updates/pyspec_tests/slightly_larger_random");

    test_case.execute(|state, context| spec::process_participation_flag_updates(state));
}
