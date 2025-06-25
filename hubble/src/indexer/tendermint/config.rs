use color_eyre::eyre::Report;
use sqlx::PgPool;
use url::Url;

use crate::indexer::{
    api::{BlockHeight, IndexerId},
    event::types::UniversalChainId,
    nats::NatsConnection,
    tendermint::{context::TmContext, fetcher_client::TmFetcherClient},
    ConsumerConfig, FinalizerConfig, FixerConfig, Indexer, PublisherConfig,
};

const DEFAULT_CHUNK_SIZE: usize = 20;
const DEFAULT_TRANSACTIONS_MAX_PAGE_SIZE: u8 = 100;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub indexer_id: IndexerId,
    pub universal_chain_id: UniversalChainId,
    pub start_height: BlockHeight,
    pub chunk_size: Option<usize>,
    pub rpc_urls: Vec<Url>,
    pub tx_search_max_page_size: Option<u8>,
    #[serde(default)]
    pub finalizer: FinalizerConfig,
    #[serde(default)]
    pub fixer: FixerConfig,
    #[serde(default)]
    pub publisher: PublisherConfig,
    #[serde(default)]
    pub consumer: ConsumerConfig,
    #[serde(default)]
    pub testnet: bool,
    #[serde(default)]
    pub drain: bool,
}

impl Config {
    pub async fn build(
        self,
        pg_pool: PgPool,
        nats: Option<NatsConnection>,
    ) -> Result<Indexer<TmFetcherClient>, Report> {
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
            TmContext {
                rpc_urls: self.rpc_urls,
                tx_search_max_page_size: self
                    .tx_search_max_page_size
                    .unwrap_or(DEFAULT_TRANSACTIONS_MAX_PAGE_SIZE),
                testnet: self.testnet,
            },
            self.drain,
        ))
    }
}
