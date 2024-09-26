use std::convert::Into;

use aptos_rest_client::{aptos_api_types::Block, Transaction};
use axum::async_trait;
use color_eyre::eyre::Report;
use futures::{stream::FuturesOrdered, Stream};
use itertools::Itertools;
use sqlx::Postgres;
use time::OffsetDateTime;
use tracing::debug;

use super::{fetcher_client::AptosFetcherClient, provider::RpcProviderId};
use crate::indexer::{api::{
            BlockHandle, BlockRange, BlockReference, BlockReferenceProvider, BlockSelection, FetchMode, IndexerError
        }, aptos::postgres::{delete_aptos_block_transactions_events, insert_aptos_block, PgBlock, PgEvent, PgTransaction}};

impl BlockReferenceProvider for Block {
    fn block_reference(&self) -> Result<BlockReference, Report> {
        Ok(BlockReference {
            height: self.block_height.into(),
            hash: self.block_hash.to_string(),
            timestamp: OffsetDateTime::from_unix_timestamp_nanos((self.block_timestamp.0 as i128) * 1000).map_err(Report::from)?
        })
    }
}

#[derive(Clone)]
pub enum BlockDetails {
    Lazy(Block),
    Eager(Block, Vec<Transaction>),
}

#[derive(Clone)]
pub struct AptosBlockHandle {
    pub internal_chain_id: i32,
    pub reference: BlockReference,
    pub details: BlockDetails,
    pub aptos_client: AptosFetcherClient,
    pub provider_id: RpcProviderId,
}

#[async_trait]
impl BlockHandle for AptosBlockHandle {
    fn reference(&self) -> BlockReference {
        self.reference.clone()
    }

    fn fetch_range(
        &self,
        block_range: BlockRange,
        fetch_mode: FetchMode,
    ) -> Result<impl Stream<Item = Result<Self, IndexerError>> + Send, IndexerError> {
        debug!("{}: fetching", block_range);

        Ok(FuturesOrdered::from_iter(
            block_range.clone().into_iter().map(|height| async move {
                self.aptos_client
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
        debug!("{}: updating", reference);

        let (block, transactions) = match &self.details {
            BlockDetails::Lazy(block) => (block, self.aptos_client.fetch_transactions(block, self.provider_id).await?),
            BlockDetails::Eager(block, transactions) => (block, transactions.clone()),
        };

        let mut event_index = 0;

        // block
        insert_aptos_block(
            tx, 
            PgBlock { 
                internal_chain_id: self.internal_chain_id, 
                height: self.reference.height as i64, 
                block_hash: self.reference.hash.clone(), 
                timestamp: self.reference.timestamp, 
                first_version: block.first_version.0 as i64, // TODO: check if .0 is ok
                last_version: block.last_version.0 as i64, 
                transactions: transactions.into_iter().enumerate().filter_map(|(transaction_index, transaction)| match transaction {
                    Transaction::UserTransaction(transaction) => Some(
                        PgTransaction { 
                            internal_chain_id: self.internal_chain_id, 
                            height: self.reference.height as i64, 
                            version: transaction.info.version.0 as i64, 
                            transaction_hash: transaction.info.hash.to_string(), 
                            transaction_index: transaction_index as i64,
                            events: transaction.events.into_iter().enumerate().map(|(transaction_event_index, event)| { 
                                let event = PgEvent { 
                                    internal_chain_id: self.internal_chain_id, 
                                    height: self.reference.height as i64, 
                                    version: transaction.info.version.0 as i64, 
                                    index: event_index as i64,
                                    transaction_event_index: transaction_event_index as i64,
                                    sequence_number: event.sequence_number.0 as i64, 
                                    creation_number: event.guid.creation_number.0 as i64, 
                                    account_address: event.guid.account_address.to_standard_string(), 
                                    typ: event.typ.to_string(), 
                                    data: event.data 
                                };

                                // TODO: can this be less verbose?
                                event_index += 1;

                                event
                            }
                            ).collect_vec()
                        }
                    ),
                    _ => None,
                }).collect_vec()
            },
        ).await?;

        debug!("{}: done", reference);
        Ok(())
    }

    async fn update(&self, tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        let reference = self.reference();
        debug!("{}: updating", reference);

        delete_aptos_block_transactions_events(tx, self.internal_chain_id, self.reference.height).await?;
        self.insert(tx).await?;

        debug!("{}: done", reference);
        Ok(())
    }
}
