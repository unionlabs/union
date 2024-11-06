use color_eyre::eyre::Report;
use sqlx::PgPool;
use url::Url;

use crate::indexer::{
    api::{BlockHeight, IndexerId},
    eth::{context::EthContext, fetcher_client::EthFetcherClient},
    FinalizerConfig, Indexer,
};

const DEFAULT_CHUNK_SIZE: usize = 200;
const DEFAULT_CLIENT_TRACKING: bool = true;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub indexer_id: IndexerId,
    pub start_height: BlockHeight,
    pub chunk_size: Option<usize>,
    pub urls: Vec<Url>,
    #[serde(default)]
    pub finalizer: FinalizerConfig,
    pub client_tracking: Option<bool>,
}

impl Config {
    pub async fn build(self, pg_pool: PgPool) -> Result<Indexer<EthFetcherClient>, Report> {
        Ok(Indexer::new(
            pg_pool,
            self.indexer_id,
            self.start_height,
            self.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE),
            self.finalizer,
            EthContext {
                urls: self.urls,
                client_tracking: self.client_tracking.unwrap_or(DEFAULT_CLIENT_TRACKING),
            },
        ))
    }
}
