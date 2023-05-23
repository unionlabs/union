// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::transition::CoreTestCase;
use ethereum_consensus::altair::mainnet as spec;
use ethereum_consensus::bellatrix::mainnet::NoOpExecutionEngine;
use ethereum_consensus::phase0::mainnet as pre_spec;
use ethereum_consensus::state_transition::mainnet::{BeaconState, Executor};
use ssz_rs::prelude::*;

#[test]
fn test_normal_transition() {
    let mut test_case = CoreTestCase::<spec::BeaconState>::from(
        "consensus-spec-tests/tests/mainnet/altair/transition/core/pyspec_tests/normal_transition",
    );

    test_case.execute(
        |state: pre_spec::BeaconState,
         pre_blocks: Vec<pre_spec::SignedBeaconBlock>,
         blocks: Vec<spec::SignedBeaconBlock>,
         context| {
            let mut executor = Executor::new(state.into(), NoOpExecutionEngine, context);
            for block in pre_blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            for block in blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            match executor.state {
                BeaconState::Altair(inner) => Ok(*inner),
                _ => unreachable!(),
            }
        },
    );
}

#[test]
fn test_transition_missing_first_post_block() {
    let mut test_case = CoreTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/altair/transition/core/pyspec_tests/transition_missing_first_post_block");

    test_case.execute(
        |state: pre_spec::BeaconState,
         pre_blocks: Vec<pre_spec::SignedBeaconBlock>,
         blocks: Vec<spec::SignedBeaconBlock>,
         context| {
            let mut executor = Executor::new(state.into(), NoOpExecutionEngine, context);
            for block in pre_blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            for block in blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            match executor.state {
                BeaconState::Altair(inner) => Ok(*inner),
                _ => unreachable!(),
            }
        },
    );
}

#[test]
fn test_transition_missing_last_pre_fork_block() {
    let mut test_case = CoreTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/altair/transition/core/pyspec_tests/transition_missing_last_pre_fork_block");

    test_case.execute(
        |state: pre_spec::BeaconState,
         pre_blocks: Vec<pre_spec::SignedBeaconBlock>,
         blocks: Vec<spec::SignedBeaconBlock>,
         context| {
            let mut executor = Executor::new(state.into(), NoOpExecutionEngine, context);
            for block in pre_blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            for block in blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            match executor.state {
                BeaconState::Altair(inner) => Ok(*inner),
                _ => unreachable!(),
            }
        },
    );
}

#[test]
fn test_transition_only_blocks_post_fork() {
    let mut test_case = CoreTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/altair/transition/core/pyspec_tests/transition_only_blocks_post_fork");

    test_case.execute(
        |state: pre_spec::BeaconState,
         pre_blocks: Vec<pre_spec::SignedBeaconBlock>,
         blocks: Vec<spec::SignedBeaconBlock>,
         context| {
            let mut executor = Executor::new(state.into(), NoOpExecutionEngine, context);
            for block in pre_blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            for block in blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            match executor.state {
                BeaconState::Altair(inner) => Ok(*inner),
                _ => unreachable!(),
            }
        },
    );
}

#[test]
fn test_transition_with_activation_at_fork_epoch() {
    let mut test_case = CoreTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/altair/transition/core/pyspec_tests/transition_with_activation_at_fork_epoch");

    test_case.execute(
        |state: pre_spec::BeaconState,
         pre_blocks: Vec<pre_spec::SignedBeaconBlock>,
         blocks: Vec<spec::SignedBeaconBlock>,
         context| {
            let mut executor = Executor::new(state.into(), NoOpExecutionEngine, context);
            for block in pre_blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            for block in blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            match executor.state {
                BeaconState::Altair(inner) => Ok(*inner),
                _ => unreachable!(),
            }
        },
    );
}

#[test]
fn test_transition_with_attester_slashing_right_after_fork() {
    let mut test_case = CoreTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/altair/transition/core/pyspec_tests/transition_with_attester_slashing_right_after_fork");

    test_case.execute(
        |state: pre_spec::BeaconState,
         pre_blocks: Vec<pre_spec::SignedBeaconBlock>,
         blocks: Vec<spec::SignedBeaconBlock>,
         context| {
            let mut executor = Executor::new(state.into(), NoOpExecutionEngine, context);
            for block in pre_blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            for block in blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            match executor.state {
                BeaconState::Altair(inner) => Ok(*inner),
                _ => unreachable!(),
            }
        },
    );
}

#[test]
fn test_transition_with_attester_slashing_right_before_fork() {
    let mut test_case = CoreTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/altair/transition/core/pyspec_tests/transition_with_attester_slashing_right_before_fork");

    test_case.execute(
        |state: pre_spec::BeaconState,
         pre_blocks: Vec<pre_spec::SignedBeaconBlock>,
         blocks: Vec<spec::SignedBeaconBlock>,
         context| {
            let mut executor = Executor::new(state.into(), NoOpExecutionEngine, context);
            for block in pre_blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            for block in blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            match executor.state {
                BeaconState::Altair(inner) => Ok(*inner),
                _ => unreachable!(),
            }
        },
    );
}

#[test]
fn test_transition_with_deposit_right_after_fork() {
    let mut test_case = CoreTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/altair/transition/core/pyspec_tests/transition_with_deposit_right_after_fork");

    test_case.execute(
        |state: pre_spec::BeaconState,
         pre_blocks: Vec<pre_spec::SignedBeaconBlock>,
         blocks: Vec<spec::SignedBeaconBlock>,
         context| {
            let mut executor = Executor::new(state.into(), NoOpExecutionEngine, context);
            for block in pre_blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            for block in blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            match executor.state {
                BeaconState::Altair(inner) => Ok(*inner),
                _ => unreachable!(),
            }
        },
    );
}

#[test]
fn test_transition_with_deposit_right_before_fork() {
    let mut test_case = CoreTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/altair/transition/core/pyspec_tests/transition_with_deposit_right_before_fork");

    test_case.execute(
        |state: pre_spec::BeaconState,
         pre_blocks: Vec<pre_spec::SignedBeaconBlock>,
         blocks: Vec<spec::SignedBeaconBlock>,
         context| {
            let mut executor = Executor::new(state.into(), NoOpExecutionEngine, context);
            for block in pre_blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            for block in blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            match executor.state {
                BeaconState::Altair(inner) => Ok(*inner),
                _ => unreachable!(),
            }
        },
    );
}

#[test]
fn test_transition_with_finality() {
    let mut test_case = CoreTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/altair/transition/core/pyspec_tests/transition_with_finality");

    test_case.execute(
        |state: pre_spec::BeaconState,
         pre_blocks: Vec<pre_spec::SignedBeaconBlock>,
         blocks: Vec<spec::SignedBeaconBlock>,
         context| {
            let mut executor = Executor::new(state.into(), NoOpExecutionEngine, context);
            for block in pre_blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            for block in blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            match executor.state {
                BeaconState::Altair(inner) => Ok(*inner),
                _ => unreachable!(),
            }
        },
    );
}

#[test]
fn test_transition_with_leaking_at_fork() {
    let mut test_case = CoreTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/altair/transition/core/pyspec_tests/transition_with_leaking_at_fork");

    test_case.execute(
        |state: pre_spec::BeaconState,
         pre_blocks: Vec<pre_spec::SignedBeaconBlock>,
         blocks: Vec<spec::SignedBeaconBlock>,
         context| {
            let mut executor = Executor::new(state.into(), NoOpExecutionEngine, context);
            for block in pre_blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            for block in blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            match executor.state {
                BeaconState::Altair(inner) => Ok(*inner),
                _ => unreachable!(),
            }
        },
    );
}

#[test]
fn test_transition_with_leaking_pre_fork() {
    let mut test_case = CoreTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/altair/transition/core/pyspec_tests/transition_with_leaking_pre_fork");

    test_case.execute(
        |state: pre_spec::BeaconState,
         pre_blocks: Vec<pre_spec::SignedBeaconBlock>,
         blocks: Vec<spec::SignedBeaconBlock>,
         context| {
            let mut executor = Executor::new(state.into(), NoOpExecutionEngine, context);
            for block in pre_blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            for block in blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            match executor.state {
                BeaconState::Altair(inner) => Ok(*inner),
                _ => unreachable!(),
            }
        },
    );
}

#[test]
fn test_transition_with_no_attestations_until_after_fork() {
    let mut test_case = CoreTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/altair/transition/core/pyspec_tests/transition_with_no_attestations_until_after_fork");

    test_case.execute(
        |state: pre_spec::BeaconState,
         pre_blocks: Vec<pre_spec::SignedBeaconBlock>,
         blocks: Vec<spec::SignedBeaconBlock>,
         context| {
            let mut executor = Executor::new(state.into(), NoOpExecutionEngine, context);
            for block in pre_blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            for block in blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            match executor.state {
                BeaconState::Altair(inner) => Ok(*inner),
                _ => unreachable!(),
            }
        },
    );
}

#[test]
fn test_transition_with_non_empty_activation_queue() {
    let mut test_case = CoreTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/altair/transition/core/pyspec_tests/transition_with_non_empty_activation_queue");

    test_case.execute(
        |state: pre_spec::BeaconState,
         pre_blocks: Vec<pre_spec::SignedBeaconBlock>,
         blocks: Vec<spec::SignedBeaconBlock>,
         context| {
            let mut executor = Executor::new(state.into(), NoOpExecutionEngine, context);
            for block in pre_blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            for block in blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            match executor.state {
                BeaconState::Altair(inner) => Ok(*inner),
                _ => unreachable!(),
            }
        },
    );
}

#[test]
fn test_transition_with_one_fourth_exiting_validators_exit_at_fork() {
    let mut test_case = CoreTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/altair/transition/core/pyspec_tests/transition_with_one_fourth_exiting_validators_exit_at_fork");

    test_case.execute(
        |state: pre_spec::BeaconState,
         pre_blocks: Vec<pre_spec::SignedBeaconBlock>,
         blocks: Vec<spec::SignedBeaconBlock>,
         context| {
            let mut executor = Executor::new(state.into(), NoOpExecutionEngine, context);
            for block in pre_blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            for block in blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            match executor.state {
                BeaconState::Altair(inner) => Ok(*inner),
                _ => unreachable!(),
            }
        },
    );
}

#[test]
fn test_transition_with_proposer_slashing_right_after_fork() {
    let mut test_case = CoreTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/altair/transition/core/pyspec_tests/transition_with_proposer_slashing_right_after_fork");

    test_case.execute(
        |state: pre_spec::BeaconState,
         pre_blocks: Vec<pre_spec::SignedBeaconBlock>,
         blocks: Vec<spec::SignedBeaconBlock>,
         context| {
            let mut executor = Executor::new(state.into(), NoOpExecutionEngine, context);
            for block in pre_blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            for block in blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            match executor.state {
                BeaconState::Altair(inner) => Ok(*inner),
                _ => unreachable!(),
            }
        },
    );
}

#[test]
fn test_transition_with_proposer_slashing_right_before_fork() {
    let mut test_case = CoreTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/altair/transition/core/pyspec_tests/transition_with_proposer_slashing_right_before_fork");

    test_case.execute(
        |state: pre_spec::BeaconState,
         pre_blocks: Vec<pre_spec::SignedBeaconBlock>,
         blocks: Vec<spec::SignedBeaconBlock>,
         context| {
            let mut executor = Executor::new(state.into(), NoOpExecutionEngine, context);
            for block in pre_blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            for block in blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            match executor.state {
                BeaconState::Altair(inner) => Ok(*inner),
                _ => unreachable!(),
            }
        },
    );
}

#[test]
fn test_transition_with_random_half_participation() {
    let mut test_case = CoreTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/altair/transition/core/pyspec_tests/transition_with_random_half_participation");

    test_case.execute(
        |state: pre_spec::BeaconState,
         pre_blocks: Vec<pre_spec::SignedBeaconBlock>,
         blocks: Vec<spec::SignedBeaconBlock>,
         context| {
            let mut executor = Executor::new(state.into(), NoOpExecutionEngine, context);
            for block in pre_blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            for block in blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            match executor.state {
                BeaconState::Altair(inner) => Ok(*inner),
                _ => unreachable!(),
            }
        },
    );
}

#[test]
fn test_transition_with_random_three_quarters_participation() {
    let mut test_case = CoreTestCase::<spec::BeaconState>::from("consensus-spec-tests/tests/mainnet/altair/transition/core/pyspec_tests/transition_with_random_three_quarters_participation");

    test_case.execute(
        |state: pre_spec::BeaconState,
         pre_blocks: Vec<pre_spec::SignedBeaconBlock>,
         blocks: Vec<spec::SignedBeaconBlock>,
         context| {
            let mut executor = Executor::new(state.into(), NoOpExecutionEngine, context);
            for block in pre_blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            for block in blocks.into_iter() {
                let mut block = block.into();
                executor.apply_block(&mut block)?;
            }
            match executor.state {
                BeaconState::Altair(inner) => Ok(*inner),
                _ => unreachable!(),
            }
        },
    );
}
