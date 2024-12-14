use std::convert::Into;

use aptos_rest_client::{
    aptos_api_types::{Block, TransactionPayload},
    Transaction,
};
use axum::async_trait;
use color_eyre::eyre::Report;
use futures::{stream::FuturesOrdered, Stream};
use itertools::Itertools;
use sqlx::Postgres;
use time::OffsetDateTime;
use tracing::{debug, trace, warn};

use crate::indexer::{
    api::{
        BlockHandle, BlockRange, BlockReference, BlockReferenceProvider, BlockSelection, FetchMode,
        IndexerError,
    },
    aptos::{
        fetcher_client::AptosFetcherClient,
        postgres::{
            active_contracts, delete_aptos_block_transactions_events, insert_aptos_block, PgBlock,
            PgEvent, PgTransaction,
        },
        provider::RpcProviderId,
    },
};

impl BlockReferenceProvider for Block {
    fn block_reference(&self) -> Result<BlockReference, Report> {
        let block_timestamp: i128 = self.block_timestamp.0.into();
        let timestamp = OffsetDateTime::from_unix_timestamp_nanos(block_timestamp * 1_000)
            .map_err(Report::from)?;
        Ok(BlockReference {
            height: self.block_height.into(),
            hash: self.block_hash.to_string(),
            timestamp,
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
        debug!(block_range = ?block_range, "Fetching block range");

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
        debug!(block = ?reference, "Inserting block");

        let (block, transactions) = match &self.details {
            BlockDetails::Lazy(block) => {
                let transactions = self
                    .aptos_client
                    .fetch_transactions(block, self.provider_id)
                    .await?;
                (block, transactions)
            }
            BlockDetails::Eager(block, transactions) => (block, transactions.clone()),
        };

        let active_contracts = fetch_active_contracts(tx, self.internal_chain_id, block.block_height)
            .await?;

        let filtered_transactions = filter_transactions(
            &transactions,
            &active_contracts,
            self.internal_chain_id,
            &self.reference,
        );

        if filtered_transactions.is_empty() {
            warn!(block = ?reference, "No matching transactions found");
            return Ok(());
        }

        debug!(
            block = ?reference,
            transactions_count = filtered_transactions.len(),
            "Inserting transactions"
        );

        insert_aptos_block(
            tx,
            PgBlock {
                internal_chain_id: self.internal_chain_id,
                height: self.reference.height.try_into()?,
                block_hash: self.reference.hash.clone(),
                timestamp: self.reference.timestamp,
                first_version: block.first_version.0.try_into()?,
                last_version: block.last_version.0.try_into()?,
                transactions: filtered_transactions,
            },
        )
        .await?;

        Ok(())
    }

    async fn update(&self, tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        debug!(block = ?self.reference, "Updating block");
        delete_aptos_block_transactions_events(tx, self.internal_chain_id, self.reference.height)
            .await?;
        self.insert(tx).await
    }
}

async fn fetch_active_contracts(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    internal_chain_id: i32,
    block_height: i32,
) -> Result<Vec<String>, IndexerError> {
    active_contracts(tx, internal_chain_id, block_height.into())
        .await
        .map_err(|e| {
            warn!(error = ?e, "Failed to fetch active contracts");
            e.into()
        })
}

fn filter_transactions(
    transactions: &[Transaction],
    active_contracts: &[String],
    internal_chain_id: i32,
    reference: &BlockReference,
) -> Vec<PgTransaction> {
    let mut event_index_iter = 0..;

    transactions
        .iter()
        .enumerate()
        .filter_map(|(transaction_index, transaction)| {
            if let Transaction::UserTransaction(tx) = transaction {
                if let TransactionPayload::EntryFunctionPayload(payload) = &tx.request.payload {
                    let account_address = payload.function.module.address;

                    if active_contracts.contains(&account_address.to_standard_string()) {
                        return Some(PgTransaction {
                            internal_chain_id,
                            height: reference.height.try_into().unwrap(),
                            version: tx.info.version.0.try_into().unwrap(),
                            transaction_hash: tx.info.hash.to_string(),
                            transaction_index: transaction_index.try_into().unwrap(),
                            events: tx
                                .events
                                .iter()
                                .enumerate()
                                .map(|(event_index, event)| PgEvent {
                                    internal_chain_id,
                                    height: reference.height.try_into().unwrap(),
                                    version: tx.info.version.0.try_into().unwrap(),
                                    index: event_index_iter.next().unwrap(),
                                    transaction_event_index: event_index.try_into().unwrap(),
                                    sequence_number: event.sequence_number.0.try_into().unwrap(),
                                    creation_number: event.guid.creation_number.0.try_into().unwrap(),
                                    account_address: event.guid.account_address.to_standard_string(),
                                    typ: event.typ.to_string(),
                                    data: event.data.clone(),
                                })
                                .collect_vec(),
                        });
                    } else {
                        trace!(contract = ?account_address, "Contract not configured");
                    }
                } else {
                    trace!("Payload is not a function");
                }
            } else {
                trace!("Not a user transaction");
            }
            None
        })
        .collect_vec()
}
