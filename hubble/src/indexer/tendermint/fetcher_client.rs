use std::{
    error::Error,
    fmt::Display,
    num::{NonZeroU32, NonZeroU64},
};

use axum::async_trait;
use color_eyre::{eyre::Report, Result};
use cometbft_rpc::{
    rpc_types::{BlockMeta, BlockResultsResponse, Order, TxResponse},
    JsonRpcError,
};
use futures::{stream::BoxStream, FutureExt, StreamExt, TryFutureExt};
use itertools::Itertools;
use serde_json::Value;
use time::OffsetDateTime;
use tokio::task::JoinSet;
use tracing::{debug, error, info, info_span, Instrument};

use crate::{
    indexer::{
        api::{BlockHeight, BlockRange, BlockSelection, FetchMode, FetcherClient, IndexerError},
        tendermint::{
            block_handle::{BlockDetails, BlockHeader, TmBlockHandle},
            context::TmContext,
            postgres::{PgBlock, PgEvent, PgTransaction},
            provider::{Provider, RpcProviderId},
        },
    },
    postgres::{fetch_or_insert_chain_id_tx, ChainId},
};

const TX_RESULT_CODE_OK: u32 = 0;

#[derive(Clone)]
pub struct TmFetcherClient {
    pub chain_id: ChainId,
    pub provider: Provider,
    pub filter: Option<regex::Regex>,
    pub tx_search_max_page_size: u8,
}

impl Display for TmFetcherClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "chain_id: {}", self.chain_id)
    }
}

impl TmFetcherClient {
    /// Fetches a range of blocks with optional provider ID.
    pub fn fetch_range_with_provider(
        &self,
        block_range: BlockRange,
        fetch_mode: FetchMode,
        provider_id: Option<RpcProviderId>,
    ) -> Result<impl Stream<Item = Result<TmBlockHandle, IndexerError>> + '_, IndexerError> {
        debug!("Fetching block range: {}", block_range);

        let futures = async move {
            let response = self.provider.blockchain(
                block_range.start_inclusive,
                block_range.end_exclusive - 1,
                provider_id,
            )
            .await
            .map_err(|e| {
                error!(?e, "Error fetching blockchain data");
                e
            })?;

            self.process_block_metas(response.response.block_metas, fetch_mode, response.provider_id)
        };

        Ok(futures.flatten_stream())
    }

    /// Processes block metadata and constructs block handles.
    fn process_block_metas(
        &self,
        block_metas: Vec<BlockMeta>,
        fetch_mode: FetchMode,
        provider_id: RpcProviderId,
    ) -> BoxStream<Result<TmBlockHandle, IndexerError>> {
        futures::stream::iter(
            block_metas
                .into_iter()
                .sorted_by_key(|meta| meta.header.height)
                .map(move |meta| {
                    let client = self.clone();
                    async move {
                        let block_reference = meta.block_reference()?;
                        let details = match fetch_mode {
                            FetchMode::Lazy => BlockDetails::Lazy(Box::new(meta.into())),
                            FetchMode::Eager => client.fetch_block_details(&meta.into(), provider_id).await?,
                        };

                        Ok(TmBlockHandle {
                            reference: block_reference,
                            details,
                            tm_client: client,
                            provider_id,
                        })
                    }
                }),
        )
        .buffer_unordered(10) // Process up to 10 blocks concurrently
        .boxed()
    }

    /// Fetches block details including transactions and events.
    async fn fetch_block_details(
        &self,
        block_header: &BlockHeader,
        provider_id: RpcProviderId,
    ) -> Result<BlockDetails, IndexerError> {
        let block_reference = block_header.block_reference()?;
        info!("Fetching details for block: {}", block_reference);

        let (block_results, transactions) = self.provider.fetch_block_data(block_reference.height, provider_id).await?;
        self.validate_block_data(provider_id, &block_results, &transactions)?;

        let (block, transactions, events) = self.transform_to_pg(block_header, block_results, transactions)?;
        Ok(BlockDetails::Eager(block, transactions, events))
    }

    /// Validates the consistency of fetched block data.
    fn validate_block_data(
        &self,
        provider_id: RpcProviderId,
        block_results: &BlockResultsResponse,
        transactions: &[TxResponse],
    ) -> Result<(), IndexerError> {
        let tx_event_count: usize = transactions.iter().map(|tx| tx.tx_result.events.len()).sum();
        let block_event_count: usize = block_results.txs_results.iter().flat_map(|r| r.iter()).map(|r| r.events.len()).sum();

        if tx_event_count != block_event_count {
            return Err(IndexerError::ProviderError(Report::msg(format!(
                "Mismatch in event counts for provider {:?}: transactions ({}) vs block ({})",
                provider_id, tx_event_count, block_event_count
            ))));
        }

        Ok(())
    }

    /// Converts fetched data into PostgreSQL-compatible types.
    fn transform_to_pg(
        &self,
        block_header: &BlockHeader,
        block_results: BlockResultsResponse,
        transactions: Vec<TxResponse>,
    ) -> Result<(PgBlock, Vec<PgTransaction>, Vec<PgEvent>), IndexerError> {
        let block_reference = block_header.block_reference()?;

        let pg_block = PgBlock {
            chain_id: self.chain_id,
            hash: block_reference.hash.clone(),
            height: block_reference.height,
            time: block_reference.timestamp,
            data: serde_json::to_value(block_header)?.to_string(),
        };

        let pg_transactions = transactions
            .into_iter()
            .map(|tx| PgTransaction {
                chain_id: self.chain_id,
                hash: tx.hash.to_string(),
                data: serde_json::to_value(&tx)?.to_string(),
                index: tx.index.try_into().unwrap(),
                block_hash: block_reference.hash.clone(),
                block_height: block_reference.height,
                time: block_reference.timestamp,
            })
            .collect();

        let pg_events = block_results.events(self.chain_id, block_reference.hash, block_reference.timestamp);

        Ok((pg_block, pg_transactions, pg_events))
    }
}
