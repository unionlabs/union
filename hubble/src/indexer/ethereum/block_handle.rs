use alloy::network::AnyRpcBlock;
use axum::async_trait;
use color_eyre::eyre::Report;
use futures::{stream::FuturesOrdered, Stream};
use serde::{Deserialize, Serialize};
use sqlx::Postgres;
use time::OffsetDateTime;
use tracing::{debug, error, warn};

use crate::{
    indexer::{
        api::{BlockHandle, BlockRange, BlockReference, BlockSelection, FetchMode, IndexerError},
        ethereum::{
            fetcher_client::EthFetcherClient,
            postgres::{delete_eth_log, insert_batch_logs},
            provider::RpcProviderId,
        },
    },
    postgres::{ChainId, InsertMode},
};

#[derive(Clone)]
pub enum BlockDetails {
    Lazy(AnyRpcBlock),
    Eager(Option<BlockInsert>),
}

#[must_use]
#[derive(Debug, Clone)]
pub struct BlockInsert {
    pub chain_id: ChainId,
    pub hash: String,
    pub header: AnyRpcBlock,
    pub height: i32,
    pub time: OffsetDateTime,
    pub transactions: Vec<TransactionInsert>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionInsert {
    pub hash: String,
    pub index: i32,
    pub events: Vec<EventInsert>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventInsert {
    pub data: serde_json::Value,
    pub log_index: usize,
    pub transaction_log_index: i32,
}

#[derive(Clone)]
pub struct EthBlockHandle {
    pub reference: BlockReference,
    pub details: BlockDetails,
    pub eth_client: EthFetcherClient,
    pub provider_id: RpcProviderId,
}

impl EthBlockHandle {
    async fn fetch_block_insert(&self) -> Result<Option<BlockInsert>, IndexerError> {
        match &self.details {
            BlockDetails::Eager(block_insert) => Ok(block_insert.clone()),
            BlockDetails::Lazy(block) => {
                self.eth_client
                    .fetch_details(block, self.provider_id)
                    .await
                    .map_err(|e| {
                        error!(error = ?e, "Failed to fetch block details");
                        IndexerError::FetchError
                    })
            }
        }
    }

    async fn process_block(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        insert_mode: InsertMode,
    ) -> Result<(), IndexerError> {
        let reference = self.reference();
        let block_to_insert = self.fetch_block_insert().await?;

        match block_to_insert {
            Some(block) => {
                debug!(
                    block_height = reference.height,
                    transaction_count = block.transactions.len(),
                    "Processing block with transactions"
                );

                insert_batch_logs(tx, vec![block.into()], insert_mode)
                    .await
                    .map_err(|e| {
                        error!(error = ?e, "Failed to insert batch logs");
                        IndexerError::InsertError
                    })?;
            }
            None => {
                debug!(
                    block_height = reference.height,
                    "Block has no transactions, skipping"
                );
            }
        }

        Ok(())
    }
}

#[async_trait]
impl BlockHandle for EthBlockHandle {
    fn reference(&self) -> BlockReference {
        self.reference.clone()
    }

    fn fetch_range(
        &self,
        block_range: BlockRange,
        fetch_mode: FetchMode,
    ) -> Result<impl Stream<Item = Result<Self, IndexerError>>, IndexerError> {
        debug!(block_range = ?block_range, "Fetching block range");

        let stream = block_range.clone().into_iter().map(|height| async move {
            self.eth_client
                .fetch_single_with_provider(
                    BlockSelection::Height(height),
                    fetch_mode,
                    Some(self.provider_id),
                )
                .await
        });

        Ok(FuturesOrdered::from_iter(stream))
    }

    async fn insert(&self, tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        debug!(block_reference = ?self.reference(), "Inserting block");
        self.process_block(tx, InsertMode::Insert).await
    }

    async fn update(&self, tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        debug!(block_reference = ?self.reference(), "Updating block");

        let block_to_insert = self.fetch_block_insert().await?;

        if let Some(block) = block_to_insert {
            debug!(
                block_height = self.reference().height,
                transaction_count = block.transactions.len(),
                "Upserting block with transactions"
            );
            insert_batch_logs(tx, vec![block.into()], InsertMode::Upsert).await?;
        } else {
            debug!(
                block_height = self.reference().height,
                "Block has no transactions, deleting logs"
            );
            delete_eth_log(tx, self.eth_client.chain_id.db, self.reference().height).await?;
        }

        Ok(())
    }
}
