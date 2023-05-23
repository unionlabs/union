// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::epoch_processing::ParticipationRecordUpdatesTestCase;
use ethereum_consensus::phase0::mainnet as spec;
use ssz_rs::prelude::*;

#[test]
fn test_updated_participation_record() {
    let mut test_case = ParticipationRecordUpdatesTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/phase0/epoch_processing/participation_record_updates/pyspec_tests/updated_participation_record");

    test_case.execute(|state, context| {
        spec::process_participation_record_updates(state);
        Ok(())
    });
}
