// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::sanity::BlocksTestCase;
use ethereum_consensus::bellatrix::minimal as spec;
use ssz_rs::prelude::*;

#[test]
fn test_attestation() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/attestation",
    );
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_attester_slashing() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/attester_slashing",
    );
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_balance_driven_status_transitions() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/balance_driven_status_transitions");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_deposit_in_block() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/deposit_in_block",
    );
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_deposit_top_up() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/deposit_top_up",
    );
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_double_same_proposer_slashings_same_block() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/double_same_proposer_slashings_same_block");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_double_similar_proposer_slashings_same_block() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/double_similar_proposer_slashings_same_block");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_double_validator_exit_same_block() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/double_validator_exit_same_block");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_duplicate_attester_slashing() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/duplicate_attester_slashing");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_empty_block_transition() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/empty_block_transition");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_empty_block_transition_large_validator_set() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/empty_block_transition_large_validator_set");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_empty_block_transition_no_tx() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/empty_block_transition_no_tx");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_empty_epoch_transition() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/empty_epoch_transition");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_empty_epoch_transition_large_validator_set() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/empty_epoch_transition_large_validator_set");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_empty_epoch_transition_not_finalizing() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/empty_epoch_transition_not_finalizing");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_empty_sync_committee_committee() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/empty_sync_committee_committee");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_empty_sync_committee_committee_genesis() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/empty_sync_committee_committee_genesis");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_eth_1_data_votes_consensus() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/eth1_data_votes_consensus");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_eth_1_data_votes_no_consensus() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/eth1_data_votes_no_consensus");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_expected_deposit_in_block() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/expected_deposit_in_block");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_full_random_operations_0() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/full_random_operations_0");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_full_random_operations_1() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/full_random_operations_1");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_full_random_operations_2() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/full_random_operations_2");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_full_random_operations_3() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/full_random_operations_3");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_full_sync_committee_committee() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/full_sync_committee_committee");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_full_sync_committee_committee_genesis() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/full_sync_committee_committee_genesis");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_half_sync_committee_committee() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/half_sync_committee_committee");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_half_sync_committee_committee_genesis() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/half_sync_committee_committee_genesis");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_high_proposer_index() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/high_proposer_index");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_historical_batch() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/historical_batch",
    );
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_inactivity_scores_full_participation_leaking() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/inactivity_scores_full_participation_leaking");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_inactivity_scores_leaking() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/inactivity_scores_leaking");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_invalid_block_sig() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/invalid_block_sig",
    );
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_invalid_proposer_index_sig_from_expected_proposer() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/invalid_proposer_index_sig_from_expected_proposer");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_invalid_proposer_index_sig_from_proposer_index() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/invalid_proposer_index_sig_from_proposer_index");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_invalid_state_root() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/invalid_state_root");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_is_execution_enabled_false() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/is_execution_enabled_false");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_multiple_attester_slashings_no_overlap() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/multiple_attester_slashings_no_overlap");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_multiple_attester_slashings_partial_overlap() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/multiple_attester_slashings_partial_overlap");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_multiple_different_proposer_slashings_same_block() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/multiple_different_proposer_slashings_same_block");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_multiple_different_validator_exits_same_block() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/multiple_different_validator_exits_same_block");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_parent_from_same_slot() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/parent_from_same_slot");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_prev_slot_block_transition() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/prev_slot_block_transition");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_proposer_after_inactive_index() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/proposer_after_inactive_index");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_proposer_self_slashing() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/proposer_self_slashing");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_proposer_slashing() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/proposer_slashing",
    );
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_same_slot_block_transition() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/same_slot_block_transition");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_skipped_slots() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/skipped_slots",
    );
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_slash_and_exit_diff_index() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/slash_and_exit_diff_index");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_slash_and_exit_same_index() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from("consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/slash_and_exit_same_index");
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_voluntary_exit() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/voluntary_exit",
    );
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}

#[test]
fn test_zero_block_sig() {
    let mut test_case = BlocksTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/minimal/bellatrix/sanity/blocks/pyspec_tests/zero_block_sig",
    );
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}
