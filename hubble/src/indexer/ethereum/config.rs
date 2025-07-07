use color_eyre::eyre::Report;
use sqlx::PgPool;
use url::Url;

use crate::indexer::{
    api::{BlockHeight, IndexerId},
    ethereum::{context::EthContext, fetcher_client::EthFetcherClient},
    event::types::UniversalChainId,
    nats::NatsConnection,
    ConsumerConfig, EnricherConfig, FinalizerConfig, FixerConfig, Indexer, PublisherConfig,
};

const DEFAULT_CHUNK_SIZE: usize = 200;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub indexer_id: IndexerId,
    pub universal_chain_id: UniversalChainId,
    pub start_height: BlockHeight,
    pub chunk_size: Option<usize>,
    pub rpc_urls: Vec<Url>,
    #[serde(default)]
    pub finalizer: FinalizerConfig,
    #[serde(default)]
    pub fixer: FixerConfig,
    #[serde(default)]
    pub publisher: PublisherConfig,
    #[serde(default)]
    pub consumer: ConsumerConfig,
    #[serde(default)]
    pub enricher: EnricherConfig,
    #[serde(default)]
    pub drain: bool,
}

impl Config {
    pub async fn build(
        self,
        pg_pool: PgPool,
        nats: Option<NatsConnection>,
    ) -> Result<Indexer<EthFetcherClient>, Report> {
        Ok(Indexer::new(
            pg_pool,
            nats,
            self.indexer_id,
            self.universal_chain_id,
            self.start_height,
            self.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE),
            self.finalizer,
            self.fixer,
            self.publisher,
            self.consumer,
            self.enricher,
            EthContext {
                rpc_urls: self.rpc_urls,
            },
            self.drain,
        ))
    }
}
