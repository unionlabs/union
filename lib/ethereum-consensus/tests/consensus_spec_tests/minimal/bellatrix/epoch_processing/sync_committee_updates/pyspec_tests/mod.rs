// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::epoch_processing::SyncCommitteeUpdatesTestCase;
use ethereum_consensus::bellatrix::minimal as spec;
use ssz_rs::prelude::*;

#[test]
fn test_sync_committees_no_progress_not_boundary() {
    let mut test_case = SyncCommitteeUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/sync_committee_updates/pyspec_tests/sync_committees_no_progress_not_boundary");

    test_case.execute(|state, context| spec::process_sync_committee_updates(state, context));
}

#[test]
fn test_sync_committees_progress_genesis() {
    let mut test_case = SyncCommitteeUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/sync_committee_updates/pyspec_tests/sync_committees_progress_genesis");

    test_case.execute(|state, context| spec::process_sync_committee_updates(state, context));
}

#[test]
fn test_sync_committees_progress_misc_balances_genesis() {
    let mut test_case = SyncCommitteeUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/sync_committee_updates/pyspec_tests/sync_committees_progress_misc_balances_genesis");

    test_case.execute(|state, context| spec::process_sync_committee_updates(state, context));
}

#[test]
fn test_sync_committees_progress_misc_balances_not_genesis() {
    let mut test_case = SyncCommitteeUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/sync_committee_updates/pyspec_tests/sync_committees_progress_misc_balances_not_genesis");

    test_case.execute(|state, context| spec::process_sync_committee_updates(state, context));
}

#[test]
fn test_sync_committees_progress_not_genesis() {
    let mut test_case = SyncCommitteeUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/sync_committee_updates/pyspec_tests/sync_committees_progress_not_genesis");

    test_case.execute(|state, context| spec::process_sync_committee_updates(state, context));
}
