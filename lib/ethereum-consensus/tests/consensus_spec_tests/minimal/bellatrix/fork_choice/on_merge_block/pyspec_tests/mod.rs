// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::fork_choice::OnMergeBlockTestCase;

#[test]
fn test_all_valid() {
    let  test_case = OnMergeBlockTestCase::from("consensus-spec-tests/tests/minimal/bellatrix/fork_choice/on_merge_block/pyspec_tests/all_valid");

    test_case.execute();
}

#[test]
fn test_block_lookup_failed() {
    let  test_case = OnMergeBlockTestCase::from("consensus-spec-tests/tests/minimal/bellatrix/fork_choice/on_merge_block/pyspec_tests/block_lookup_failed");

    test_case.execute();
}

#[test]
fn test_too_early_for_merge() {
    let  test_case = OnMergeBlockTestCase::from("consensus-spec-tests/tests/minimal/bellatrix/fork_choice/on_merge_block/pyspec_tests/too_early_for_merge");

    test_case.execute();
}

#[test]
fn test_too_late_for_merge() {
    let  test_case = OnMergeBlockTestCase::from("consensus-spec-tests/tests/minimal/bellatrix/fork_choice/on_merge_block/pyspec_tests/too_late_for_merge");

    test_case.execute();
}
