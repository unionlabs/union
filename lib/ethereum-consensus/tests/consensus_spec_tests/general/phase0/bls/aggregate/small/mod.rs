// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::bls::AggregateTestCase;
use crate::test_utils::TestCase;

#[test]
fn test_aggregate_0_x_0000000000000000000000000000000000000000000000000000000000000000() {
    let  test_case = AggregateTestCase::from("consensus-spec-tests/tests/general/phase0/bls/aggregate/small/aggregate_0x0000000000000000000000000000000000000000000000000000000000000000");

    test_case.execute();
}

#[test]
fn test_aggregate_0_x_5656565656565656565656565656565656565656565656565656565656565656() {
    let  test_case = AggregateTestCase::from("consensus-spec-tests/tests/general/phase0/bls/aggregate/small/aggregate_0x5656565656565656565656565656565656565656565656565656565656565656");

    test_case.execute();
}

#[test]
fn test_aggregate_0_xabababababababababababababababababababababababababababababababab() {
    let  test_case = AggregateTestCase::from("consensus-spec-tests/tests/general/phase0/bls/aggregate/small/aggregate_0xabababababababababababababababababababababababababababababababab");

    test_case.execute();
}

#[test]
fn test_aggregate_infinity_signature() {
    let  test_case = AggregateTestCase::from("consensus-spec-tests/tests/general/phase0/bls/aggregate/small/aggregate_infinity_signature");

    test_case.execute();
}

#[test]
fn test_aggregate_na_signatures() {
    let test_case = AggregateTestCase::from(
        "consensus-spec-tests/tests/general/phase0/bls/aggregate/small/aggregate_na_signatures",
    );

    test_case.execute();
}
