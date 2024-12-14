use std::{
    fmt::{Debug, Display},
    ops::Range,
};

use axum::async_trait;
use color_eyre::eyre::Report;
use futures::Stream;
use sqlx::Postgres;
use time::OffsetDateTime;
use tokio::task::JoinSet;
use tracing::{debug, error};

/// Enum representing fetch modes.
#[derive(Clone, Copy, Debug)]
pub enum FetchMode {
    /// Fetch data eagerly.
    Eager,
    /// Fetch data lazily.
    Lazy,
}

/// Represents the selection of a block (by height or last finalized).
#[derive(Debug)]
pub enum BlockSelection {
    LastFinalized,
    Height(BlockHeight),
}

impl Display for BlockSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockSelection::LastFinalized => write!(f, "last-finalized"),
            BlockSelection::Height(height) => write!(f, "{}", height),
        }
    }
}

pub type BlockHeight = u64;
pub type BlockHash = String;
pub type BlockTimestamp = OffsetDateTime;

/// Represents a range of block heights.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockRange {
    pub start_inclusive: BlockHeight,
    pub end_exclusive: BlockHeight,
}

impl BlockRange {
    /// Splits the block range into smaller chunks.
    pub fn range_chunks(self, chunk_size: usize) -> impl Iterator<Item = BlockRange> {
        let range: Range<BlockHeight> = self.clone().into();
        range.step_by(chunk_size).map(move |start_inclusive| {
            let chunk_size = chunk_size as BlockHeight;
            let end_exclusive = (start_inclusive + chunk_size).min(self.end_exclusive);
            (start_inclusive..end_exclusive).into()
        })
    }

    /// Returns the length of the block range.
    pub fn len(&self) -> u64 {
        self.end_exclusive - self.start_inclusive
    }

    /// Checks if a block height is within the range.
    pub fn contains(&self, block_height: BlockHeight) -> bool {
        (self.start_inclusive..self.end_exclusive).contains(&block_height)
    }
}

impl From<Range<BlockHeight>> for BlockRange {
    fn from(range: Range<BlockHeight>) -> Self {
        Self {
            start_inclusive: range.start,
            end_exclusive: range.end,
        }
    }
}

impl From<BlockRange> for Range<BlockHeight> {
    fn from(val: BlockRange) -> Self {
        val.start_inclusive..val.end_exclusive
    }
}

impl Display for BlockRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{},{})", self.start_inclusive, self.end_exclusive)
    }
}

impl IntoIterator for BlockRange {
    type Item = BlockHeight;
    type IntoIter = Range<BlockHeight>;

    fn into_iter(self) -> Self::IntoIter {
        self.into()
    }
}

/// Represents a reference to a block.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlockReference {
    pub height: BlockHeight,
    pub hash: BlockHash,
    pub timestamp: BlockTimestamp,
}

impl BlockReference {
    /// Creates a new `BlockReference`.
    pub fn new(height: BlockHeight, hash: BlockHash, timestamp: BlockTimestamp) -> Self {
        Self {
            height,
            hash,
            timestamp,
        }
    }
}

impl Display for BlockReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.height, self.hash)
    }
}

/// Trait for converting an object into a `BlockReference`.
pub trait BlockReferenceProvider {
    fn block_reference(&self) -> Result<BlockReference, Report>;
}

#[async_trait]
pub trait BlockHandle: Debug + Send + Sync + 'static {
    /// Gets the reference for the block.
    fn reference(&self) -> BlockReference;

    /// Fetches a range of blocks.
    fn fetch_range(
        &self,
        range: BlockRange,
        mode: FetchMode,
    ) -> Result<impl Stream<Item = Result<Self, IndexerError>> + Send, IndexerError>;

    /// Inserts the block into the database.
    async fn insert(&self, tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError>;

    /// Updates the block in the database.
    async fn update(&self, tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError>;
}

#[async_trait]
pub trait FetcherClient: Display + Send + Sync + Clone + 'static {
    type BlockHandle: BlockHandle;
    type Context: Display + Send + Sync + Clone + 'static;

    /// Creates a new client.
    async fn create(
        pg_pool: sqlx::PgPool,
        join_set: &mut JoinSet<Result<(), IndexerError>>,
        context: Self::Context,
    ) -> Result<Self, IndexerError>;

    /// Fetches a single block.
    async fn fetch_single(
        &self,
        selection: BlockSelection,
        mode: FetchMode,
    ) -> Result<Self::BlockHandle, IndexerError>;
}

/// Represents errors that may occur during indexing.
#[derive(Debug, thiserror::Error)]
pub enum IndexerError {
    #[error("Unexpected height: received {0}, expected {1}")]
    UnexpectedHeightSingle(BlockHeight, BlockHeight),
    #[error("Unexpected height: received {0} in range {1}, expected {2}")]
    UnexpectedHeightRange(BlockHeight, BlockRange, BlockHeight),
    #[error("Error reading block {0} in range {1}: {2}")]
    ErrorReadingBlock(BlockHeight, BlockRange, Report),
    #[error("Missing block: expected {0} in range {1}")]
    MissingBlock(BlockHeight, BlockRange),
    #[error("Too many blocks: received block {1} in range {0}")]
    TooManyBlocks(BlockRange, BlockReference),
    #[error("Provider error: {0}")]
    ProviderError(Report),
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Internal error: {0}")]
    InternalError(Report),
}

impl From<Report> for IndexerError {
    fn from(error: Report) -> Self {
        Self::InternalError(error)
    }
}
