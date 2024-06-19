use color_eyre::Result;
use tracing::{debug, info};

use crate::postgres::ChainId;

pub struct Indexer<T: Querier + Send + Sync> {
    chain_id: ChainId,
    pool: sqlx::PgPool,
    querier: T,
}

pub trait Querier {
    async fn get_execution_height(&self, height: i64) -> Result<(i64, i64)>;
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
        let mut consensus_height =
            crate::postgres::get_max_consensus_height(&self.pool, self.chain_id).await? + 1;
        loop {
            info!("mapping consensus height {consensus_height}");

            debug!("getting unmapped consensus heights");
            let (slot, height) = self.querier.get_execution_height(consensus_height).await?;

            debug!("got execution height {height} for consensus height {consensus_height}");

            debug!("inserting execution height");
            crate::postgres::insert_mapped_execution_heights(
                &self.pool,
                vec![height],
                vec![slot],
                self.chain_id,
            )
            .await?;
            consensus_height = slot + 1;
        }
    }
}
