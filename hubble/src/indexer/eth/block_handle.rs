use alloy::rpc::types::Block;
use axum::async_trait;
use color_eyre::eyre::Report;
use const_hex::ToHexExt;
use futures::{stream::FuturesOrdered, Stream};
use sqlx::Postgres;
use tracing::debug;

use super::fetcher_client::EthFetcherClient;
use crate::{
    eth::BlockInsert,
    indexer::{
        api::{BlockHandle, BlockRange, BlockReference, BlockSelection, FetchMode, IndexerError},
        eth::postgres::delete_eth_log,
    },
    postgres::{insert_batch_logs, update_contracts_indexed_heights, InsertMode},
};

#[derive(Clone)]
pub enum BlockDetails {
    Lazy(Block),
    Eager(Option<BlockInsert>),
}

#[derive(Clone)]
pub struct EthBlockHandle {
    pub reference: BlockReference,
    pub details: BlockDetails,
    pub eth_client: EthFetcherClient,
    pub provider_index: usize,
}

impl EthBlockHandle {
    async fn get_block_insert(&self) -> Result<Option<BlockInsert>, Report> {
        Ok(match self.details.clone() {
            BlockDetails::Eager(block_insert) => block_insert,
            BlockDetails::Lazy(block) => {
                self.eth_client
                    .fetch_details(&block, self.provider_index)
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
                        Some(self.provider_index),
                    )
                    .await
            }),
        ))
    }

    async fn insert(&self, tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        let reference = self.reference();
        debug!("{}", reference);

        let block_to_insert = self.get_block_insert().await?;

        match block_to_insert {
            Some(block_to_insert) => {
                debug!(
                    "{}: block with transactions ({}) => insert",
                    reference,
                    block_to_insert.transactions.len()
                );

                // TODO: remove to this module once legacy eth is removed
                insert_batch_logs(tx, vec![block_to_insert.into()], InsertMode::Insert).await?;
            }
            None => {
                debug!("{}: block without transactions => ignore", reference);
            }
        }

        // TODO: remove once all data based on new hubble tables
        debug!("{}: updating contract heights", reference);
        update_contracts_indexed_heights(
            tx,
            self.eth_client
                .contracts
                .iter()
                .map(|addr| format!("0x{}", addr.encode_hex()))
                .collect(),
            reference.height as i64,
            reference.timestamp,
            self.eth_client.chain_id,
        )
        .await?;

        debug!("{}: done", reference);

        Ok(())
    }

    async fn update(&self, tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        let reference = self.reference();
        debug!("{}", reference);

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
