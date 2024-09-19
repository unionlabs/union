use color_eyre::eyre::Report;
use sqlx::PgPool;
use url::Url;

use super::{context::EthContext, fetcher_client::EthFetcherClient};
use crate::indexer::{
    api::{BlockHeight, IndexerId},
    Indexer,
};

const DEFAULT_CHUNK_SIZE: usize = 200;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub indexer_id: IndexerId,
    pub start_height: BlockHeight,
    pub chunk_size: Option<usize>,
    pub urls: Vec<Url>,
    pub internal_chain_id: Option<i32>,
    pub new_chain_override: Option<bool>,
}

impl Config {
    pub async fn build(self, pg_pool: PgPool) -> Result<Indexer<EthFetcherClient>, Report> {
        // temporary safety-measure to fetch the start height from the contracts table,
        // because there will be no chain_state record after migrations

        let start_height = match self.internal_chain_id {
            Some(internal_chain_id) => {
                let record = sqlx::query!(
                    r#"
                        SELECT MAX(height) + 1 as height
                        FROM hubble.contract_status
                        WHERE internal_chain_id = $1
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
            EthContext { urls: self.urls },
        ))
    }
}
