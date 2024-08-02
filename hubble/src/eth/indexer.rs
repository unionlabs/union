use core::fmt::Debug;
use std::{ops::Range, sync::Arc, time::Duration};

use backon::{ConstantBuilder, Retryable};
use color_eyre::Report;
use const_hex::ToHexExt;
use ethers::{
    abi::ethereum_types::Address,
    core::types::BlockId,
    providers::{Http, Provider},
    types::H256,
};
use futures::{stream::FuturesOrdered, FutureExt, StreamExt, TryFutureExt};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres};
use time::OffsetDateTime;
use tracing::{debug, info, info_span, warn, Instrument};
use url::Url;

use crate::{
    metrics,
    postgres::{self, ChainId, InsertMode},
    race_client::RaceClient,
};

const DEFAULT_CHUNK_SIZE: usize = 200;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub label: String,

    pub urls: Vec<Url>,

    /// How many blocks to fetch at the same time
    pub chunk_size: Option<usize>,

    #[serde(default)]
    pub mode: InsertMode,
}

/// Unit struct describing parametrization of associated types for Evm based chains.
pub struct Evm;

impl postgres::ChainType for Evm {
    type BlockHash = String;
    type BlockHeight = i32;
    type TransactionHash = String;
}

pub type PgLog<T> = postgres::Log<Evm, T>;

pub struct Indexer {
    range: Range<u64>,
    chain_id: ChainId,
    tasks: tokio::task::JoinSet<Result<(), Report>>,
    pool: PgPool,
    provider: RaceClient<Provider<Http>>,
    chunk_size: usize,
    mode: InsertMode,
    contracts: Vec<Address>,
}

impl Config {
    pub async fn indexer(self, pool: PgPool) -> Result<Indexer, Report> {
        let provider = RaceClient::new(
            self.urls
                .into_iter()
                .map(|url| Provider::<Http>::try_from(url.as_str()).unwrap())
                .collect(),
        );

        info!("fetching chain-id from node");
        let chain_id = (|| {
            debug!(?provider, "retry fetching chain-id from node");
            provider.get_chainid()
        })
        .retry(&crate::expo_backoff())
        .await?
        .as_u64();

        let indexing_span = info_span!("indexer", chain_id = chain_id);
        async move {
            let chain_id = postgres::fetch_or_insert_chain_id(&pool, chain_id.to_string())
                .await?
                .get_inner_logged();

            let rows = loop {
                let rows = sqlx::query!(
                    r#"SELECT address, indexed_height from v0.contracts where chain_id = $1"#,
                    chain_id.db
                )
                .fetch_all(&pool)
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
                .map(|row| row.indexed_height)
                .min()
                .expect("contracts should exist in the db")
                .try_into()
                .expect("indexed_height should be positive");

            let highest: u64 = rows
                .iter()
                .map(|row| row.indexed_height)
                .max()
                .expect("contracts should exist in the db")
                .try_into()
                .expect("indexed_height should be positive");

            let mode = if lowest != highest {
                info!("detected new contract. using upsert mode.");
                postgres::InsertMode::Upsert
            } else {
                self.mode
            };

            let contracts = rows
                .into_iter()
                .map(|row| {
                    row.address
                        .parse()
                        .expect("database should contain valid addresses")
                })
                .collect();

            let range = (lowest + 1)..u64::MAX;

            Ok(Indexer {
                range,
                tasks: tokio::task::JoinSet::new(),
                chain_id,
                pool,
                provider,
                chunk_size: self.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE),
                mode,
                contracts,
            })
        }
        .instrument(indexing_span)
        .await
    }
}

impl Indexer {
    /// Spawns two long running tasks, one which continuously indexes from current to head, and one which indexes from current-20 to current.
    /// The second routine handles fixing up block reorgs.
    pub async fn index(mut self) -> Result<(), Report> {
        let indexing_span = info_span!("indexer", chain_id = self.chain_id.canonical);
        async move {
            info!(
                "starting indexing from {} to {}",
                self.range.start, self.range.end
            );

            debug!("spawning main indexing routine");
            self.tasks.spawn(
                index_blocks(
                    self.pool.clone(),
                    self.range,
                    self.chain_id,
                    self.provider.clone(),
                    self.chunk_size,
                    self.mode,
                    self.contracts,
                )
                .in_current_span(),
            );

            self.tasks
                .join_next()
                .await
                .expect("set has at least one task")??;
            Ok(())
        }
        .instrument(indexing_span)
        .await
    }
}

#[derive(Debug, thiserror::Error)]
enum IndexBlockError {
    #[error("something went wrong, but can be resolved by retrying: {err:?}")]
    Retryable { height: u64, err: FromProviderError },
    #[error("unknown error: {0}")]
    Other(#[from] Report),
}

impl From<sqlx::Error> for IndexBlockError {
    fn from(err: sqlx::Error) -> Self {
        IndexBlockError::Other(Report::from(err))
    }
}

async fn index_blocks(
    pool: PgPool,
    range: Range<u64>,
    chain_id: ChainId,
    provider: RaceClient<Provider<Http>>,
    chunk_size: usize,
    mode: InsertMode,
    filter: Vec<Address>,
) -> Result<(), Report> {
    let filter = Arc::new(filter);
    let err = match index_blocks_by_chunk(
        pool.clone(),
        range.clone(),
        chain_id,
        provider.clone(),
        chunk_size,
        mode,
        filter.clone(),
    )
    .await
    {
        Ok(()) => return Ok(()),
        Err(err) => err,
    };

    match err {
        IndexBlockError::Retryable {
            height,
            err: FromProviderError::BlockNotFound,
        } => {
            // This most likely indicates we caught up indexing with the node. We now switch to
            // single block mode.
            index_blocks_by_chunk(
                pool.clone(),
                height..range.end,
                chain_id,
                provider.clone(),
                1,
                mode,
                filter.clone(),
            )
            .inspect_err(|e| {
                debug!(
                    ?e,
                    height, "err indexing block, not problematic if not found"
                )
            })
            .await?
        }
        err => return Err(err.into()),
    }
    Ok(())
}

async fn index_blocks_by_chunk(
    pool: PgPool,
    range: Range<u64>,
    chain_id: ChainId,
    provider: RaceClient<Provider<Http>>,
    chunk_size: usize,
    mode: InsertMode,
    filter: Arc<Vec<Address>>,
) -> Result<(), IndexBlockError> {
    let mut chunks = futures::stream::iter(range.into_iter()).chunks(chunk_size);

    while let Some(chunk) = chunks.next().await {
        if chunk.len() > 1 {
            info!(
                "indexing blocks for chunk: {}..{}",
                chunk.first().unwrap(),
                chunk.last().unwrap()
            );
        } else {
            info!("indexing block {}", chunk.first().unwrap(),);
        }

        let mut inserts = FuturesOrdered::from_iter(chunk.into_iter().map(|height| {
            let provider_clone = provider.clone();
            let filter_clone = filter.clone();
            (move || {
                BlockInsert::from_provider_retried_filtered(
                    chain_id,
                    height,
                    provider_clone.clone(),
                    filter_clone.clone(),
                )
            })
            .retry(
                &ConstantBuilder::default()
                    .with_delay(Duration::from_secs(1))
                    .with_max_times(30),
            )
            .when(|_| chunk_size == 1)
            .map(move |res| (height, res))
        }));

        while let Some((height, block)) = inserts.next().await {
            let block = match block {
                Err(FromProviderError::Other(err)) => {
                    return Err(IndexBlockError::Other(err));
                }
                Err(err) => {
                    return Err(IndexBlockError::Retryable { height, err });
                }
                Ok(Some(block)) => block,
                Ok(None) => {
                    // No relevant data for the current filter in this block. We still update the
                    // contracts.indexed_heights to avoid rechecking on crashes.
                    let mut tx = pool.begin().await?;
                    postgres::update_contracts_indexed_heights(
                        &mut tx,
                        filter
                            .iter()
                            .map(|addr| format!("0x{}", addr.encode_hex()))
                            .collect(),
                        filter.iter().map(|_| height.try_into().unwrap()).collect(),
                        chain_id,
                    )
                    .await?;
                    tx.commit().await?;
                    continue;
                }
            };
            debug!(?height, "attempting to insert");

            let mut tx = pool.begin().await?;

            match block.execute(&mut tx, mode).await {
                Err(err) => {
                    debug!(?err, "error executing block insert");
                    return Err(err.into());
                }
                Ok(info) => {
                    let updated = postgres::update_contracts_indexed_heights(
                        &mut tx,
                        filter
                            .iter()
                            .map(|addr| format!("0x{}", addr.encode_hex()))
                            .collect(),
                        filter.iter().map(|_| info.height.into()).collect(),
                        chain_id,
                    )
                    .await?;

                    // Hacky way to force hubble to crash and restart everything, allowing for it to refetch the addresses.
                    assert_eq!(
                        updated,
                        filter.len(),
                        "no contracts should be removed while hubble is running"
                    );

                    tx.commit().await?;
                    debug!(
                        height = info.height,
                        hash = info.hash,
                        num_transactions = info.num_tx,
                        num_events = info.num_events,
                        "indexed block"
                    );
                    metrics::BLOCK_COLLECTOR
                        .with_label_values(&[chain_id.canonical])
                        .inc();
                    metrics::TRANSACTION_COLLECTOR
                        .with_label_values(&[chain_id.canonical])
                        .inc_by(info.num_tx as u64);
                    metrics::EVENT_COLLECTOR
                        .with_label_values(&[chain_id.canonical])
                        .inc_by(info.num_events as u64);
                }
            }
        }
    }
    panic!("end of index_blocks_by_chunk should not occur");
}

pub struct InsertInfo {
    height: i32,
    hash: String,
    num_tx: usize,
    num_events: i32,
}

#[must_use]
#[derive(Debug, Clone)]
pub struct BlockInsert {
    pub chain_id: ChainId,
    pub hash: String,
    pub header: ethers::types::Block<H256>,
    pub height: i32,
    pub time: OffsetDateTime,
    pub transactions: Vec<TransactionInsert>,
}

#[derive(Serialize, Deserialize)]
pub struct LogData {
    pub transactions: Vec<TransactionInsert>,
    pub header: ethers::types::Block<H256>,
}

#[derive(Debug, thiserror::Error)]
pub enum FromProviderError {
    #[error("block not found")]
    BlockNotFound,
    #[error("data belonging to block not found")]
    DataNotFound(ethers::providers::ProviderError),
    #[error("something else went wrong")]
    Other(Report),
}

impl From<ethers::providers::ProviderError> for FromProviderError {
    fn from(error: ethers::providers::ProviderError) -> Self {
        Self::Other(Report::from(error))
    }
}

impl From<Report> for FromProviderError {
    fn from(error: Report) -> Self {
        Self::Other(error)
    }
}

impl From<BlockInsert> for PgLog<LogData> {
    fn from(block: BlockInsert) -> Self {
        PgLog {
            chain_id: block.chain_id,
            block_hash: block.hash.clone(),
            height: block.height,
            time: block.time,
            data: LogData {
                header: block.header,
                transactions: block.transactions,
            },
        }
    }
}

impl BlockInsert {
    pub async fn from_provider_retried_filtered<T: Into<BlockId> + Send + Sync + Debug + Clone>(
        chain_id: ChainId,
        id: T,
        provider: RaceClient<Provider<Http>>,
        filter: impl Into<Option<Arc<Vec<Address>>>>,
    ) -> Result<Option<Self>, FromProviderError> {
        let filter = filter.into();
        let id = id.into();
        debug!(?id, "fetching block from provider");
        (move || {
            Self::from_provider_filtered(chain_id, id, provider.clone(), filter.clone())
                .inspect_err(move |e| debug!(?e, ?id, "error fetching block from provider"))
        })
        .retry(&crate::expo_backoff())
        .when(|e| !matches!(e, FromProviderError::BlockNotFound))
        .await
        .map_err(Into::into)
    }

    async fn from_provider_filtered<T: Into<BlockId> + Send + Sync + Debug>(
        chain_id: ChainId,
        id: T,
        provider: RaceClient<Provider<Http>>,
        filter: Option<Arc<Vec<Address>>>,
    ) -> Result<Option<Self>, FromProviderError> {
        use std::collections::HashMap;

        use ethers::middleware::Middleware;

        let id = id.into();

        use ethers::{
            core::{abi::ethabi::ethereum_types::BloomInput, types::Filter},
            types::Log,
        };

        let block = provider
            .get_block(id)
            .await?
            .ok_or(FromProviderError::BlockNotFound)?;

        let provider = provider.fastest();

        // We check for a potential log match, which potentially avoids querying
        // eth_getLogs.
        if let (Some(bloom), Some(filter)) = (block.logs_bloom, &filter) {
            if !filter
                .iter()
                .any(|address| bloom.contains_input(BloomInput::Raw(address.as_bytes())))
            {
                return Ok(None);
            }
        }

        // We know now there is a potential match, we still apply a Filter to only
        // get the logs we want.
        let log_filter = Filter::new().select(block.hash.unwrap());

        let log_filter = if let Some(ref filter) = filter {
            let addresses: Vec<_> = filter.iter().cloned().collect();
            log_filter.address(addresses)
        } else {
            log_filter
        };

        let logs = provider
            .get_logs(&log_filter)
            .await
            .map_err(FromProviderError::DataNotFound)?;

        // The bloom filter returned a false positive, and we don't actually have matching logs.
        if logs.is_empty() && filter.is_some() {
            return Ok(None);
        }

        let partitioned = {
            let mut map: HashMap<(_, _), Vec<Log>> = HashMap::with_capacity(logs.len());
            for log in logs {
                if log.removed.unwrap_or_default() {
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

        let result: Result<Option<Self>, Report> = try {
            let ts = block.time().unwrap_or_default().timestamp();
            let time = OffsetDateTime::from_unix_timestamp(ts)?;
            let block_hash = block.hash.unwrap().to_lower_hex();
            let height: i32 = block.number.unwrap().as_u32().try_into()?;

            let transactions = partitioned
                .into_iter()
                .map(|((transaction_hash, transaction_index), logs)| {
                    let transaction_hash = transaction_hash.to_lower_hex();
                    let transaction_index = transaction_index.as_u32() as i32;

                    let events = logs
                        .into_iter()
                        .enumerate()
                        .map(|(transaction_log_index, log)| {
                            let data = serde_json::to_value(&log).unwrap();
                            EventInsert {
                                data,
                                log_index: log.log_index.unwrap().as_usize(),
                                transaction_log_index: transaction_log_index as i32,
                            }
                        })
                        .collect();
                    TransactionInsert {
                        hash: transaction_hash,
                        index: transaction_index,
                        events,
                    }
                })
                .collect();

            Some(BlockInsert {
                chain_id,
                hash: block_hash,
                header: block,
                height,
                time,
                transactions,
            })
        };
        result.map_err(Into::into)
    }

    /// Handles inserting the block data and transactions as a log.
    async fn execute(
        self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        mode: InsertMode,
    ) -> Result<InsertInfo, Report> {
        let num_tx = self.transactions.len();
        let num_events = self
            .transactions
            .iter()
            .map(|tx| tx.events.len() as i32)
            .sum();

        let (height, hash) = (self.height, self.hash.clone());

        postgres::insert_batch_logs(tx, std::iter::once(self.into()), mode).await?;

        Ok(InsertInfo {
            height,
            hash,
            num_tx,
            num_events,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionInsert {
    hash: String,
    index: i32,
    events: Vec<EventInsert>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventInsert {
    data: serde_json::Value,
    log_index: usize,
    transaction_log_index: i32,
}

pub trait ToLowerHex {
    fn to_lower_hex(&self) -> String;
}

impl ToLowerHex for H256 {
    fn to_lower_hex(&self) -> String {
        format!("{:#x}", self)
    }
}
