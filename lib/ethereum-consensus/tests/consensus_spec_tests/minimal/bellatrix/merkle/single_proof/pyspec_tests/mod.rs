// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::merkle::SingleProofTestCase;

#[test]
fn test_finality_root_merkle_proof() {
    let  test_case = SingleProofTestCase::from("consensus-spec-tests/tests/minimal/bellatrix/merkle/single_proof/pyspec_tests/finality_root_merkle_proof");

    test_case.execute();
}

#[test]
fn test_next_sync_committee_merkle_proof() {
    let  test_case = SingleProofTestCase::from("consensus-spec-tests/tests/minimal/bellatrix/merkle/single_proof/pyspec_tests/next_sync_committee_merkle_proof");

    test_case.execute();
}
