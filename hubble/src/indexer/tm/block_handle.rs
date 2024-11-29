use axum::async_trait;
use color_eyre::eyre::{eyre, Report};
use cometbft_rpc::{
    rpc_types::{BlockMeta, BlockResponse, CommitResponse},
    types::types::{block_id::BlockId, header::Header},
};
use futures::Stream;
use sqlx::Postgres;
use time::OffsetDateTime;
use tracing::debug;

use crate::indexer::{
    api::{
        BlockHandle, BlockRange, BlockReference, BlockReferenceProvider, FetchMode, IndexerError,
    },
    tm::{
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

impl BlockReferenceProvider for BlockHeader {
    fn block_reference(&self) -> Result<BlockReference, Report> {
        Ok(BlockReference {
            height: self.header.height.inner().try_into().unwrap(),
            hash: self
                .block_id
                .hash
                .ok_or(IndexerError::ProviderError(eyre!("expected hash")))?
                .to_string(),
            timestamp: OffsetDateTime::from_unix_timestamp_nanos(
                self.header.time.as_unix_nanos().into(),
            )
            .map_err(|err| IndexerError::ProviderError(err.into()))?,
        })
    }
}

impl BlockReferenceProvider for BlockMeta {
    fn block_reference(&self) -> Result<BlockReference, Report> {
        Ok(BlockReference {
            height: self.header.height.inner().try_into().unwrap(),
            hash: self
                .block_id
                .hash
                .ok_or(IndexerError::ProviderError(eyre!("expected hash")))?
                .to_string(),
            timestamp: OffsetDateTime::from_unix_timestamp_nanos(
                self.header.time.as_unix_nanos().into(),
            )
            .map_err(|err| IndexerError::ProviderError(err.into()))?,
        })
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
    async fn get_block_insert(
        &self,
    ) -> Result<(PgBlock, Vec<PgTransaction>, Vec<PgEvent>), Report> {
        Ok(match self.details.clone() {
            BlockDetails::Eager(block, transactions, events) => (block, transactions, events),
            BlockDetails::Lazy(block_header) => {
                self.tm_client
                    .fetch_details(&block_header, self.provider_id)
                    .await?
            }
        })
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
        debug!("{}: fetching", block_range);

        self.tm_client
            .fetch_range_with_provider(block_range, fetch_mode, Some(self.provider_id))
    }

    async fn insert(&self, tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        let reference = self.reference();
        debug!("{}: inserting", reference);

        let (block, transactions, events) = self.get_block_insert().await?;

        insert_batch_blocks(tx, vec![block]).await?;
        insert_batch_transactions(tx, transactions).await?;
        insert_batch_events(tx, events).await?;

        debug!("{}: done", reference);
        Ok(())
    }

    async fn update(&self, tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        let reference = self.reference();
        debug!("{}: updating", reference);

        delete_tm_block_transactions_events(tx, self.tm_client.chain_id.db, self.reference.height)
            .await?;
        self.insert(tx).await?;

        debug!("{}: done", reference);
        Ok(())
    }
}
