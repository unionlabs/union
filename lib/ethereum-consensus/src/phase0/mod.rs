//! This module provides an implementation of the `phase0` fork
//! of the consensus spec. The primary entrypoints should be one of
//! the "presets" like `mainnet` or `minimal`.
mod beacon_block;
mod beacon_state;
mod block_processing;
mod epoch_processing;
mod fork;
mod genesis;
mod helpers;
mod networking;
mod operations;
mod presets;
mod slot_processing;
mod state_transition;
mod validator;

pub use crate::signing::SigningData;
pub use beacon_block::*;
pub use beacon_state::*;
pub use block_processing::*;
pub use epoch_processing::*;
pub use fork::*;
pub use genesis::*;
pub use helpers::*;
pub use networking::*;
pub use operations::*;
pub use presets::Preset;
pub use slot_processing::*;
pub use state_transition::*;
pub use validator::*;

pub const BASE_REWARDS_PER_EPOCH: u64 = 4;
pub const DEPOSIT_CONTRACT_TREE_DEPTH: usize = 2usize.pow(5);
pub const JUSTIFICATION_BITS_LENGTH: usize = 4;
pub const DEPOSIT_DATA_LIST_BOUND: usize = 2usize.pow(DEPOSIT_CONTRACT_TREE_DEPTH as u32);

pub use presets::mainnet;
pub use presets::minimal;
