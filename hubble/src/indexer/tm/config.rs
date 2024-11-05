use color_eyre::eyre::Report;
use regex::Regex;
use sqlx::PgPool;
use url::Url;

use crate::indexer::{
    api::{BlockHeight, IndexerId},
    tm::{context::TmContext, fetcher_client::TmFetcherClient},
    FinalizerConfig, Indexer,
};

const DEFAULT_CHUNK_SIZE: usize = 20;
const DEFAULT_TRANSACTIONS_MAX_PAGE_SIZE: u8 = 100;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub indexer_id: IndexerId,
    pub start_height: BlockHeight,
    pub chunk_size: Option<usize>,
    pub rpc_urls: Vec<Url>,
    pub grpc_urls: Vec<Url>,
    pub filter: Option<String>,
    pub internal_chain_id: Option<i32>,
    pub new_chain_override: Option<bool>,
    pub tx_search_max_page_size: Option<u8>,
    #[serde(default)]
    pub finalizer: FinalizerConfig,
}

impl Config {
    pub async fn build(self, pg_pool: PgPool) -> Result<Indexer<TmFetcherClient>, Report> {
        // temporary safety-measure to fetch the start height from the blocks table,
        // because there will be no chain_state record after migrations

        let start_height = match self.internal_chain_id {
            Some(internal_chain_id) => {
                let record = sqlx::query!(
                    r#"
                        SELECT MAX(height) + 1 as height
                        FROM v0.blocks
                        WHERE chain_id = $1
                    "#,
                    internal_chain_id,
                )
                .fetch_optional(&pg_pool)
                .await?;

                record
                    .expect("record when internal chain id is configured")
                    .height
                    .map(|h| h as BlockHeight)
                    .expect("expecting height when existing chain is configured")
            }
            None => {
                assert!(
                    self.new_chain_override
                        .expect("new chain override to be configured"),
                    "new chain override to be true"
                );

                self.start_height
            }
        };

        Ok(Indexer::new(
            pg_pool,
            self.indexer_id,
            start_height,
            self.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE),
            self.finalizer,
            TmContext {
                rpc_urls: self.rpc_urls,
                grpc_urls: self.grpc_urls,
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
