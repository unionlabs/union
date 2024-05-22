use std::{ops::Range, time::Duration};

use backon::Retryable;
use color_eyre::Report;
use ethers::{
    providers::{Http, Middleware, Provider},
    types::{BlockNumber, H256},
};
use futures::{
    stream::{self, FuturesOrdered},
    StreamExt, TryFutureExt, TryStreamExt,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres};
use time::OffsetDateTime;
use tracing::{debug, info};
use url::Url;

const DEFAULT_CHUNK_SIZE: usize = 200;

use crate::{
    metrics,
    postgres::{self, ChainId},
};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub url: Url,

    /// The height from which we start indexing
    pub start_height: Option<i32>,

    /// How many blocks to fetch at the same time
    pub chunk_size: Option<usize>,
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
    provider: Provider<Http>,
    chunk_size: usize,
}

impl Config {
    pub async fn indexer(self, pool: PgPool) -> Result<Indexer, Report> {
        let provider = Provider::<Http>::try_from(self.url.clone().as_str()).unwrap();

        info!("fetching chain-id from node");
        let chain_id = (|| {
            debug!(?provider, "retry fetching chain-id from node");
            provider.get_chainid()
        })
        .retry(&crate::expo_backoff())
        .await?
        .as_u64();

        let chain_id = postgres::fetch_or_insert_chain_id(&pool, chain_id.to_string())
            .await?
            .get_inner_logged();

        let current = sqlx::query!(
            r#"SELECT height 
            FROM "v0"."logs" 
            WHERE chain_id = $1 
            ORDER BY height DESC 
            NULLS LAST 
            LIMIT 1"#,
            chain_id.db
        )
        .fetch_optional(&pool)
        .await?
        .map(|block| {
            if block.height == 0 {
                info!(?self.start_height, "no block found, starting at configured start height, or 0 if not defined");
                self.start_height.unwrap_or_default()
            } else {
                info!("block {} found, starting max(start_height, {})", block.height, block.height);
                block.height.max(self.start_height.unwrap_or_default())
            }
        })
        .unwrap_or_default() as u64;

        let range = current..u64::MAX;

        Ok(Indexer {
            range,
            tasks: tokio::task::JoinSet::new(),
            chain_id,
            pool,
            provider,
            chunk_size: self.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE),
        })
    }
}

impl Indexer {
    /// Spawns two long running tasks, one which continuously indexes from current to head, and one which indexes from current-20 to current.
    /// The second routine handles fixing up block reorgs.
    pub async fn index(mut self) -> Result<(), Report> {
        info!(
            self.chain_id.canonical,
            "starting indexing from {} to {}", self.range.start, self.range.end
        );

        debug!(self.chain_id.canonical, "spawning main indexing routine");
        self.tasks.spawn(index_blocks(
            self.pool.clone(),
            self.range,
            self.chain_id,
            self.provider.clone(),
            self.chunk_size,
        ));

        debug!(self.chain_id.canonical, "spawning fork indexing routine");
        self.tasks.spawn(reindex_blocks(
            self.pool.clone(),
            self.chain_id,
            self.provider.clone(),
        ));

        self.tasks
            .join_next()
            .await
            .expect("set has at least one task")??;
        Ok(())
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
    provider: Provider<Http>,
    chunk_size: usize,
) -> Result<(), Report> {
    let err = match index_blocks_by_chunk(
        pool.clone(),
        range.clone(),
        chain_id,
        provider.clone(),
        chunk_size,
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
            index_blocks_by_chunk(pool, height..range.end, chain_id, provider, 1).await?;
        }
        err => return Err(err.into()),
    }
    Ok(())
}

async fn index_blocks_by_chunk(
    pool: PgPool,
    range: Range<u64>,
    chain_id: ChainId,
    provider: Provider<Http>,
    chunk_size: usize,
) -> Result<(), IndexBlockError> {
    let mut chunks = futures::stream::iter(range.into_iter()).chunks(chunk_size);

    while let Some(chunk) = chunks.next().await {
        if chunk.len() > 1 {
            info!(
                chain_id.canonical,
                "indexing blocks for chunk: {}..{}",
                chunk.first().unwrap(),
                chunk.last().unwrap()
            );
        } else {
            info!(
                chain_id.canonical,
                "indexing block {}",
                chunk.first().unwrap(),
            );
        }

        let mut tx = pool.begin().await.map_err(Report::from)?;

        let mut inserts = FuturesOrdered::from_iter(chunk.into_iter().map(|height| {
            // Hack workaround because partial move for async blocks isn't possible.
            async fn from_provider_retried(
                chain_id: ChainId,
                height: u64,
                provider: &Provider<Http>,
            ) -> (u64, Result<(usize, BlockInsert), FromProviderError>) {
                (
                    height,
                    BlockInsert::from_provider_retried(chain_id, height, provider).await,
                )
            }
            from_provider_retried(chain_id, height, &provider)
        }));

        while let Some((height, block)) = inserts.next().await {
            let (_tries, block) = match block {
                Err(FromProviderError::Other(err)) => {
                    tx.commit().await?;
                    return Err(IndexBlockError::Other(err));
                }
                Err(err) => {
                    tx.commit().await?;
                    return Err(IndexBlockError::Retryable { height, err });
                }
                Ok(block) => block,
            };

            match block.execute(&mut tx).await {
                Err(err) => {
                    tx.rollback().await?;
                    return Err(err.into());
                }
                Ok(info) => {
                    debug!(
                        chain_id.canonical,
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
        tx.commit().await?;
    }
    panic!("end of index_blocks_by_chunk should not occur");
}

/// A worker routine which continuously re-indexes the last 20 blocks into `PgLogs`.
async fn reindex_blocks(
    pool: PgPool,
    chain_id: ChainId,
    provider: Provider<Http>,
) -> Result<(), Report> {
    loop {
        // Set start to the current height so we can continue re-indexing.
        let current = fetch_latest_height(&pool, chain_id).await?;
        let latest = provider
            .get_block(BlockNumber::Latest)
            .await?
            .expect("provider should have latest block");
        if latest.number.unwrap().as_u32() - current as u32 > 32 {
            tokio::time::sleep(Duration::from_secs(12 * 32)).await;
            continue;
        }

        let tx = pool.begin().await?;
        let chunk = (current - 20)..current;
        let inserts = FuturesOrdered::from_iter(chunk.into_iter().map(|height| {
            BlockInsert::from_provider_retried(chain_id, height as u64, &provider)
                .map_err(Report::from)
        }));
        inserts
            .try_fold(tx, |mut tx, (_, block)| async move {
                let log = PgLog {
                    chain_id: block.chain_id,
                    block_hash: block.hash.clone(),
                    height: block.height,
                    time: block.time,
                    data: LogData {
                        header: block.header,
                        transactions: block.transactions,
                    },
                };
                postgres::upsert_log(&mut tx, log).await?;
                Ok(tx)
            })
            .await?;
    }
}

async fn fetch_latest_height(pool: &PgPool, chain_id: ChainId) -> Result<i32, sqlx::Error> {
    let latest = sqlx::query!(
        "SELECT height FROM \"v0\".logs 
        WHERE chain_id = $1 
        ORDER BY height DESC 
        NULLS LAST 
        LIMIT 1",
        chain_id.db
    )
    .fetch_optional(pool)
    .await?
    .map(|block| block.height)
    .unwrap_or_default();
    Ok(latest)
}

pub struct InsertInfo {
    height: i32,
    hash: String,
    num_tx: usize,
    num_events: i32,
}

pub struct BlockInsert {
    chain_id: ChainId,
    hash: String,
    header: ethers::types::Block<H256>,
    height: i32,
    time: OffsetDateTime,
    transactions: Vec<TransactionInsert>,
}

#[derive(Serialize, Deserialize)]
pub struct LogData {
    pub transactions: Vec<TransactionInsert>,
    pub header: ethers::types::Block<H256>,
}

#[derive(Debug, thiserror::Error)]
enum FromProviderError {
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

impl BlockInsert {
    async fn from_provider_retried(
        chain_id: ChainId,
        height: u64,
        provider: &Provider<Http>,
    ) -> Result<(usize, Self), FromProviderError> {
        loop {
            debug!(chain_id=chain_id.canonical, height, "fetching block from provider");
            (|| {
                Self::from_provider(chain_id, height, provider)
                    .inspect_err(|e| debug!(?e, chain_id=chain_id.canonical, height, "error fetching block from provider"))
            })
            .retry(&crate::expo_backoff())
            .await?;
        }
    }

    async fn from_provider(
        chain_id: ChainId,
        height: u64,
        provider: &Provider<Http>,
    ) -> Result<Self, FromProviderError> {
        let block = provider
            .get_block(height)
            .await?
            .ok_or(FromProviderError::BlockNotFound)?;
        let mut receipts = provider
            .get_block_receipts(height)
            .await
            .map_err(FromProviderError::DataNotFound)?;

        let result: Result<Self, Report> = try {
            let ts = block.time().unwrap_or_default().timestamp();
            let time = OffsetDateTime::from_unix_timestamp(ts)?;
            let block_hash = block.hash.unwrap().to_lower_hex();
            let height: i32 = block.number.unwrap().as_u32().try_into()?;
            receipts.sort_by(|a, b| a.transaction_index.cmp(&b.transaction_index));

            let transactions = receipts
                .into_iter()
                .map(|receipt| {
                    let transaction_hash = receipt.transaction_hash.to_lower_hex();
                    let transaction_index = receipt.transaction_index.as_u32() as i32;

                    let events = receipt
                        .clone()
                        .logs
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
                        data: receipt,
                        index: transaction_index,
                        events,
                    }
                })
                .collect();

            BlockInsert {
                chain_id,
                hash: block_hash,
                header: block,
                height,
                time,
                transactions,
            }
        };
        result.map_err(Into::into)
    }

    /// Handles inserting the block data and transactions as a log.
    async fn execute(self, tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<InsertInfo, Report> {
        let num_tx = self.transactions.len();
        let num_events = self
            .transactions
            .iter()
            .map(|tx| tx.events.len() as i32)
            .sum();

        let log = PgLog {
            chain_id: self.chain_id,
            block_hash: self.hash.clone(),
            height: self.height,
            time: self.time,
            data: LogData {
                header: self.header,
                transactions: self.transactions,
            },
        };

        postgres::insert_batch_logs(tx, stream::iter(Some(log))).await?;

        Ok(InsertInfo {
            height: self.height,
            hash: self.hash,
            num_tx,
            num_events,
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TransactionInsert {
    hash: String,
    data: ethers::types::TransactionReceipt,
    index: i32,
    events: Vec<EventInsert>,
}

#[derive(Clone, Serialize, Deserialize)]
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
