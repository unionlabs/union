// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::epoch_processing::SlashingsResetTestCase;
use ethereum_consensus::bellatrix::mainnet as spec;
use ssz_rs::prelude::*;

#[test]
fn test_flush_slashings() {
    let mut test_case = SlashingsResetTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/bellatrix/epoch_processing/slashings_reset/pyspec_tests/flush_slashings");

    test_case.execute(|state, context| {
        spec::process_slashings_reset(state, context);
        Ok(())
    });
}
