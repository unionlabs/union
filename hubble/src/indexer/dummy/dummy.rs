use std::fmt::{Display, Formatter};

use axum::async_trait;
use color_eyre::eyre::Report;
use futures::{stream::FuturesOrdered, Stream};
use sqlx::Postgres;
use time::OffsetDateTime;
use tokio::task::JoinSet;
use tracing::{debug, info};

use crate::indexer::api::{
    BlockHandle, BlockRange, BlockReference, BlockSelection, FetchMode, FetcherClient, IndexerError,
};

#[derive(Clone)]
pub struct DummyBlock {
    reference: BlockReference,
    dummy_client: DummyFetcherClient,
    content: Option<String>,
}

#[async_trait]
impl BlockHandle for DummyBlock {
    fn reference(&self) -> BlockReference {
        self.reference.clone()
    }

    fn fetch_range(
        &self,
        block_range: BlockRange,
        mode: FetchMode,
    ) -> Result<impl Stream<Item = Result<Self, IndexerError>>, IndexerError> {
        debug!("fetch_range => {}", block_range);

        Ok(FuturesOrdered::from_iter(block_range.into_iter().map(
            |height| async move {
                let message = match self
                    .dummy_client
                    .fetch_single(BlockSelection::Height(height), mode)
                    .await
                {
                    Ok(block) => {
                        info!("fetch_range - fetched: {}", height);

                        Ok(block)
                    }
                    Err(report) => {
                        info!("fetch_range - error at {}: {}", height, report);

                        Err(report)
                    }
                };
                debug!("sending message for {}", height);

                message
            },
        )))
    }

    async fn insert(&self, _tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        let content = match self.content.clone() {
            Some(content) => content,
            None => {
                self.dummy_client
                    .fetch_content(self.reference.clone())
                    .await?
            }
        };

        info!(
            "chain: {} - insert: {} - {}",
            self.dummy_client.internal_chain_id,
            self.reference(),
            content,
        );

        // sleep(Duration::from_millis(50)).await;

        Ok(())
    }

    async fn update(&self, _tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        info!(
            "chain: {} - update: {} - {}",
            self.dummy_client.internal_chain_id,
            self.reference(),
            match self.content.clone() {
                Some(content) => content,
                None =>
                    self.dummy_client
                        .fetch_content(self.reference.clone())
                        .await?,
            },
        );

        // sleep(Duration::from_millis(50)).await;

        Ok(())
    }
}

#[derive(Clone)]
pub struct DummyFetcherClient {
    internal_chain_id: u32,
}

impl Display for DummyFetcherClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "dummy: internal-chain-id: {}", self.internal_chain_id)
    }
}

impl DummyFetcherClient {
    fn new(internal_chain_id: u32) -> Self {
        DummyFetcherClient { internal_chain_id }
    }

    async fn fetch_content(&self, block_reference: BlockReference) -> Result<String, Report> {
        // sleep(Duration::from_millis(10)).await;
        Ok(format!("{}", block_reference))
    }
}

#[derive(Clone)]
pub struct DummyContext {
    pub bla: u64,
}

impl Display for DummyContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "dummy: bla: {}", self.bla)
    }
}

#[async_trait]
impl FetcherClient for DummyFetcherClient {
    type BlockHandle = DummyBlock;
    type Context = DummyContext;

    async fn create(
        _pg_pool: sqlx::PgPool,
        _join_set: &mut JoinSet<Result<(), IndexerError>>,
        _context: DummyContext,
    ) -> Result<Self, IndexerError> {
        let fetcher_client = DummyFetcherClient::new(8);

        Ok(fetcher_client)
    }

    async fn fetch_single(
        &self,
        selection: BlockSelection,
        mode: FetchMode,
    ) -> Result<Self::BlockHandle, IndexerError> {
        // sleep(Duration::from_millis(10)).await;
        let reference = BlockReference::new(
            match selection {
                BlockSelection::LastFinalized => 43000,
                BlockSelection::Height(height) => height,
            },
            match selection {
                BlockSelection::LastFinalized => "42".to_string(),
                BlockSelection::Height(height) => format!("{}", height),
                // BlockReference::Height(height) => format!("{}-{}", height, OffsetDateTime::now_utc()),
            },
            OffsetDateTime::now_utc(),
        );

        Ok(DummyBlock {
            reference: reference.clone(),
            dummy_client: self.clone(),
            content: match mode {
                FetchMode::Eager => Some(self.fetch_content(reference.clone()).await?),
                FetchMode::Lazy => None,
            },
        })
    }
}
