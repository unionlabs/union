use core::fmt::Debug;
use std::{ops::Range, sync::Arc, time::Duration};

use alloy::{
    eips::BlockId,
    primitives::{Address, BloomInput, FixedBytes},
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::types::{Block, BlockTransactionsKind, Filter, Log},
    transports::{
        http::{Client, Http},
        RpcError, TransportErrorKind,
    },
};
use backon::{ConstantBuilder, Retryable};
use chrono::{offset::LocalResult, TimeZone, Utc};
use color_eyre::Report;
use const_hex::ToHexExt;
use futures::{stream::FuturesOrdered, FutureExt, StreamExt, TryFutureExt};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::{debug, info, info_span, warn, Instrument};
use url::Url;

use crate::{
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
    provider: RaceClient<RootProvider<Http<Client>>>,
    chunk_size: usize,
    mode: InsertMode,
    contracts: Vec<Address>,
}

impl Config {
    pub async fn indexer(self, pool: PgPool) -> Result<Indexer, Report> {
        let provider = RaceClient::new(
            self.urls
                .into_iter()
                .map(|url| ProviderBuilder::new().on_http(url))
                .collect(),
        );

        info!("fetching chain-id from node");
        let chain_id = (|| {
            debug!(?provider, "retry fetching chain-id from node");
            provider.get_chain_id()
        })
        .retry(&crate::expo_backoff())
        .await?;

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
    provider: RaceClient<RootProvider<Http<Client>>>,
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
    provider: RaceClient<RootProvider<Http<Client>>>,
    chunk_size: usize,
    mode: InsertMode,
    filter: Arc<Vec<Address>>,
) -> Result<(), IndexBlockError> {
    use itertools::{Either, Itertools};

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

        let mut inserts = FuturesOrdered::from_iter(chunk.iter().copied().map(|height| {
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
        }))
        .ready_chunks(chunk.len());

        // The below algorithm is a little complex in syntax but relatively simple:
        // 1. We take all ready futures from inserts (1 or more).
        // 2. We partition such that a chunk of [ok, ok, err, ok, err] becomes blocks: [ok, ok], errs: [err].
        // 3. if the blocks contain Some(block) (when the filter matches), we insert them.
        // 4. we update the indexed_height to blocks.last().height.
        // 5. if err is not empty, throw the error afterwards.
        while let Some(blocks) = inserts.next().await {
            let (blocks, mut err): (Vec<(_, Option<PgLog<_>>)>, Vec<_>) = blocks
                .into_iter()
                .take_while_inclusive(|(_, block)| block.is_ok())
                .partition_map(|(h, result)| match result {
                    Ok(block) => Either::Left((h, block.map(Into::into))),
                    Err(err) => Either::Right((h, err)),
                });

            if !blocks.is_empty() {
                let mut tx = pool.begin().await?;
                let height = blocks.last().unwrap().0.try_into().unwrap();
                let blocks = blocks.into_iter().filter_map(|(_, b)| b);

                postgres::insert_batch_logs(&mut tx, blocks, mode).await?;
                let updated = postgres::update_contracts_indexed_heights(
                    &mut tx,
                    filter
                        .iter()
                        .map(|addr| format!("0x{}", addr.encode_hex()))
                        .collect(),
                    filter.iter().map(|_| height).collect(),
                    chain_id,
                )
                .await?;
                assert_eq!(
                    updated,
                    filter.len(),
                    "no contracts should be removed while hubble is running"
                );
                tx.commit().await?;
            }

            if !err.is_empty() {
                match err.remove(0) {
                    (_, FromProviderError::Other(err)) => {
                        return Err(IndexBlockError::Other(err));
                    }
                    (height, err) => {
                        return Err(IndexBlockError::Retryable { height, err });
                    }
                }
            }
        }
    }
    Ok(())
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

#[derive(Serialize, Deserialize)]
pub struct LogData {
    pub transactions: Vec<TransactionInsert>,
    pub header: Block,
}

#[derive(Debug, thiserror::Error)]
pub enum FromProviderError {
    #[error("block not found")]
    BlockNotFound,
    #[error("data belonging to block not found")]
    DataNotFound(RpcError<TransportErrorKind>),
    #[error("something else went wrong")]
    Other(Report),
}

impl From<RpcError<TransportErrorKind>> for FromProviderError {
    fn from(error: RpcError<TransportErrorKind>) -> Self {
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
        provider: RaceClient<RootProvider<Http<Client>>>,
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
        provider: RaceClient<RootProvider<Http<Client>>>,
        filter: Option<Arc<Vec<Address>>>,
    ) -> Result<Option<Self>, FromProviderError> {
        use std::collections::HashMap;

        let id = id.into();

        let block = provider
            .get_block(id, BlockTransactionsKind::Full)
            .await?
            .ok_or(FromProviderError::BlockNotFound)?;

        let provider = provider.fastest();

        // We check for a potential log match, which potentially avoids querying
        // eth_getLogs.
        let bloom = block.header.logs_bloom;
        if let Some(filter) = &filter {
            if !filter
                .iter()
                .any(|address| bloom.contains_input(BloomInput::Raw(&address.into_array())))
            {
                return Ok(None);
            }
        }

        // We know now there is a potential match, we still apply a Filter to only
        // get the logs we want.
        let log_filter = Filter::new().select(block.header.hash.unwrap());

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

        let result: Result<Option<Self>, Report> = try {
            let timestamp_i64 = block.header.timestamp as i64;
            let datetime = match Utc.timestamp_opt(timestamp_i64, 0) {
                LocalResult::Single(datetime) => datetime,
                _ => Utc::now(),
            };
            let ts = datetime.timestamp();
            let time = OffsetDateTime::from_unix_timestamp(ts)?;
            let block_hash = block.header.hash.unwrap().to_lower_hex();
            let height: i32 = block.header.number.unwrap() as i32;

            let transactions = partitioned
                .into_iter()
                .map(|((transaction_hash, transaction_index), logs)| {
                    let transaction_hash = transaction_hash.to_lower_hex();
                    let transaction_index = transaction_index as i32;

                    let events = logs
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

impl ToLowerHex for FixedBytes<32> {
    fn to_lower_hex(&self) -> String {
        format!("{:#x}", self)
    }
}
