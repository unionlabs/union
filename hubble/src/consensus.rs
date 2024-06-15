use color_eyre::Result;
use futures::{stream::FuturesOrdered, TryStreamExt};
use tracing::{debug, info};

use crate::postgres::ChainId;

pub struct Indexer<T: Querier + Send + Sync> {
    chain_id: ChainId,
    pool: sqlx::PgPool,
    querier: T,
}

pub trait Querier {
    async fn get_execution_height(&self, height: i64) -> Result<i64>;
}

impl<T: Querier + Send + Sync> Indexer<T> {
    pub fn new(chain_id: ChainId, pool: sqlx::PgPool, querier: T) -> Self {
        Self {
            chain_id,
            pool,
            querier,
        }
    }

    pub async fn index(&self) -> Result<()> {
        loop {
            info!("fixing next batch of unmapped_execution_heights");
            debug!("getting unmapped consensus heights");
            let consensus_heights =
                crate::postgres::get_batch_of_unmapped_execution_heights(&self.pool, self.chain_id)
                    .await?;

            if consensus_heights.is_empty() {
                debug!("no unmapped heights found, sleeping");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }

            let futures = FuturesOrdered::from_iter(
                consensus_heights
                    .iter()
                    .map(|height| self.querier.get_execution_height(*height)),
            );

            debug!("getting execution heights");
            let execution_heights: Vec<i64> = futures.try_collect().await?;

            debug!("inserting execution heights");
            crate::postgres::insert_mapped_execution_heights(
                &self.pool,
                execution_heights,
                consensus_heights,
                self.chain_id,
            )
            .await?;
        }
    }
}
