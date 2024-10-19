use color_eyre::eyre::Report;
use sqlx::PgPool;
use url::Url;

use crate::indexer::{
    api::{BlockHeight, IndexerId},
    eth::{context::EthContext, fetcher_client::EthFetcherClient},
    Indexer,
};

const DEFAULT_CHUNK_SIZE: usize = 200;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub indexer_id: IndexerId,
    pub start_height: BlockHeight,
    pub chunk_size: Option<usize>,
    pub urls: Vec<Url>,
}

impl Config {
    pub async fn build(self, pg_pool: PgPool) -> Result<Indexer<EthFetcherClient>, Report> {
        Ok(Indexer::new(
            pg_pool,
            self.indexer_id,
            self.start_height,
            self.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE),
            EthContext { urls: self.urls },
        ))
    }
}
