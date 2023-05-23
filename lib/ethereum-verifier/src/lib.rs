mod context;
mod error;
mod utils;
mod verify;

pub use context::*;
pub use error::*;
pub use utils::*;
pub use verify::*;

#[cfg(feature = "config-minimal")]
pub(crate) use minimal::*;

#[cfg(feature = "config-minimal")]
pub(crate) mod minimal {
    pub use ethereum_consensus::{
        altair::minimal::{EPOCHS_PER_SYNC_COMMITTEE_PERIOD, MIN_SYNC_COMMITTEE_PARTICIPANTS},
        capella::minimal::*,
        configs::minimal::*,
        phase0::minimal::SLOTS_PER_EPOCH,
    };
}

#[cfg(feature = "config-mainnet")]
pub(crate) use mainnet::*;

#[cfg(feature = "config-mainnet")]
pub(crate) mod mainnet {
    pub use ethereum_consensus::{
        altair::mainnet::{EPOCHS_PER_SYNC_COMMITTEE_PERIOD, MIN_SYNC_COMMITTEE_PARTICIPANTS},
        capella::mainnet::*,
        configs::mainnet::*,
        phase0::mainnet::SLOTS_PER_EPOCH,
    };
}

pub const FINALIZED_ROOT_INDEX: usize = 105;
pub const FINALIZED_ROOT_SUBTREE_INDEX: usize = 41;
pub const FINALIZED_ROOT_DEPTH: usize = 6;
pub const EXECUTION_PAYLOAD_INDEX: usize = 25;
pub const EXECUTION_PAYLOAD_DEPTH: usize = 4;
pub const NEXT_SYNC_COMMITTEE_SUBTREE_INDEX: usize = 23;
