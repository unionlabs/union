// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::epoch_processing::RandaoMixesResetTestCase;
use ethereum_consensus::bellatrix::minimal as spec;
use ssz_rs::prelude::*;

#[test]
fn test_updated_randao_mixes() {
    let mut test_case = RandaoMixesResetTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/bellatrix/epoch_processing/randao_mixes_reset/pyspec_tests/updated_randao_mixes");

    test_case.execute(|state, context| {
        spec::process_randao_mixes_reset(state, context);
        Ok(())
    });
}
