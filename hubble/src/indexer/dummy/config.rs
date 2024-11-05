use color_eyre::eyre::Report;
use sqlx::PgPool;
use unionlabs::aptos::block_info::BlockHeight;

use super::dummy::{DummyContext, DummyFetcherClient};
use crate::indexer::{api::IndexerId, FinalizerConfig, Indexer};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub indexer_id: IndexerId,
    pub start_height: BlockHeight,
    pub finalizer: FinalizerConfig,
}

impl Config {
    pub async fn build(self, pg_pool: PgPool) -> Result<Indexer<DummyFetcherClient>, Report> {
        Ok(Indexer::new(
            pg_pool,
            self.indexer_id,
            self.start_height,
            5,
            self.finalizer,
            DummyContext { bla: 42 },
        ))
    }
}
