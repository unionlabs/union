use alloy::network::AnyRpcBlock;
use axum::async_trait;
use color_eyre::eyre::Report;
use futures::{stream::FuturesOrdered, Stream};
use serde::{Deserialize, Serialize};
use sqlx::Postgres;
use time::OffsetDateTime;
use tracing::{debug, trace};

use crate::{
    indexer::{
        api::{BlockHandle, BlockRange, BlockReference, BlockSelection, FetchMode, IndexerError},
        ethereum::{
            fetcher_client::EthFetcherClient, postgres::insert_batch_logs, provider::RpcProviderId,
        },
        event::{BlockEvents, SupportedBlockEvent},
    },
    postgres::ChainId,
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
    // passing the ucs events to keep the existing flow
    // BlockInsert can be removed once legacy events are deprecated
    pub ucs_events: Vec<SupportedBlockEvent>,
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

    async fn insert(
        &self,
        _tx: &mut sqlx::Transaction<'_, Postgres>,
    ) -> Result<Option<BlockEvents>, IndexerError> {
        let reference = self.reference();
        debug!("{}: inserting", reference);

        let block_to_insert = self.get_block_insert().await?;

        let events = match block_to_insert {
            Some(block_to_insert) => {
                debug!(
                    "{}: block with transactions ({}) => insert",
                    reference,
                    block_to_insert.transactions.len()
                );

                let ucs_events = block_to_insert.ucs_events.clone();
                // legacy: convert to SupportedBlockEvent::EthereumLog
                let legacy_events = insert_batch_logs(vec![block_to_insert.into()]).await?;

                legacy_events.into_iter().chain(ucs_events).collect()
            }
            None => {
                debug!("{}: block without transactions => ignore", reference);

                vec![]
            }
        };

        trace!("{}: insert => events: {:?}", reference, events);
        debug!("{}: insert => done", reference);

        Ok((!events.is_empty()).then_some(events.into()))
    }

    async fn update(
        &self,
        _tx: &mut sqlx::Transaction<'_, Postgres>,
    ) -> Result<Option<BlockEvents>, IndexerError> {
        let reference = self.reference();
        debug!("{}: updating", reference);

        let block_to_insert = self.get_block_insert().await?;

        let events = if let Some(block_to_insert) = block_to_insert {
            debug!(
                "{}: block with transactions ({}) => upsert",
                reference,
                block_to_insert.transactions.len()
            );
            let ucs_events = block_to_insert.ucs_events.clone();
            // legacy: convert to SupportedBlockEvent::EthereumLog
            let legacy_events = insert_batch_logs(vec![block_to_insert.into()]).await?;

            legacy_events.into_iter().chain(ucs_events).collect()
        } else {
            debug!("{}: block without transactions => delete", reference);
            vec![]
        };

        trace!("{}: update => events: {:?}", reference, events);
        debug!("{}: update => done", reference);

        Ok((!events.is_empty()).then_some(events.into()))
    }
}
