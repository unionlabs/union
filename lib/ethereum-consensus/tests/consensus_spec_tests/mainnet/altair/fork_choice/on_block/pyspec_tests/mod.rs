// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::fork_choice::OnBlockTestCase;

#[test]
fn test_basic() {
    let test_case = OnBlockTestCase::from(
        "consensus-spec-tests/tests/mainnet/altair/fork_choice/on_block/pyspec_tests/basic",
    );

    test_case.execute();
}

#[test]
fn test_on_block_bad_parent_root() {
    let  test_case = OnBlockTestCase::from("consensus-spec-tests/tests/mainnet/altair/fork_choice/on_block/pyspec_tests/on_block_bad_parent_root");

    test_case.execute();
}

#[test]
fn test_on_block_future_block() {
    let  test_case = OnBlockTestCase::from("consensus-spec-tests/tests/mainnet/altair/fork_choice/on_block/pyspec_tests/on_block_future_block");

    test_case.execute();
}

#[test]
fn test_proposer_boost() {
    let  test_case = OnBlockTestCase::from("consensus-spec-tests/tests/mainnet/altair/fork_choice/on_block/pyspec_tests/proposer_boost");

    test_case.execute();
}

#[test]
fn test_proposer_boost_root_same_slot_untimely_block() {
    let  test_case = OnBlockTestCase::from("consensus-spec-tests/tests/mainnet/altair/fork_choice/on_block/pyspec_tests/proposer_boost_root_same_slot_untimely_block");

    test_case.execute();
}
