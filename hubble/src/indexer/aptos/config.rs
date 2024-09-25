use color_eyre::eyre::Report;
use regex::Regex;
use sqlx::PgPool;
use url::Url;

use super::{context::AptosContext, fetcher_client::AptosFetcherClient};
use crate::indexer::{
    api::{BlockHeight, IndexerId},
    Indexer,
};

const DEFAULT_CHUNK_SIZE: usize = 20;
const DEFAULT_TRANSACTIONS_MAX_PAGE_SIZE: u16 = 100;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub indexer_id: IndexerId,
    pub start_height: BlockHeight,
    pub chunk_size: Option<usize>,
    pub rpc_urls: Vec<Url>,
    pub filter: Option<String>,
    pub internal_chain_id: Option<i32>,
    pub tx_search_max_page_size: Option<u16>,
}

impl Config {
    pub async fn build(self, pg_pool: PgPool) -> Result<Indexer<AptosFetcherClient>, Report> {
        Ok(Indexer::new(
            pg_pool,
            self.indexer_id,
            self.start_height,
            self.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE),
            AptosContext {
                rpc_urls: self.rpc_urls,
                filter: self
                    .filter
                    .map(|regex| Regex::new(regex.as_str()).expect("valid filter regex")),
                tx_search_max_page_size: self
                    .tx_search_max_page_size
                    .unwrap_or(DEFAULT_TRANSACTIONS_MAX_PAGE_SIZE),
            },
        ))
    }
}
