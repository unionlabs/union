use std::{fmt::Display, num::ParseIntError, ops::Range};

use async_nats::jetstream::{
    consumer::{
        pull::{BatchErrorKind, MessagesErrorKind},
        StreamErrorKind,
    },
    context::PublishErrorKind,
    stream::ConsumerErrorKind,
};
use axum::async_trait;
use color_eyre::eyre::Report;
use futures::Stream;
use sqlx::Postgres;
use time::OffsetDateTime;
use tokio::task::JoinSet;
use tracing::error;

use crate::indexer::event::BlockEvents;

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
    #[error("nats publish error: {0}")]
    NatsPublishError(#[from] async_nats::error::Error<PublishErrorKind>),
    #[error("nats consumer error: {0}")]
    NatsConsumerError(#[from] async_nats::error::Error<ConsumerErrorKind>),
    #[error("nats fetch error: {0}")]
    NatsFetchError(#[from] async_nats::error::Error<BatchErrorKind>),
    #[error("nats messages error: {0}")]
    NatsMessagesError(#[from] async_nats::error::Error<StreamErrorKind>),
    #[error("nats pull error: {0}")]
    NatsPullError(#[from] async_nats::error::Error<MessagesErrorKind>),
    #[error("nats next error: {0}")]
    NatsNextError(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("nats ack error: {0}")]
    NatsAckError(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("nats nack error: {0}")]
    NatsNackError(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("nats meta error: {0}")]
    NatsMetaError(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("formatting json error: {0}")]
    FormattingJsonError(#[from] serde_json::Error),
    #[error("error decoding data: {0}")]
    NatsDecodeError(#[from] lz4_flex::block::DecompressError),
    #[error("unsupported encoding: {0}")]
    NatsUnsupportedEncoding(String),
    #[error("missing message sequence in stream sequence: {0}, consumer_sequence sequence: {1}")]
    NatsMissingMessageHeaders(u64, u64),
    #[error("missing headers: in stream sequence: {0}, consumer_sequence sequence: {1}")]
    NatsMissingMessageSequence(u64, u64),
    #[error("unsupported message sequence:{0} in stream sequence: {1}, consumer_sequence sequence: {2} ({3})")]
    NatsUnparseableMessageSequence(String, u64, u64, ParseIntError),
    #[error("missing message hash: in stream sequence: {0}, consumer_sequence sequence: {1}")]
    NatsMissingMessageHash(u64, u64),
    #[error("unsupported message hash:{0} in stream sequence: {1}, consumer_sequence sequence: {2} ({3})")]
    NatsUnparseableMessageHash(String, u64, u64, hex::FromHexError),
    #[error(
        "missing universal chain id: in stream sequence: {0}, consumer_sequence sequence: {1}"
    )]
    NatsMissingUniversalChainId(u64, u64),
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
pub type UniversalChainId = String;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct BlockRange {
    pub start_inclusive: BlockHeight,
    pub end_exclusive: BlockHeight,
}

impl BlockRange {
    pub fn range_chunks(self, chunk_size: usize) -> impl Iterator<Item = BlockRange> {
        let range: Range<BlockHeight> = self.clone().into();

        range.step_by(chunk_size).map(move |start_inclusive| {
            let chunk_size: u64 = chunk_size.try_into().unwrap();
            let end_exclusive = (start_inclusive + chunk_size).min(self.end_exclusive);
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
        write!(f, "{} ({})", self.height, self.hash)
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
    async fn insert(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
    ) -> Result<Option<BlockEvents>, IndexerError>;
    async fn update(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
    ) -> Result<Option<BlockEvents>, IndexerError>;
}
