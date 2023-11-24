//! The Ethereum indexing algorithm is relatively complicated compared to the Cosmos indexer
//! due to the fact that Ethereum does not have single-slot finality, hence we must handle
//! reorgs.
//!
//! # Algorithm
//!
//! 1. Index head eagerly.
//! 2. Index from the latest finalized block backwards to the last finalized block in the DB.
//! 3. Remove all data associated with uncle blocks.
//!
//! # Parsing
//!
//! Since EthAbi is not self-describing, we need a separate parsing step to transform the data
//! for consumption. This indexer does not handle parsing, just creating an up-to-date view of logs, transactions and blocks.
//!
use std::{ops::Range, time::Duration};

use async_stream::stream;
use color_eyre::Report;
use ethers::{
    providers::{Http, Middleware, Provider},
    types::{Block, TransactionReceipt, H256},
};
use futures::{Stream, TryStreamExt};
use time::OffsetDateTime;
use tracing::info;
use url::Url;

pub const STAGE_TRANSACTION: i16 = 1;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    url: Url,
    range: Option<Range<u64>>,
}

impl Config {
    pub async fn index(&self, pool: sqlx::PgPool) -> Result<(), Report> {
        let provider = Provider::<Http>::try_from(self.url.clone().as_str()).unwrap();
        let chain_id = provider.get_chainid().await?;

        info!("fetching chain-id from node");
        let chain_id = if let Some(chain_id) = sqlx::query!(
            "SELECT id FROM \"v0\".chains WHERE chain_id = $1 LIMIT 1",
            chain_id.to_string()
        )
        .fetch_optional(&pool)
        .await?
        {
            chain_id.id
        } else {
            info!("no chain-id found, creating...");
            sqlx::query!(
                "INSERT INTO \"v0\".chains (name, chain_id) VALUES ($1, $1) RETURNING id",
                chain_id.to_string()
            )
            .fetch_one(&pool)
            .await?
            .id
        };

        let lowest = sqlx::query!("SELECT height FROM \"v0\".blocks WHERE chain_id = $1 ORDER BY height ASC NULLS LAST LIMIT 1", chain_id).fetch_optional(&pool).await?.map(|block| block.height).unwrap_or_default() as u64;
        let current = sqlx::query!("SELECT height FROM \"v0\".blocks WHERE chain_id = $1 ORDER BY height DESC NULLS LAST LIMIT 1", chain_id).fetch_optional(&pool).await?.map(|block| block.height).unwrap_or_default() as u64;

        let range = if let Some(range) = self.range.clone() {
            if lowest > range.start {
                range
            } else {
                current..range.end
            }
        } else {
            current..u64::MAX
        };

        let execution = ExecutionStream::new(provider.clone());

        execution.stream(range).map_err(Report::from).try_for_each(|(block, mut receipts)| {
            info!("indexing block {}", block.number.unwrap());
            let json = serde_json::to_value(&block).unwrap();
            let tx = pool.begin();

            async move {
                let mut tx = tx.await?;
                let time = OffsetDateTime::from_unix_timestamp(block.time()?.timestamp())?;
                let block_id = sqlx::query!(
                    "
                    INSERT INTO v0.blocks (chain_id, hash, data, height, time, is_finalized) VALUES ($1, $2, $3, $4, $5, false) RETURNING id;
                    ",
                    chain_id,
                    block.hash.unwrap().to_string(),
                    json,
                    block.number.unwrap().as_u32() as i64,
                    time,
                ).fetch_one(tx.as_mut()).await?.id;
                receipts.sort_by(|a, b| a.transaction_index.cmp(&b.transaction_index));

                for transaction in receipts {
                    let json = serde_json::to_value(&transaction)?;
                    let transaction_id = sqlx::query!(
                        "
                        INSERT INTO v0.transactions (block_id, hash, data, index) VALUES ($1, $2, $3, $4) RETURNING id;
                        ",
                        block_id,
                        transaction.transaction_hash.to_string(),
                        json,
                        transaction.transaction_index.as_u32() as i64,
                    ).fetch_one(tx.as_mut()).await?.id;

                    for log in transaction.logs {
                        let json = serde_json::to_value(&log)?;
                        let _log_id = sqlx::query!(
                            "
                            INSERT INTO v0.events (block_id, transaction_id, index, stage, data) VALUES ($1, $2, $3, $4, $5) RETURNING id;
                            ",
                            block_id,
                            transaction_id,
                            transaction.transaction_index.as_u32() as i64,
                            STAGE_TRANSACTION,
                            json,
                        ).fetch_one(tx.as_mut()).await?;
                    }
                }
                Ok(())
            }
        }).await?;

        Ok(())
    }
}

pub struct ExecutionStream {
    provider: Provider<Http>,
}

impl ExecutionStream {
    fn new(provider: Provider<Http>) -> Self {
        Self { provider }
    }
}

impl ExecutionStream {
    /// Creates a stream which yields blocks and receipts up to head; and retries until a new block is produced. May return data from unfinalized/uncle blocks.
    pub fn stream(
        &self,
        numbers: Range<u64>,
    ) -> impl Stream<
        Item = Result<(Block<H256>, Vec<TransactionReceipt>), ethers::providers::ProviderError>,
    > + '_ {
        stream! {
            for current in numbers {
                loop {
                    let block = self.provider.get_block(current).await?;

                    if block.is_none() {
                       tokio::time::sleep(Duration::from_millis(1000)).await;
                    }

                    let receipts = self.provider.get_block_receipts(current).await.unwrap();
                    yield Ok((block.unwrap(), receipts));
                    break;
                }

            }
        }
    }
}
