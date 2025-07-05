use color_eyre::eyre::Report;
use sqlx::PgPool;

use super::dummy::{DummyContext, DummyFetcherClient};
use crate::indexer::{
    api::IndexerId, event::types::UniversalChainId, nats::NatsConnection, ConsumerConfig,
    FinalizerConfig, FixerConfig, Indexer, PublisherConfig,
};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub indexer_id: IndexerId,
    pub universal_chain_id: UniversalChainId,
    pub start_height: u64,
    pub finalizer: FinalizerConfig,
    pub fixer: FixerConfig,
    pub publisher: PublisherConfig,
    pub consumer: ConsumerConfig,
    pub drain: bool,
}

impl Config {
    pub async fn build(
        self,
        pg_pool: PgPool,
        nats: Option<NatsConnection>,
    ) -> Result<Indexer<DummyFetcherClient>, Report> {
        Ok(Indexer::new(
            pg_pool,
            nats,
            self.indexer_id,
            self.universal_chain_id,
            self.start_height,
            5,
            self.finalizer,
            self.fixer,
            self.publisher,
            self.consumer,
            DummyContext { bla: 42 },
            self.drain,
        ))
    }
}
