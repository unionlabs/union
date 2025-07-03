use std::{collections::HashMap, fmt::Display};

use alloy::{
    eips::BlockId,
    network::AnyRpcBlock,
    primitives::BloomInput,
    rpc::types::{BlockTransactionsKind, Filter, Log},
};
use alloy_primitives::Address;
use axum::async_trait;
use color_eyre::eyre::Report;
use itertools::Itertools;
use time::OffsetDateTime;
use tokio::task::JoinSet;
use tracing::{debug, error, info, info_span, trace, warn, Instrument};

use crate::{
    github_client::GitCommitHash,
    indexer::{
        api::{BlockReference, BlockSelection, FetchMode, FetcherClient, IndexerError},
        ethereum::{
            abi::AbiRegistration,
            block_handle::{
                BlockDetails, BlockInsert, EthBlockHandle, EventInsert, TransactionInsert,
            },
            context::EthContext,
            mapping::legacy::ToLowerHex,
            postgres::{
                ensure_abi_dependency, generated_abi, get_abi_registration, update_contract_abi,
            },
            provider::{Provider, RpcProviderId},
        },
        record::InternalChainId,
    },
    postgres::{fetch_chain_id_tx, ChainId},
};

pub trait BlockReferenceProvider {
    fn block_reference(&self) -> Result<BlockReference, Report>;
}

impl BlockReferenceProvider for AnyRpcBlock {
    fn block_reference(&self) -> Result<BlockReference, Report> {
        Ok(BlockReference {
            height: self.header.number,
            hash: self.header.hash.to_lower_hex(),
            timestamp: OffsetDateTime::from_unix_timestamp(
                self.header.timestamp.try_into().unwrap(),
            )
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

#[derive(Clone, Debug)]
pub struct TransactionFilter {
    pub chain_id: ChainId,
    pub pg_pool: sqlx::PgPool,
}
impl TransactionFilter {
    pub(crate) async fn abi_registration_at(
        &self,
        height: crate::indexer::event::types::BlockHeight,
    ) -> Result<AbiRegistration, IndexerError> {
        get_abi_registration(
            &mut self.pg_pool.begin().await?,
            self.chain_id.db.into(),
            height,
        )
        .await
    }

    pub(crate) async fn update_contract_abi(
        &self,
        internal_chain_id: InternalChainId,
        contract: Address,
        abi: String,
    ) -> Result<bool, IndexerError> {
        let mut tx = self.pg_pool.begin().await?;
        let result = update_contract_abi(&mut tx, internal_chain_id, contract, abi).await?;
        tx.commit().await?;
        Ok(result)
    }

    pub(crate) async fn register_abi_dependency(
        &self,
        commit: GitCommitHash,
    ) -> Result<bool, IndexerError> {
        let mut tx = self.pg_pool.begin().await?;
        let result = ensure_abi_dependency(&mut tx, commit).await?;
        tx.commit().await?;
        Ok(result)
    }

    pub(crate) async fn generated_abi(
        &self,
        commit: GitCommitHash,
        description: String,
    ) -> Result<Option<String>, IndexerError> {
        generated_abi(&mut self.pg_pool.begin().await?, commit, description).await
    }
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
        block: &AnyRpcBlock,
        provider_id: RpcProviderId,
    ) -> Result<Option<BlockInsert>, IndexerError> {
        let block_reference = block.block_reference()?;

        info!("{}: fetch", block_reference);

        let abi_registration = self
            .transaction_filter
            .abi_registration_at(block_reference.height.into())
            .await?;
        debug!(
            "{}: contract-addresses: {}",
            block_reference, &abi_registration
        );
        // We check for a potential log match, which potentially avoids querying
        // eth_getLogs.
        let bloom = block.header.logs_bloom;

        if abi_registration
            .addresses()
            .into_iter()
            .all(|contract_address| {
                !bloom.contains_input(BloomInput::Raw(&contract_address.into_array()))
            })
        {
            info!("{}: ignored (bloom)", block_reference);
            return Ok(None);
        }

        // We know now there is a potential match, we still apply a Filter to only
        // get the logs we want.
        let log_filter = Filter::new().select(block.header.hash);
        let log_filter = log_filter.address(abi_registration.addresses());

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
            for log in &logs {
                if log.removed {
                    continue;
                }

                map.entry((
                    log.transaction_hash.unwrap(),
                    log.transaction_index.unwrap(),
                ))
                .and_modify(|logs| logs.push(log.clone()))
                .or_insert(vec![log.clone()]);
            }
            map
        };

        let transactions: Vec<TransactionInsert> = events_by_transaction
            .into_iter()
            .map(|((transaction_hash, transaction_index), logs)| {
                let transaction_hash = transaction_hash.to_lower_hex();
                let transaction_index: i32 = transaction_index.try_into().unwrap();

                let events: Vec<EventInsert> = logs
                    .iter()
                    .enumerate()
                    .map(|(transaction_log_index, log)| {
                        let data = serde_json::to_value(log).unwrap();
                        EventInsert {
                            data,
                            log_index: log.log_index.expect("log_index").try_into().unwrap(),
                            transaction_log_index: transaction_log_index.try_into().unwrap(),
                        }
                    })
                    .sorted_by_key(|e| e.log_index)
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
            .sorted_by_key(|t| t.index)
            .collect();

        debug!(
            "{}: fetch => done (transactions: {})",
            block_reference,
            transactions.len()
        );

        // do ucs transformation
        let ucs_events = match self.transform_logs_to_ucs_events(&abi_registration, block, &logs) {
            Ok(events) => Ok(events),
            Err(IndexerError::AbiCannotParse(
                err,
                internal_chain_id,
                contract,
                description,
                commit,
            )) => {
                match self
                    .transaction_filter
                    .generated_abi(commit.clone(), description.clone())
                    .await?
                {
                    Some(abi) => {
                        match self
                            .transaction_filter
                            .update_contract_abi(internal_chain_id, contract, abi)
                            .await
                        {
                            Ok(true) => {
                                info!("error decoding message ({err}): contract abi updated")
                            }
                            Ok(false) => {
                                warn!("error decoding message ({err}): contract abi already up to date")
                            }
                            Err(err) => {
                                error!("error decoding message ({err}): cannot update contract abi")
                            }
                        }
                    }
                    None => {
                        info!("error decoding message ({err}): no abi definition found with commit {commit} => ensure it's registered");

                        match self
                            .transaction_filter
                            .register_abi_dependency(commit.clone())
                            .await
                        {
                            Ok(true) => {
                                info!("error decoding message ({err}): ensured that abi dependency ({commit}) is registered")
                            }
                            Ok(false) => {
                                warn!(
                                    "error decoding message: abi dependency was already registered"
                                )
                            }
                            Err(err) => {
                                error!("error registering abi dependency: {err}");
                            }
                        };
                    }
                };

                Err(IndexerError::AbiCannotParse(
                    err,
                    internal_chain_id,
                    contract,
                    description,
                    commit,
                ))
            }
            Err(other) => Err(other),
        }?;

        debug!(
            "{}: fetch => converted (events: {})",
            block_reference,
            ucs_events.len()
        );

        Ok(Some(BlockInsert {
            chain_id: self.chain_id,
            hash: block_reference.hash,
            header: block.clone(),
            height: block_reference.height.try_into().unwrap(),
            time: block_reference.timestamp,
            transactions,
            ucs_events,
        }))
    }
}

#[async_trait]
impl FetcherClient for EthFetcherClient {
    type BlockHandle = EthBlockHandle;
    type Context = EthContext;

    async fn create(
        pg_pool: sqlx::PgPool,
        _join_set: &mut JoinSet<Result<(), IndexerError>>,
        context: EthContext,
    ) -> Result<Self, IndexerError> {
        let provider = Provider::new(context.rpc_urls);

        info!("fetching chain-id from node");
        let chain_id = provider.get_chain_id(None).await?.response;
        info!("fetched chain-id from node: {}", chain_id);

        let indexing_span = info_span!("indexer", chain_id = chain_id);
        async move {
            let mut tx = pg_pool.begin().await?;

            let chain_id = fetch_chain_id_tx(&mut tx, chain_id.to_string()).await?;
            info!("fetched chain-id from database: {}", chain_id);

            tx.commit().await?;

            let transaction_filter = TransactionFilter { chain_id, pg_pool };

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
