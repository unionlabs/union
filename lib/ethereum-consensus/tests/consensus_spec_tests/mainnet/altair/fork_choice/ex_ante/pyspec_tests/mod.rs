// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::fork_choice::ExAnteTestCase;

#[test]
fn test_ex_ante_attestations_is_greater_than_proposer_boost_with_boost() {
    let  test_case = ExAnteTestCase::from("consensus-spec-tests/tests/mainnet/altair/fork_choice/ex_ante/pyspec_tests/ex_ante_attestations_is_greater_than_proposer_boost_with_boost");

    test_case.execute();
}

#[test]
fn test_ex_ante_sandwich_with_boost_not_sufficient() {
    let  test_case = ExAnteTestCase::from("consensus-spec-tests/tests/mainnet/altair/fork_choice/ex_ante/pyspec_tests/ex_ante_sandwich_with_boost_not_sufficient");

    test_case.execute();
}

#[test]
fn test_ex_ante_sandwich_with_honest_attestation() {
    let  test_case = ExAnteTestCase::from("consensus-spec-tests/tests/mainnet/altair/fork_choice/ex_ante/pyspec_tests/ex_ante_sandwich_with_honest_attestation");

    test_case.execute();
}

#[test]
fn test_ex_ante_sandwich_without_attestations() {
    let  test_case = ExAnteTestCase::from("consensus-spec-tests/tests/mainnet/altair/fork_choice/ex_ante/pyspec_tests/ex_ante_sandwich_without_attestations");

    test_case.execute();
}

#[test]
fn test_ex_ante_vanilla() {
    let  test_case = ExAnteTestCase::from("consensus-spec-tests/tests/mainnet/altair/fork_choice/ex_ante/pyspec_tests/ex_ante_vanilla");

    test_case.execute();
}
