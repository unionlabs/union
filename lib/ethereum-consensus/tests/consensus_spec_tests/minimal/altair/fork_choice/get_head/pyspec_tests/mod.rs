// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::fork_choice::GetHeadTestCase;

#[test]
fn test_chain_no_attestations() {
    let  test_case = GetHeadTestCase::from("consensus-spec-tests/tests/minimal/altair/fork_choice/get_head/pyspec_tests/chain_no_attestations");

    test_case.execute();
}

#[test]
fn test_filtered_block_tree() {
    let  test_case = GetHeadTestCase::from("consensus-spec-tests/tests/minimal/altair/fork_choice/get_head/pyspec_tests/filtered_block_tree");

    test_case.execute();
}

#[test]
fn test_genesis() {
    let test_case = GetHeadTestCase::from(
        "consensus-spec-tests/tests/minimal/altair/fork_choice/get_head/pyspec_tests/genesis",
    );

    test_case.execute();
}

#[test]
fn test_proposer_boost_correct_head() {
    let  test_case = GetHeadTestCase::from("consensus-spec-tests/tests/minimal/altair/fork_choice/get_head/pyspec_tests/proposer_boost_correct_head");

    test_case.execute();
}

#[test]
fn test_shorter_chain_but_heavier_weight() {
    let  test_case = GetHeadTestCase::from("consensus-spec-tests/tests/minimal/altair/fork_choice/get_head/pyspec_tests/shorter_chain_but_heavier_weight");

    test_case.execute();
}

#[test]
fn test_split_tie_breaker_no_attestations() {
    let  test_case = GetHeadTestCase::from("consensus-spec-tests/tests/minimal/altair/fork_choice/get_head/pyspec_tests/split_tie_breaker_no_attestations");

    test_case.execute();
}
