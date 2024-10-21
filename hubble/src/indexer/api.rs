use std::{fmt::Display, ops::Range};

use axum::async_trait;
use color_eyre::eyre::Report;
use futures::Stream;
use sqlx::Postgres;
use time::OffsetDateTime;
use tokio::task::JoinSet;
use tracing::error;

#[derive(Debug, thiserror::Error)]
pub enum IndexerError {
    #[error("received unexpected height {0}: expecting {1}")]
    UnexpectedHeightSingle(BlockHeight, BlockHeight),
    #[error("received unexpected height {0} (range {1}): expecting {2}")]
    UnexpectedHeightRange(BlockHeight, BlockRange, BlockHeight),
    #[error("error reading block {0} (range {1}): {2}")]
    ErrorReadingBlock(BlockHeight, BlockRange, Report),
    #[error("expected to receive block {0} (range {1})")]
    MissingBlock(BlockHeight, BlockRange),
    #[error("received block while not expecting more (range {0}): {1}")]
    TooManyBlocks(BlockRange, BlockReference),
    #[error("received error while not expecting more (range {0}): {1}")]
    TooManyBlocksError(BlockRange, Report),
    #[error("no block at: {0}")]
    NoBlock(BlockSelection),
    #[error("database error: {0}")]
    DatabaseError(sqlx::Error),
    #[error("provider error: {0}")]
    ProviderError(Report),
    #[error("internal error: {0}")]
    InternalError(Report),
}

impl From<Report> for IndexerError {
    fn from(error: Report) -> Self {
        Self::InternalError(error)
    }
}

impl From<sqlx::Error> for IndexerError {
    fn from(error: sqlx::Error) -> Self {
        Self::DatabaseError(error)
    }
}

pub type IndexerId = String;
pub type BlockHeight = u64;
pub type BlockHash = String;
pub type BlockTimestamp = OffsetDateTime;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct BlockRange {
    pub start_inclusive: BlockHeight,
    pub end_exclusive: BlockHeight,
}

impl BlockRange {
    pub fn range_chunks(self, chunk_size: usize) -> impl Iterator<Item = BlockRange> {
        let range: Range<BlockHeight> = self.clone().into();

        range.step_by(chunk_size).map(move |start_inclusive| {
            let end_exclusive = (start_inclusive + chunk_size as u64).min(self.end_exclusive);
            (start_inclusive..end_exclusive).into()
        })
    }

    pub fn len(&self) -> u64 {
        self.end_exclusive - self.start_inclusive
    }

    pub fn contains(&self, block_height: BlockHeight) -> bool {
        block_height >= self.start_inclusive && block_height < self.end_exclusive
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

#[derive(Clone, Copy)]
pub enum FetchMode {
    Eager,
    Lazy,
}

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

#[async_trait]
pub trait FetcherClient: Display + Send + Sync + Clone + Sized + 'static {
    type BlockHandle: BlockHandle;
    type Context: Display + Send + Sync + Clone + 'static;

    async fn create(
        pg_pool: sqlx::PgPool,
        join_set: &mut JoinSet<Result<(), IndexerError>>,
        context: Self::Context,
    ) -> Result<Self, IndexerError>;

    async fn fetch_single(
        &self,
        selection: BlockSelection,
        mode: FetchMode,
    ) -> Result<Self::BlockHandle, IndexerError>;
}

#[derive(Clone, Debug)]
pub struct BlockReference {
    pub height: BlockHeight,
    pub hash: BlockHash,
    pub timestamp: BlockTimestamp,
}

impl BlockReference {
    pub fn new(height: BlockHeight, hash: BlockHash, timestamp: BlockTimestamp) -> BlockReference {
        BlockReference {
            height,
            hash,
            timestamp,
        }
    }
}

pub trait BlockReferenceProvider {
    fn block_reference(&self) -> Result<BlockReference, Report>;
}

impl Display for BlockReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.height)
    }
}

#[async_trait]
pub trait BlockHandle: Send + Sync + Sized {
    fn reference(&self) -> BlockReference;
    fn fetch_range(
        &self,
        range: BlockRange,
        mode: FetchMode,
    ) -> Result<impl Stream<Item = Result<Self, IndexerError>> + Send, IndexerError>;
    async fn insert(&self, tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError>;
    async fn update(&self, tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError>;
}
