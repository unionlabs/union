use near_account_id::AccountId;
use near_primitives::views::{BlockHeaderView, BlockView};

use crate::types::{ChunkHeader, NearToken};
use crate::{BlockHeight, CryptoHash};

/// Struct containing information on block coming from the network
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Block {
    author: AccountId,
    header: BlockHeader,
    chunks: Vec<ChunkHeader>,
}

impl Block {
    /// The account id of the block author.
    pub fn author(&self) -> &AccountId {
        &self.author
    }

    /// The block header info.
    pub fn header(&self) -> &BlockHeader {
        &self.header
    }

    /// The list of chunks in this block.
    pub fn chunks(&self) -> &[ChunkHeader] {
        &self.chunks
    }

    /// The block timestamp in nanoseconds.
    pub fn timestamp(&self) -> u64 {
        self.header.timestamp_nanosec
    }

    /// Current height of this block.
    pub fn height(&self) -> BlockHeight {
        self.header.height
    }

    /// The hash of the block itself.
    pub fn hash(&self) -> &CryptoHash {
        &self.header.hash
    }

    /// The id of an epoch this block belongs to.
    pub fn epoch_id(&self) -> &CryptoHash {
        &self.header.epoch_id
    }
}

/// The block header info. This is a non-exhaustive list of items that
/// could be present in a block header. More can be added in the future.
///
/// NOTE: For maintainability purposes, some items have been excluded. If required,
/// please submit an issue to [workspaces](https://github.com/near/workspaces-rs/issues).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockHeader {
    height: BlockHeight,
    epoch_id: CryptoHash,
    next_epoch_id: CryptoHash,
    hash: CryptoHash,
    prev_hash: CryptoHash,
    timestamp_nanosec: u64,
    random_value: CryptoHash,
    gas_price: NearToken,
    block_ordinal: Option<u64>,
    total_supply: NearToken,
    last_final_block: CryptoHash,
    last_ds_final_block: CryptoHash,
    next_bp_hash: CryptoHash,
    latest_protocol_version: u32,

    prev_state_root: CryptoHash,
    chunk_receipts_root: CryptoHash,
    chunk_headers_root: CryptoHash,
    chunk_tx_root: CryptoHash,
    outcome_root: CryptoHash,
    challenges_root: CryptoHash,
    block_merkle_root: CryptoHash,
}

impl BlockHeader {
    /// Current height of this block.
    pub fn height(&self) -> BlockHeight {
        self.height
    }

    /// The id of an epoch this block belongs to.
    pub fn epoch_id(&self) -> &CryptoHash {
        &self.epoch_id
    }

    /// The next epoch id.
    pub fn next_epoch_id(&self) -> &CryptoHash {
        &self.next_epoch_id
    }

    /// The hash of the block itself.
    pub fn hash(&self) -> &CryptoHash {
        &self.hash
    }

    /// The hash of the previous block.
    pub fn prev_hash(&self) -> &CryptoHash {
        &self.prev_hash
    }

    /// The block timestamp in nanoseconds.
    pub fn timestamp_nanosec(&self) -> u64 {
        self.timestamp_nanosec
    }

    /// The random value of the block.
    pub fn random_value(&self) -> &CryptoHash {
        &self.random_value
    }

    /// The gas price of the block.
    pub fn gas_price(&self) -> NearToken {
        self.gas_price
    }

    /// The block ordinal.
    pub fn block_ordinal(&self) -> Option<u64> {
        self.block_ordinal
    }

    /// The total supply balance of the block.
    pub fn total_supply(&self) -> NearToken {
        self.total_supply
    }

    /// The last final block hash.
    pub fn last_final_block(&self) -> &CryptoHash {
        &self.last_final_block
    }

    /// The last ds final block hash.
    pub fn last_ds_final_block(&self) -> &CryptoHash {
        &self.last_ds_final_block
    }

    /// The next bp hash.
    pub fn next_bp_hash(&self) -> &CryptoHash {
        &self.next_bp_hash
    }

    /// The latest protocol version.
    pub fn latest_protocol_version(&self) -> u32 {
        self.latest_protocol_version
    }

    /// The previous state root.
    pub fn prev_state_root(&self) -> &CryptoHash {
        &self.prev_state_root
    }

    /// The chunk receipts root.
    pub fn chunk_receipts_root(&self) -> &CryptoHash {
        &self.chunk_receipts_root
    }

    /// The chunk headers root.
    pub fn chunk_headers_root(&self) -> &CryptoHash {
        &self.chunk_headers_root
    }

    /// The chunk tx root.
    pub fn chunk_tx_root(&self) -> &CryptoHash {
        &self.chunk_tx_root
    }

    /// The outcome root.
    pub fn outcome_root(&self) -> &CryptoHash {
        &self.outcome_root
    }

    /// The challenges root.
    pub fn challenges_root(&self) -> &CryptoHash {
        &self.challenges_root
    }

    /// The block merkle root.
    pub fn block_merkle_root(&self) -> &CryptoHash {
        &self.block_merkle_root
    }
}

impl From<BlockView> for Block {
    fn from(view: BlockView) -> Self {
        Self {
            author: view.author,
            header: view.header.into(),
            chunks: view.chunks.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<BlockHeaderView> for BlockHeader {
    fn from(header_view: BlockHeaderView) -> Self {
        Self {
            height: header_view.height,
            epoch_id: header_view.epoch_id.into(),
            next_epoch_id: header_view.next_epoch_id.into(),
            hash: header_view.hash.into(),
            prev_hash: header_view.prev_hash.into(),
            timestamp_nanosec: header_view.timestamp_nanosec,
            random_value: header_view.random_value.into(),
            gas_price: NearToken::from_yoctonear(header_view.gas_price),
            block_ordinal: header_view.block_ordinal,
            total_supply: NearToken::from_yoctonear(header_view.total_supply),
            last_final_block: header_view.last_final_block.into(),
            last_ds_final_block: header_view.last_ds_final_block.into(),
            next_bp_hash: header_view.next_bp_hash.into(),
            latest_protocol_version: header_view.latest_protocol_version,

            prev_state_root: header_view.prev_state_root.into(),
            chunk_receipts_root: header_view.chunk_receipts_root.into(),
            chunk_headers_root: header_view.chunk_headers_root.into(),
            chunk_tx_root: header_view.chunk_tx_root.into(),
            outcome_root: header_view.outcome_root.into(),
            challenges_root: header_view.challenges_root.into(),
            block_merkle_root: header_view.block_merkle_root.into(),
        }
    }
}
