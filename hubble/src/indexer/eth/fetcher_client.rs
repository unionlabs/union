use std::{collections::HashMap, fmt::Display, time::Duration};

use alloy::{
    eips::BlockId,
    primitives::{Address, BloomInput},
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::types::{Block, BlockTransactionsKind, Filter, Log},
    transports::http::{Client, Http},
};
use axum::async_trait;
use color_eyre::eyre::Report;
use time::OffsetDateTime;
use tokio::task::JoinSet;
use tracing::{debug, info, info_span, trace, warn, Instrument};

use super::{block_handle::EthBlockHandle, context::EthContext};
use crate::{
    eth::{BlockInsert, EventInsert, FromProviderError, ToLowerHex, TransactionInsert},
    indexer::{
        api::{BlockReference, BlockSelection, FetchMode, FetcherClient, IndexerError},
        eth::{block_handle::BlockDetails, create_client_tracker::schedule_create_client_checker},
    },
    postgres::{fetch_or_insert_chain_id_tx, ChainId},
    race_client::RaceClient,
};

trait BlockReferenceProvider {
    fn block_reference(&self) -> Result<BlockReference, Report>;
}

impl BlockReferenceProvider for Block {
    fn block_reference(&self) -> Result<BlockReference, Report> {
        Ok(BlockReference {
            height: self
                .header
                .number
                .ok_or(Report::msg("block without a number"))?,
            hash: self
                .header
                .hash
                .map(|h| h.to_lower_hex())
                .ok_or(Report::msg("block without a hash"))?,
            timestamp: OffsetDateTime::from_unix_timestamp(self.header.timestamp as i64)
                .map_err(FromProviderError::from)?,
        })
    }
}

#[derive(Clone)]
pub struct EthFetcherClient {
    pub chain_id: ChainId,
    pub provider: RaceClient<RootProvider<Http<Client>>>,
    pub contracts: Vec<Address>,
}

impl Display for EthFetcherClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "chain_id: {}", self.chain_id)
    }
}

impl EthFetcherClient {
    pub async fn fetch_single_with_provider(
        &self,
        selection: BlockSelection,
        mode: FetchMode,
        provider_index: Option<usize>,
    ) -> Result<EthBlockHandle, IndexerError> {
        let provider = match provider_index {
            Some(provider_index) => {
                RaceClient::new(vec![self.provider.clients[provider_index].clone()])
            }
            None => self.provider.clone(),
        };

        match provider_index {
            None => debug!("{}: fetching (race)", selection),
            Some(provider_index) => debug!(
                "{}: fetching (provider index: {})",
                selection, provider_index
            ),
        }

        let block = provider
            .get_block(
                match selection {
                    BlockSelection::LastFinalized => BlockId::finalized(),
                    BlockSelection::Height(height) => BlockId::number(height),
                },
                BlockTransactionsKind::Full,
            )
            .await;

        match block {
            Ok(Some(block)) => {
                let fastest_index = self.provider.fastest_index();

                debug!("{}: fetched (provider index: {})", selection, fastest_index);

                Ok(EthBlockHandle {
                    reference: block.block_reference()?,
                    details: match mode {
                        FetchMode::Lazy => BlockDetails::Lazy(block),
                        FetchMode::Eager => {
                            BlockDetails::Eager(self.fetch_details(&block, fastest_index).await?)
                        }
                    },
                    eth_client: self.clone(),
                    provider_index: fastest_index,
                })
            }
            Ok(None) => {
                info!("{}: does not exist", selection);

                Err(IndexerError::NoBlock(selection))
            }
            Err(report) => {
                info!("{}: error: {}", selection, report);

                Err(report.into())
            }
        }
    }

    pub async fn fetch_details(
        &self,
        block: &Block,
        provider_index: usize,
    ) -> Result<Option<BlockInsert>, Report> {
        let block_reference = block.block_reference()?;

        info!("{}: fetch", block_reference);
        // We check for a potential log match, which potentially avoids querying
        // eth_getLogs.
        let bloom = block.header.logs_bloom;
        if self
            .contracts
            .iter()
            .all(|contract| !bloom.contains_input(BloomInput::Raw(&contract.into_array())))
        {
            info!("{}: ignored (bloom)", block_reference);
            return Ok(None);
        }

        // We know now there is a potential match, we still apply a Filter to only
        // get the logs we want.
        let log_filter = Filter::new().select(block.header.hash.unwrap());
        let log_addresses: Vec<Address> = self.contracts.to_vec();
        let log_filter = log_filter.address(log_addresses);

        let logs = self.provider.clients[provider_index]
            .get_logs(&log_filter)
            .await
            .map_err(FromProviderError::DataNotFound)?;

        // The bloom filter returned a false positive, and we don't actually have matching logs.
        if logs.is_empty() {
            info!("{}: fetch => ignored (strict)", block_reference);
            return Ok(None);
        }

        let events_by_transaction = {
            let mut map: HashMap<(_, _), Vec<Log>> = HashMap::with_capacity(logs.len());
            for log in logs {
                if log.removed {
                    continue;
                }

                map.entry((
                    log.transaction_hash.unwrap(),
                    log.transaction_index.unwrap(),
                ))
                .and_modify(|logs| logs.push(log.clone()))
                .or_insert(vec![log]);
            }
            map
        };

        let transactions: Vec<TransactionInsert> = events_by_transaction
            .into_iter()
            .map(|((transaction_hash, transaction_index), logs)| {
                let transaction_hash = transaction_hash.to_lower_hex();
                let transaction_index = transaction_index as i32;

                let events: Vec<EventInsert> = logs
                    .into_iter()
                    .enumerate()
                    .map(|(transaction_log_index, log)| {
                        let data = serde_json::to_value(&log).unwrap();
                        EventInsert {
                            data,
                            log_index: log.log_index.unwrap() as usize,
                            transaction_log_index: transaction_log_index as i32,
                        }
                    })
                    .collect();

                trace!(
                    "{}: fetch => events: {} (transaction {}/{})",
                    block_reference,
                    events.len(),
                    transaction_index,
                    transaction_hash
                );

                TransactionInsert {
                    hash: transaction_hash,
                    index: transaction_index,
                    events,
                }
            })
            .collect();

        debug!(
            "{}: fetch => done (transactions: {})",
            block_reference,
            transactions.len()
        );

        Ok(Some(BlockInsert {
            chain_id: self.chain_id,
            hash: block_reference.hash,
            header: block.clone(),
            height: block_reference.height as i32,
            time: block_reference.timestamp,
            transactions,
        }))
    }
}

#[async_trait]
impl FetcherClient for EthFetcherClient {
    type BlockHandle = EthBlockHandle;
    type Context = EthContext;

    async fn create(
        pg_pool: sqlx::PgPool,
        join_set: &mut JoinSet<Result<(), IndexerError>>,
        context: EthContext,
    ) -> Result<Self, IndexerError> {
        let provider = RaceClient::new(
            context
                .urls
                .into_iter()
                .map(|url| ProviderBuilder::new().on_http(url))
                .collect(),
        );

        info!("fetching chain-id from node");
        let chain_id = provider.get_chain_id().await?;
        info!("fetched chain-id from node: {}", chain_id);

        let indexing_span = info_span!("indexer", chain_id = chain_id);
        async move {
            let mut tx = pg_pool.begin().await?;

            let chain_id = fetch_or_insert_chain_id_tx(&mut tx, chain_id.to_string())
                .await?
                .get_inner_logged();

            // TODO: remove once all data is based on new hubble tables
            let rows = loop {
                let rows = sqlx::query!(
                    r#"
                        SELECT c.address, COALESCE(cs.height, c.height - 1) as indexed_height
                        FROM v0.contracts c
                                LEFT JOIN hubble.contract_status cs
                                        ON c.chain_id = cs.internal_chain_id and c.address = cs.address
                        WHERE c.chain_id = $1 
                    "#,
                    chain_id.db
                )
                .fetch_all(tx.as_mut())
                .await?;

                if rows.is_empty() {
                    warn!("no contracts found to track, retrying in 20 seconds");
                    tokio::time::sleep(Duration::from_secs(20)).await;
                    continue;
                }
                break rows;
            };

            let lowest: u64 = rows
                .iter()
                .map(|row| row.indexed_height.expect("query to return indexed_height"))
                .min()
                .expect("contracts should exist in the db")
                .try_into()
                .expect("indexed_height should be positive");

            let highest: u64 = rows
                .iter()
                .map(|row| row.indexed_height.expect("query to return indexed_height"))
                .max()
                .expect("contracts should exist in the db")
                .try_into()
                .expect("indexed_height should be positive");

            if lowest != highest {
                info!("detected new contract. reload blocks that might be affected.");
                warn!("NOT YET IMPLEMENTED");
                // TODO: initiate repair
            };

            let contracts = rows
                .into_iter()
                .map(|row| {
                    row.address
                        .parse()
                        .expect("database should contain valid addresses")
                })
                .collect();

            tx.commit().await?;

            schedule_create_client_checker(pg_pool, join_set, provider.clone(), chain_id.db);

            Ok(EthFetcherClient {
                chain_id,
                provider,
                contracts,
            })
        }
        .instrument(indexing_span)
        .await
    }

    async fn fetch_single(
        &self,
        selection: BlockSelection,
        mode: FetchMode,
    ) -> Result<Self::BlockHandle, IndexerError> {
        self.fetch_single_with_provider(selection, mode, None).await
    }
}
