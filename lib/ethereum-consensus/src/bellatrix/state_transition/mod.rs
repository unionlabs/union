//! WARNING: This file was derived by the `gen-spec` utility. DO NOT EDIT MANUALLY.
pub mod block_processing;
pub mod epoch_processing;
pub mod helpers;
pub mod slot_processing;

pub use crate::bellatrix::state_transition_bellatrix::state_transition;
pub use crate::bellatrix::state_transition_bellatrix::state_transition_block_in_slot;

use ssz_rs::prelude::*;
