// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::epoch_processing::Eth1DataResetTestCase;
use ethereum_consensus::altair::minimal as spec;
use ssz_rs::prelude::*;

#[test]
fn test_eth_1_vote_no_reset() {
    let mut test_case = Eth1DataResetTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/eth1_data_reset/pyspec_tests/eth1_vote_no_reset");

    test_case.execute(|state, context| {
        spec::process_eth1_data_reset(state, context);
        Ok(())
    });
}

#[test]
fn test_eth_1_vote_reset() {
    let mut test_case = Eth1DataResetTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/minimal/altair/epoch_processing/eth1_data_reset/pyspec_tests/eth1_vote_reset");

    test_case.execute(|state, context| {
        spec::process_eth1_data_reset(state, context);
        Ok(())
    });
}
