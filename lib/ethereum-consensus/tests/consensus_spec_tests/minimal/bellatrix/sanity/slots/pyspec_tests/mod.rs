// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::sanity::SlotsTestCase;
use ethereum_consensus::bellatrix::minimal as spec;
use ssz_rs::prelude::*;

#[test]
fn test_double_empty_epoch() {
    let mut test_case = SlotsTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/minimal/bellatrix/sanity/slots/pyspec_tests/double_empty_epoch",
    );

    test_case.execute(|state, offset, context| {
        let target_slot = state.slot + offset;
        spec::process_slots(state, target_slot, context).unwrap();
    });
}

#[test]
fn test_empty_epoch() {
    let mut test_case = SlotsTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/minimal/bellatrix/sanity/slots/pyspec_tests/empty_epoch",
    );

    test_case.execute(|state, offset, context| {
        let target_slot = state.slot + offset;
        spec::process_slots(state, target_slot, context).unwrap();
    });
}

#[test]
fn test_over_epoch_boundary() {
    let mut test_case = SlotsTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/slots/pyspec_tests/over_epoch_boundary");

    test_case.execute(|state, offset, context| {
        let target_slot = state.slot + offset;
        spec::process_slots(state, target_slot, context).unwrap();
    });
}

#[test]
fn test_slots_1() {
    let mut test_case = SlotsTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/minimal/bellatrix/sanity/slots/pyspec_tests/slots_1",
    );

    test_case.execute(|state, offset, context| {
        let target_slot = state.slot + offset;
        spec::process_slots(state, target_slot, context).unwrap();
    });
}

#[test]
fn test_slots_2() {
    let mut test_case = SlotsTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/minimal/bellatrix/sanity/slots/pyspec_tests/slots_2",
    );

    test_case.execute(|state, offset, context| {
        let target_slot = state.slot + offset;
        spec::process_slots(state, target_slot, context).unwrap();
    });
}
