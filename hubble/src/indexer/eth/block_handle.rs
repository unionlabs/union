use alloy::rpc::types::Block;
use axum::async_trait;
use color_eyre::eyre::Report;
use futures::{stream::FuturesOrdered, Stream};
use serde::{Deserialize, Serialize};
use sqlx::Postgres;
use time::OffsetDateTime;
use tracing::debug;

use crate::{
    indexer::{
        api::{BlockHandle, BlockRange, BlockReference, BlockSelection, FetchMode, IndexerError},
        eth::{
            fetcher_client::EthFetcherClient,
            postgres::{delete_eth_log, insert_batch_logs},
            provider::RpcProviderId,
        },
    },
    postgres::{ChainId, InsertMode},
};

#[derive(Clone)]
pub enum BlockDetails {
    Lazy(Block),
    Eager(Option<BlockInsert>),
}

#[must_use]
#[derive(Debug, Clone)]
pub struct BlockInsert {
    pub chain_id: ChainId,
    pub hash: String,
    pub header: Block,
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
    async fn get_block_insert(&self) -> Result<Option<BlockInsert>, Report> {
        Ok(match self.details.clone() {
            BlockDetails::Eager(block_insert) => block_insert,
            BlockDetails::Lazy(block) => {
                self.eth_client
                    .fetch_details(&block, self.provider_id)
                    .await?
            }
        })
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
        debug!("{}: fetching", block_range);

        Ok(FuturesOrdered::from_iter(
            block_range.clone().into_iter().map(|height| async move {
                self.eth_client
                    .fetch_single_with_provider(
                        BlockSelection::Height(height),
                        fetch_mode,
                        Some(self.provider_id),
                    )
                    .await
            }),
        ))
    }

    async fn insert(&self, tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        let reference = self.reference();
        debug!("{}: inserting", reference);

        let block_to_insert = self.get_block_insert().await?;

        match block_to_insert {
            Some(block_to_insert) => {
                debug!(
                    "{}: block with transactions ({}) => insert",
                    reference,
                    block_to_insert.transactions.len()
                );

                insert_batch_logs(tx, vec![block_to_insert.into()], InsertMode::Insert).await?;
            }
            None => {
                debug!("{}: block without transactions => ignore", reference);
            }
        }

        debug!("{}: done", reference);

        Ok(())
    }

    async fn update(&self, tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        let reference = self.reference();
        debug!("{}: updating", reference);

        let block_to_insert = self.get_block_insert().await?;

        if let Some(block_to_insert) = block_to_insert {
            debug!(
                "{}: block with transactions ({}) => upsert",
                reference,
                block_to_insert.transactions.len()
            );
            insert_batch_logs(tx, vec![block_to_insert.into()], InsertMode::Upsert).await?;
        } else {
            debug!("{}: block without transactions => delete", reference);
            delete_eth_log(tx, self.eth_client.chain_id.db, reference.height).await?;
        }

        debug!("{}: done", reference);
        Ok(())
    }
}
