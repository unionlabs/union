// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::fork_choice::OnBlockTestCase;

#[test]
fn test_basic() {
    let test_case = OnBlockTestCase::from(
        "consensus-spec-tests/tests/minimal/bellatrix/fork_choice/on_block/pyspec_tests/basic",
    );

    test_case.execute();
}

#[test]
fn test_new_finalized_slot_is_justified_checkpoint_ancestor() {
    let  test_case = OnBlockTestCase::from("consensus-spec-tests/tests/minimal/bellatrix/fork_choice/on_block/pyspec_tests/new_finalized_slot_is_justified_checkpoint_ancestor");

    test_case.execute();
}

#[test]
fn test_new_finalized_slot_is_not_justified_checkpoint_ancestor() {
    let  test_case = OnBlockTestCase::from("consensus-spec-tests/tests/minimal/bellatrix/fork_choice/on_block/pyspec_tests/new_finalized_slot_is_not_justified_checkpoint_ancestor");

    test_case.execute();
}

#[test]
fn test_new_justified_is_later_than_store_justified() {
    let  test_case = OnBlockTestCase::from("consensus-spec-tests/tests/minimal/bellatrix/fork_choice/on_block/pyspec_tests/new_justified_is_later_than_store_justified");

    test_case.execute();
}

#[test]
fn test_on_block_bad_parent_root() {
    let  test_case = OnBlockTestCase::from("consensus-spec-tests/tests/minimal/bellatrix/fork_choice/on_block/pyspec_tests/on_block_bad_parent_root");

    test_case.execute();
}

#[test]
fn test_on_block_before_finalized() {
    let  test_case = OnBlockTestCase::from("consensus-spec-tests/tests/minimal/bellatrix/fork_choice/on_block/pyspec_tests/on_block_before_finalized");

    test_case.execute();
}

#[test]
fn test_on_block_checkpoints() {
    let  test_case = OnBlockTestCase::from("consensus-spec-tests/tests/minimal/bellatrix/fork_choice/on_block/pyspec_tests/on_block_checkpoints");

    test_case.execute();
}

#[test]
fn test_on_block_finalized_skip_slots() {
    let  test_case = OnBlockTestCase::from("consensus-spec-tests/tests/minimal/bellatrix/fork_choice/on_block/pyspec_tests/on_block_finalized_skip_slots");

    test_case.execute();
}

#[test]
fn test_on_block_finalized_skip_slots_not_in_skip_chain() {
    let  test_case = OnBlockTestCase::from("consensus-spec-tests/tests/minimal/bellatrix/fork_choice/on_block/pyspec_tests/on_block_finalized_skip_slots_not_in_skip_chain");

    test_case.execute();
}

#[test]
fn test_on_block_future_block() {
    let  test_case = OnBlockTestCase::from("consensus-spec-tests/tests/minimal/bellatrix/fork_choice/on_block/pyspec_tests/on_block_future_block");

    test_case.execute();
}

#[test]
fn test_on_block_outside_safe_slots_but_finality() {
    let  test_case = OnBlockTestCase::from("consensus-spec-tests/tests/minimal/bellatrix/fork_choice/on_block/pyspec_tests/on_block_outside_safe_slots_but_finality");

    test_case.execute();
}

#[test]
fn test_on_block_update_justified_checkpoint_within_safe_slots() {
    let  test_case = OnBlockTestCase::from("consensus-spec-tests/tests/minimal/bellatrix/fork_choice/on_block/pyspec_tests/on_block_update_justified_checkpoint_within_safe_slots");

    test_case.execute();
}

#[test]
fn test_proposer_boost() {
    let  test_case = OnBlockTestCase::from("consensus-spec-tests/tests/minimal/bellatrix/fork_choice/on_block/pyspec_tests/proposer_boost");

    test_case.execute();
}

#[test]
fn test_proposer_boost_root_same_slot_untimely_block() {
    let  test_case = OnBlockTestCase::from("consensus-spec-tests/tests/minimal/bellatrix/fork_choice/on_block/pyspec_tests/proposer_boost_root_same_slot_untimely_block");

    test_case.execute();
}
