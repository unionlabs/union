use near_account_id::AccountId;
use near_gas::NearGas;
use near_primitives::views::{ChunkHeaderView, ChunkView};

use crate::types::{Gas, NearToken, ShardId};
use crate::{BlockHeight, CryptoHash};

// Chunk object associated to a chunk on chain. This provides info about what
// current state of a chunk is like.
#[derive(Debug, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub struct Chunk {
    pub author: AccountId,
    pub header: ChunkHeader,
}

/// The header belonging to a [`Chunk`]. This is a non-exhaustive list of
/// members belonging to a Chunk, where newer fields can be added in the future.
///
/// NOTE: For maintainability purposes, some items have been excluded. If required,
/// please submit an issue to [workspaces](https://github.com/near/workspaces-rs/issues).
#[derive(Debug, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub struct ChunkHeader {
    pub chunk_hash: CryptoHash,
    pub prev_block_hash: CryptoHash,
    pub height_created: BlockHeight,
    pub height_included: BlockHeight,
    pub shard_id: ShardId,
    pub gas_used: Gas,
    pub gas_limit: Gas,
    pub balance_burnt: NearToken,

    pub tx_root: CryptoHash,
    pub outcome_root: CryptoHash,
    pub prev_state_root: CryptoHash,
    pub outgoing_receipts_root: CryptoHash,
    pub encoded_merkle_root: CryptoHash,
    pub encoded_length: u64,
}

impl From<ChunkView> for Chunk {
    fn from(view: ChunkView) -> Self {
        Self {
            author: view.author,
            header: view.header.into(),
        }
    }
}

impl From<ChunkHeaderView> for ChunkHeader {
    fn from(view: ChunkHeaderView) -> Self {
        Self {
            chunk_hash: view.chunk_hash.into(),
            prev_block_hash: view.prev_block_hash.into(),
            height_created: view.height_created,
            height_included: view.height_included,
            shard_id: view.shard_id,
            gas_used: NearGas::from_gas(view.gas_used),
            gas_limit: NearGas::from_gas(view.gas_limit),
            balance_burnt: NearToken::from_yoctonear(view.balance_burnt),
            tx_root: view.tx_root.into(),
            outcome_root: view.outcome_root.into(),
            prev_state_root: view.prev_state_root.into(),
            outgoing_receipts_root: view.outgoing_receipts_root.into(),
            encoded_merkle_root: view.encoded_merkle_root.into(),
            encoded_length: view.encoded_length,
        }
    }
}

impl Chunk {
    /// The hash of the chunk itself.
    pub fn hash(&self) -> &CryptoHash {
        &self.header.chunk_hash
    }

    /// Which specific shard this chunk belongs to.
    pub fn shard_id(&self) -> ShardId {
        self.header.shard_id
    }
}
