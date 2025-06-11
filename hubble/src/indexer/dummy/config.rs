use color_eyre::eyre::Report;
use sqlx::PgPool;

use super::dummy::{DummyContext, DummyFetcherClient};
use crate::indexer::{api::IndexerId, FinalizerConfig, Indexer, PublisherConfig};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub indexer_id: IndexerId,
    pub start_height: u64,
    pub finalizer: FinalizerConfig,
    pub publisher: PublisherConfig,
}

impl Config {
    pub async fn build(
        self,
        pg_pool: PgPool,
        nats: Option<async_nats::jetstream::context::Context>,
    ) -> Result<Indexer<DummyFetcherClient>, Report> {
        Ok(Indexer::new(
            pg_pool,
            nats,
            self.indexer_id,
            self.start_height,
            5,
            self.finalizer,
            self.publisher,
            DummyContext { bla: 42 },
        ))
    }
}
