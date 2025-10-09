use unionlabs_primitives::{H256, encoding::HexUnprefixed};

use crate::epoch_state::EpochState;

/// The round of a block is a consensus-internal counter, which starts with 0 and increases
/// monotonically.
pub type Round = u64;

pub type Version = u64;

/// A continuously increasing sequence number for committed blocks.
pub type BlockHeight = u64;

// Constants for the initial genesis block.
pub const GENESIS_EPOCH: u64 = 0;
pub const GENESIS_ROUND: Round = 0;
pub const GENESIS_VERSION: Version = 0;
pub const GENESIS_TIMESTAMP_USECS: u64 = 0;

/// This structure contains all the information needed for tracking a block
/// without having access to the block or its execution output state. It
/// assumes that the block is the last block executed within the ledger.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct BlockInfo {
    /// The epoch to which the block belongs.
    pub epoch: u64,
    /// The consensus protocol is executed in rounds, which monotonically increase per epoch.
    pub round: Round,
    /// The identifier (hash) of the block.
    pub id: H256<HexUnprefixed>,
    /// The accumulator root hash after executing this block.
    pub executed_state_id: H256<HexUnprefixed>,
    /// The version of the latest transaction after executing this block.
    pub version: Version,
    /// The timestamp this block was proposed by a proposer.
    pub timestamp_usecs: u64,
    /// An optional field containing the next epoch info
    pub next_epoch_state: Option<EpochState>,
}
