use ethereum_consensus::altair::mainnet as altair;
use ethereum_consensus::bellatrix::mainnet as bellatrix;
use ethereum_consensus::phase0::mainnet as phase0;
use ethereum_consensus::state_transition::{Context, Validation};

fn main() {
    let context = Context::for_mainnet();

    // phase0 transition
    let mut state = phase0::BeaconState::default();
    let mut signed_block = phase0::SignedBeaconBlock::default();
    signed_block.message.slot = 1;

    let previous_epoch = phase0::get_previous_epoch(&state, &context);
    dbg!(previous_epoch);

    let _ = phase0::state_transition(&mut state, &mut signed_block, Validation::Enabled, &context);
    dbg!(state.fork);

    // altair transition
    let mut state = altair::BeaconState::default();
    let mut signed_block = altair::SignedBeaconBlock::default();
    state.slot = 32;
    signed_block.message.slot = 33;

    let current_epoch = altair::get_current_epoch(&state, &context);
    dbg!(current_epoch);

    let _ = altair::state_transition(&mut state, &mut signed_block, Validation::Enabled, &context);
    dbg!(state.fork);

    // bellatrix transition
    let mut state = bellatrix::BeaconState::default();
    let mut signed_block = bellatrix::SignedBeaconBlock::default();
    state.slot = 32;
    signed_block.message.slot = 33;

    let current_epoch = bellatrix::get_current_epoch(&state, &context);
    dbg!(current_epoch);

    let execution_engine = bellatrix::NoOpExecutionEngine;
    let _ = bellatrix::state_transition(
        &mut state,
        &mut signed_block,
        execution_engine,
        Validation::Enabled,
        &context,
    );
    dbg!(state.fork);
}
