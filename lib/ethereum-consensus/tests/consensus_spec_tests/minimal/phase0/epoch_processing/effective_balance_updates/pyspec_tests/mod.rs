// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::epoch_processing::EffectiveBalanceUpdatesTestCase;
use ethereum_consensus::phase0::minimal as spec;
use ssz_rs::prelude::*;

#[test]
fn test_effective_balance_hysteresis() {
    let mut test_case = EffectiveBalanceUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/phase0/epoch_processing/effective_balance_updates/pyspec_tests/effective_balance_hysteresis");

    test_case.execute(|state, context| {
        spec::process_effective_balance_updates(state, context);
        Ok(())
    });
}
