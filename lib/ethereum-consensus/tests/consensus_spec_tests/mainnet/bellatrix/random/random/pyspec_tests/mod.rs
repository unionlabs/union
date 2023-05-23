// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::random::RandomTestCase;
use ethereum_consensus::bellatrix::mainnet as spec;
use ssz_rs::prelude::*;

#[test]
fn test_randomized_0() {
    let mut test_case = RandomTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/random/random/pyspec_tests/randomized_0",
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
fn test_randomized_1() {
    let mut test_case = RandomTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/random/random/pyspec_tests/randomized_1",
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
fn test_randomized_10() {
    let mut test_case = RandomTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/random/random/pyspec_tests/randomized_10",
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
fn test_randomized_11() {
    let mut test_case = RandomTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/random/random/pyspec_tests/randomized_11",
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
fn test_randomized_12() {
    let mut test_case = RandomTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/random/random/pyspec_tests/randomized_12",
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
fn test_randomized_13() {
    let mut test_case = RandomTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/random/random/pyspec_tests/randomized_13",
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
fn test_randomized_14() {
    let mut test_case = RandomTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/random/random/pyspec_tests/randomized_14",
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
fn test_randomized_15() {
    let mut test_case = RandomTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/random/random/pyspec_tests/randomized_15",
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
fn test_randomized_2() {
    let mut test_case = RandomTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/random/random/pyspec_tests/randomized_2",
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
fn test_randomized_3() {
    let mut test_case = RandomTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/random/random/pyspec_tests/randomized_3",
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
fn test_randomized_4() {
    let mut test_case = RandomTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/random/random/pyspec_tests/randomized_4",
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
fn test_randomized_5() {
    let mut test_case = RandomTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/random/random/pyspec_tests/randomized_5",
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
fn test_randomized_6() {
    let mut test_case = RandomTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/random/random/pyspec_tests/randomized_6",
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
fn test_randomized_7() {
    let mut test_case = RandomTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/random/random/pyspec_tests/randomized_7",
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
fn test_randomized_8() {
    let mut test_case = RandomTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/random/random/pyspec_tests/randomized_8",
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
fn test_randomized_9() {
    let mut test_case = RandomTestCase::<spec::BeaconState, spec::SignedBeaconBlock>::from(
        "consensus-spec-tests/tests/mainnet/bellatrix/random/random/pyspec_tests/randomized_9",
    );
    let execution_engine = spec::NoOpExecutionEngine;
    test_case.execute(|state, blocks, validation, context| {
        for block in blocks.iter_mut() {
            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
        }
        Ok(())
    });
}
