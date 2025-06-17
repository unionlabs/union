use color_eyre::eyre::Report;
use sqlx::PgPool;
use url::Url;

use crate::indexer::{
    api::{BlockHeight, IndexerId, UniversalChainId},
    ethereum::{context::EthContext, fetcher_client::EthFetcherClient},
    nats::NatsConnection,
    ConsumerConfig, FinalizerConfig, Indexer, PublisherConfig,
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
    pub publisher: PublisherConfig,
    #[serde(default)]
    pub consumer: ConsumerConfig,
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
            self.publisher,
            self.consumer,
            EthContext {
                rpc_urls: self.rpc_urls,
            },
        ))
    }
}
