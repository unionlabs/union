// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::finality::FinalityTestCase;
use ethereum_consensus::altair::mainnet as spec;
use ssz_rs::prelude::*;

#[test]
fn test_finality_no_updates_at_genesis() {
    let mut test_case = FinalityTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/mainnet/altair/finality/finality/pyspec_tests/finality_no_updates_at_genesis");

    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_finality_rule_1() {
    let mut test_case = FinalityTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/mainnet/altair/finality/finality/pyspec_tests/finality_rule_1",
    );

    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_finality_rule_2() {
    let mut test_case = FinalityTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/mainnet/altair/finality/finality/pyspec_tests/finality_rule_2",
    );

    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_finality_rule_3() {
    let mut test_case = FinalityTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/mainnet/altair/finality/finality/pyspec_tests/finality_rule_3",
    );

    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_finality_rule_4() {
    let mut test_case = FinalityTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/mainnet/altair/finality/finality/pyspec_tests/finality_rule_4",
    );

    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, validation, context)?;
        }
        Ok(())
    });
}
