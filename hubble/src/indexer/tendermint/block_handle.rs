use axum::async_trait;
use color_eyre::eyre::{eyre, Report};
use cometbft_rpc::{
    rpc_types::{BlockMeta, BlockResponse, CommitResponse},
    types::types::{block_id::BlockId, header::Header},
};
use futures::Stream;
use sqlx::Postgres;
use time::OffsetDateTime;
use tracing::{debug, error};

use crate::indexer::{
    api::{
        BlockHandle, BlockRange, BlockReference, BlockReferenceProvider, FetchMode, IndexerError,
    },
    tendermint::{
        fetcher_client::TmFetcherClient,
        postgres::{
            delete_tm_block_transactions_events, insert_batch_blocks, insert_batch_events,
            insert_batch_transactions, PgBlock, PgEvent, PgTransaction,
        },
        provider::RpcProviderId,
    },
};

#[derive(Clone)]
pub struct BlockHeader {
    pub block_id: BlockId,
    pub header: Header,
}

impl From<BlockResponse> for BlockHeader {
    fn from(block_response: BlockResponse) -> Self {
        BlockHeader {
            block_id: block_response.block_id,
            header: block_response.block.header,
        }
    }
}

impl From<BlockMeta> for BlockHeader {
    fn from(block_meta: BlockMeta) -> Self {
        BlockHeader {
            block_id: block_meta.block_id,
            header: block_meta.header,
        }
    }
}

impl From<CommitResponse> for BlockHeader {
    fn from(commit_response: CommitResponse) -> Self {
        BlockHeader {
            block_id: commit_response.signed_header.commit.block_id,
            header: commit_response.signed_header.header,
        }
    }
}

/// Generic implementation for block reference creation.
fn block_reference_from_parts(
    height: i64,
    hash: Option<String>,
    timestamp_nanos: u64,
) -> Result<BlockReference, Report> {
    Ok(BlockReference {
        height: height.try_into()?,
        hash: hash.ok_or_else(|| IndexerError::ProviderError(eyre!("Expected hash")))?,
        timestamp: OffsetDateTime::from_unix_timestamp_nanos(timestamp_nanos.into())
            .map_err(|err| IndexerError::ProviderError(err.into()))?,
    })
}

impl BlockReferenceProvider for BlockHeader {
    fn block_reference(&self) -> Result<BlockReference, Report> {
        block_reference_from_parts(
            self.header.height.inner(),
            self.block_id.hash.map(|hash| hash.to_string()),
            self.header.time.as_unix_nanos(),
        )
    }
}

impl BlockReferenceProvider for BlockMeta {
    fn block_reference(&self) -> Result<BlockReference, Report> {
        block_reference_from_parts(
            self.header.height.inner(),
            self.block_id.hash.map(|hash| hash.to_string()),
            self.header.time.as_unix_nanos(),
        )
    }
}

#[derive(Clone)]
pub enum BlockDetails {
    Lazy(Box<BlockHeader>),
    Eager(PgBlock, Vec<PgTransaction>, Vec<PgEvent>),
}

#[derive(Clone)]
pub struct TmBlockHandle {
    pub reference: BlockReference,
    pub details: BlockDetails,
    pub tm_client: TmFetcherClient,
    pub provider_id: RpcProviderId,
}

impl TmBlockHandle {
    /// Fetches block insert details.
    async fn get_block_insert(
        &self,
    ) -> Result<(PgBlock, Vec<PgTransaction>, Vec<PgEvent>), Report> {
        match self.details.clone() {
            BlockDetails::Eager(block, transactions, events) => Ok((block, transactions, events)),
            BlockDetails::Lazy(block_header) => {
                self.tm_client
                    .fetch_details(&block_header, self.provider_id)
                    .await
            }
        }
    }

    /// Inserts block, transactions, and events into the database.
    async fn insert_block_data(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        block: PgBlock,
        transactions: Vec<PgTransaction>,
        events: Vec<PgEvent>,
    ) -> Result<(), IndexerError> {
        insert_batch_blocks(tx, vec![block]).await?;
        insert_batch_transactions(tx, transactions).await?;
        insert_batch_events(tx, events).await?;
        Ok(())
    }
}

#[async_trait]
impl BlockHandle for TmBlockHandle {
    fn reference(&self) -> BlockReference {
        self.reference.clone()
    }

    fn fetch_range(
        &self,
        block_range: BlockRange,
        fetch_mode: FetchMode,
    ) -> Result<impl Stream<Item = Result<Self, IndexerError>> + Send, IndexerError> {
        debug!(block_range = ?block_range, "Fetching block range");

        self.tm_client
            .fetch_range_with_provider(block_range, fetch_mode, Some(self.provider_id))
    }

    async fn insert(&self, tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        debug!(block_reference = ?self.reference, "Inserting block");

        let (block, transactions, events) = self
            .get_block_insert()
            .await
            .map_err(|err| {
                error!(error = ?err, block_reference = ?self.reference, "Failed to fetch block details");
                IndexerError::ProviderError(err)
            })?;

        self.insert_block_data(tx, block, transactions, events).await?;

        debug!(block_reference = ?self.reference, "Insertion complete");
        Ok(())
    }

    async fn update(&self, tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        debug!(block_reference = ?self.reference, "Updating block");

        delete_tm_block_transactions_events(tx, self.tm_client.chain_id.db, self.reference.height)
            .await?;
        self.insert(tx).await?;

        debug!(block_reference = ?self.reference, "Update complete");
        Ok(())
    }
}
