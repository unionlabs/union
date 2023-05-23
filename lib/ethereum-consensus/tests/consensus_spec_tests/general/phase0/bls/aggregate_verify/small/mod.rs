// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::bls::AggregateVerifyTestCase;
use crate::test_utils::TestCase;

#[test]
fn test_aggregate_verify_infinity_pubkey() {
    let  test_case = AggregateVerifyTestCase::from("consensus-spec-tests/tests/general/phase0/bls/aggregate_verify/small/aggregate_verify_infinity_pubkey");

    test_case.execute();
}

#[test]
fn test_aggregate_verify_na_pubkeys_and_infinity_signature() {
    let  test_case = AggregateVerifyTestCase::from("consensus-spec-tests/tests/general/phase0/bls/aggregate_verify/small/aggregate_verify_na_pubkeys_and_infinity_signature");

    test_case.execute();
}

#[test]
fn test_aggregate_verify_na_pubkeys_and_zero_signature() {
    let  test_case = AggregateVerifyTestCase::from("consensus-spec-tests/tests/general/phase0/bls/aggregate_verify/small/aggregate_verify_na_pubkeys_and_zero_signature");

    test_case.execute();
}

#[test]
fn test_aggregate_verify_tampered_signature() {
    let  test_case = AggregateVerifyTestCase::from("consensus-spec-tests/tests/general/phase0/bls/aggregate_verify/small/aggregate_verify_tampered_signature");

    test_case.execute();
}

#[test]
fn test_aggregate_verify_valid() {
    let  test_case = AggregateVerifyTestCase::from("consensus-spec-tests/tests/general/phase0/bls/aggregate_verify/small/aggregate_verify_valid");

    test_case.execute();
}
