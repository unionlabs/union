use std::{collections::HashMap, fmt::Display};

use alloy::{
    eips::BlockId,
    primitives::{Address, BloomInput, FixedBytes},
    rpc::types::{Block, BlockTransactionsKind, Filter, Log},
};
use axum::async_trait;
use color_eyre::eyre::Report;
use itertools::Itertools;
use time::OffsetDateTime;
use tokio::task::JoinSet;
use tracing::{debug, info, info_span, trace, Instrument};

use crate::{
    indexer::{
        api::{
            BlockHeight, BlockRange, BlockReference, BlockSelection, FetchMode, FetcherClient,
            IndexerError,
        },
        eth::{
            block_handle::{
                BlockDetails, BlockInsert, EthBlockHandle, EventInsert, TransactionInsert,
            },
            context::EthContext,
            create_client_tracker::schedule_create_client_checker,
            postgres::transaction_filter,
            provider::{Provider, RpcProviderId},
        },
    },
    postgres::{fetch_or_insert_chain_id_tx, ChainId},
};

pub trait ToLowerHex {
    fn to_lower_hex(&self) -> String;
}

impl ToLowerHex for FixedBytes<32> {
    fn to_lower_hex(&self) -> String {
        format!("{:#x}", self)
    }
}

trait BlockReferenceProvider {
    fn block_reference(&self) -> Result<BlockReference, Report>;
}

impl BlockReferenceProvider for Block {
    fn block_reference(&self) -> Result<BlockReference, Report> {
        Ok(BlockReference {
            height: self.header.number,
            hash: self.header.hash.to_lower_hex(),
            timestamp: OffsetDateTime::from_unix_timestamp(self.header.timestamp as i64)
                .map_err(|err| IndexerError::ProviderError(err.into()))?,
        })
    }
}

#[derive(Clone)]
pub struct EthFetcherClient {
    pub chain_id: ChainId,
    pub provider: Provider,
    pub transaction_filter: TransactionFilter,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct TransactionFilter {
    pub address_filters: Vec<AddressFilter>,
}
impl TransactionFilter {
    pub(crate) fn addresses_at(&self, height: BlockHeight) -> Vec<Address> {
        self.address_filters
            .iter()
            .filter(|address_filter| address_filter.block_range.contains(height))
            .map(|address_filter| address_filter.address)
            .collect_vec()
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct AddressFilter {
    pub block_range: BlockRange,
    pub address: Address,
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
        provider_id: Option<RpcProviderId>,
    ) -> Result<EthBlockHandle, IndexerError> {
        let block = self
            .provider
            .get_block(
                match selection {
                    BlockSelection::LastFinalized => BlockId::finalized(),
                    BlockSelection::Height(height) => BlockId::number(height),
                },
                BlockTransactionsKind::Full,
                provider_id,
            )
            .await;

        match block {
            Ok(rpc_result) => match rpc_result {
                Some(result) => {
                    let block = result.response;
                    debug!(
                        "{}: fetched (provider index: {:?})",
                        selection, result.provider_id
                    );

                    Ok(EthBlockHandle {
                        reference: block.block_reference()?,
                        details: match mode {
                            FetchMode::Lazy => BlockDetails::Lazy(block),
                            FetchMode::Eager => BlockDetails::Eager(
                                self.fetch_details(&block, result.provider_id).await?,
                            ),
                        },
                        eth_client: self.clone(),
                        provider_id: result.provider_id,
                    })
                }
                None => {
                    info!("{}: does not exist", selection);

                    Err(IndexerError::NoBlock(selection))
                }
            },
            Err(report) => {
                info!("{}: error: {}", selection, report);

                Err(report.into())
            }
        }
    }

    pub async fn fetch_details(
        &self,
        block: &Block,
        provider_id: RpcProviderId,
    ) -> Result<Option<BlockInsert>, IndexerError> {
        let block_reference = block.block_reference()?;

        info!("{}: fetch", block_reference);

        let contract_addresses = self.transaction_filter.addresses_at(block_reference.height);
        debug!(
            "{}: contract-addresses: {:?}",
            block_reference, &contract_addresses
        );
        // We check for a potential log match, which potentially avoids querying
        // eth_getLogs.
        let bloom = block.header.logs_bloom;

        if contract_addresses.iter().all(|contract_address| {
            !bloom.contains_input(BloomInput::Raw(&contract_address.into_array()))
        }) {
            info!("{}: ignored (bloom)", block_reference);
            return Ok(None);
        }

        // We know now there is a potential match, we still apply a Filter to only
        // get the logs we want.
        let log_filter = Filter::new().select(block.header.hash);
        let log_filter = log_filter.address(contract_addresses);

        let logs = self
            .provider
            .get_logs(&log_filter, Some(provider_id))
            .await
            .map_err(|err| IndexerError::ProviderError(err.into()))?
            .response;

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
        let provider = Provider::new(context.urls);

        info!("fetching chain-id from node");
        let chain_id = provider.get_chain_id(None).await?.response;
        info!("fetched chain-id from node: {}", chain_id);

        let indexing_span = info_span!("indexer", chain_id = chain_id);
        async move {
            let mut tx = pg_pool.begin().await?;

            let chain_id = fetch_or_insert_chain_id_tx(&mut tx, chain_id.to_string())
                .await?
                .get_inner_logged();

            let transaction_filter = transaction_filter(&pg_pool, chain_id.db).await?;
            debug!("transaction-filter: {:?}", &transaction_filter);

            tx.commit().await?;

            if context.client_tracking {
                info!("scheduling client tracking");
                schedule_create_client_checker(pg_pool, join_set, provider.clone(), chain_id.db);
            } else {
                info!("client tracking disabled");
            }

            Ok(EthFetcherClient {
                chain_id,
                provider,
                transaction_filter,
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
