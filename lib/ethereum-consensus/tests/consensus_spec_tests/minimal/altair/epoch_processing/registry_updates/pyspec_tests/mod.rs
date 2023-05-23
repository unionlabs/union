// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::epoch_processing::RegistryUpdatesTestCase;
use ethereum_consensus::altair::minimal as spec;
use ssz_rs::prelude::*;

#[test]
fn test_activation_queue_activation_and_ejection_1() {
    let mut test_case = RegistryUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/registry_updates/pyspec_tests/activation_queue_activation_and_ejection__1");

    test_case.execute(|state, context| {
        spec::process_registry_updates(state, context);
        Ok(())
    });
}

#[test]
fn test_activation_queue_activation_and_ejection_churn_limit() {
    let mut test_case = RegistryUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/registry_updates/pyspec_tests/activation_queue_activation_and_ejection__churn_limit");

    test_case.execute(|state, context| {
        spec::process_registry_updates(state, context);
        Ok(())
    });
}

#[test]
fn test_activation_queue_activation_and_ejection_exceed_churn_limit() {
    let mut test_case = RegistryUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/registry_updates/pyspec_tests/activation_queue_activation_and_ejection__exceed_churn_limit");

    test_case.execute(|state, context| {
        spec::process_registry_updates(state, context);
        Ok(())
    });
}

#[test]
fn test_activation_queue_activation_and_ejection_exceed_scaled_churn_limit() {
    let mut test_case = RegistryUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/registry_updates/pyspec_tests/activation_queue_activation_and_ejection__exceed_scaled_churn_limit");

    test_case.execute(|state, context| {
        spec::process_registry_updates(state, context);
        Ok(())
    });
}

#[test]
fn test_activation_queue_activation_and_ejection_scaled_churn_limit() {
    let mut test_case = RegistryUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/registry_updates/pyspec_tests/activation_queue_activation_and_ejection__scaled_churn_limit");

    test_case.execute(|state, context| {
        spec::process_registry_updates(state, context);
        Ok(())
    });
}

#[test]
fn test_activation_queue_efficiency_min() {
    let mut test_case = RegistryUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/registry_updates/pyspec_tests/activation_queue_efficiency_min");

    test_case.execute(|state, context| {
        spec::process_registry_updates(state, context);
        Ok(())
    });
}

#[test]
fn test_activation_queue_efficiency_scaled() {
    let mut test_case = RegistryUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/registry_updates/pyspec_tests/activation_queue_efficiency_scaled");

    test_case.execute(|state, context| {
        spec::process_registry_updates(state, context);
        Ok(())
    });
}

#[test]
fn test_activation_queue_no_activation_no_finality() {
    let mut test_case = RegistryUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/registry_updates/pyspec_tests/activation_queue_no_activation_no_finality");

    test_case.execute(|state, context| {
        spec::process_registry_updates(state, context);
        Ok(())
    });
}

#[test]
fn test_activation_queue_sorting() {
    let mut test_case = RegistryUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/registry_updates/pyspec_tests/activation_queue_sorting");

    test_case.execute(|state, context| {
        spec::process_registry_updates(state, context);
        Ok(())
    });
}

#[test]
fn test_activation_queue_to_activated_if_finalized() {
    let mut test_case = RegistryUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/registry_updates/pyspec_tests/activation_queue_to_activated_if_finalized");

    test_case.execute(|state, context| {
        spec::process_registry_updates(state, context);
        Ok(())
    });
}

#[test]
fn test_add_to_activation_queue() {
    let mut test_case = RegistryUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/registry_updates/pyspec_tests/add_to_activation_queue");

    test_case.execute(|state, context| {
        spec::process_registry_updates(state, context);
        Ok(())
    });
}

#[test]
fn test_ejection() {
    let mut test_case = RegistryUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/registry_updates/pyspec_tests/ejection");

    test_case.execute(|state, context| {
        spec::process_registry_updates(state, context);
        Ok(())
    });
}

#[test]
fn test_ejection_past_churn_limit_min() {
    let mut test_case = RegistryUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/registry_updates/pyspec_tests/ejection_past_churn_limit_min");

    test_case.execute(|state, context| {
        spec::process_registry_updates(state, context);
        Ok(())
    });
}

#[test]
fn test_ejection_past_churn_limit_scaled() {
    let mut test_case = RegistryUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/registry_updates/pyspec_tests/ejection_past_churn_limit_scaled");

    test_case.execute(|state, context| {
        spec::process_registry_updates(state, context);
        Ok(())
    });
}
