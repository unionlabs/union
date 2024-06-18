use color_eyre::Result;
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
        let begin = crate::postgres::get_max_consensus_height(&self.pool, self.chain_id).await?;

        for consensus_height in begin + 1..i64::MAX {
            info!("mapping consensus height {consensus_height}");

            debug!("getting unmapped consensus heights");
            let height = self.querier.get_execution_height(consensus_height).await?;

            debug!("got execution height {height} for consensus height {consensus_height}");

            debug!("inserting execution height");
            crate::postgres::insert_mapped_execution_heights(
                &self.pool,
                vec![height],
                vec![consensus_height],
                self.chain_id,
            )
            .await?;
        }
        unreachable!("indexing consensus heights should never end")
    }
}
