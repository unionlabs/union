use std::time::Duration;

use backon::Retryable;
use color_eyre::Report;
use ethers::{
    providers::{Http, Middleware, Provider},
    types::BlockId,
};
use futures::{StreamExt, TryStreamExt};
use sqlx::PgPool;
use tracing::{debug, info, info_span, Instrument};
use url::Url;

use crate::{
    eth::BlockInsert,
    postgres::{self, ChainId},
};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub label: String,
    pub url: Url,
    #[serde(default = "default_interval")]
    pub interval: Duration,
}

fn default_interval() -> Duration {
    Duration::from_secs(12)
}

pub struct Indexer {
    chain_id: ChainId,
    pool: PgPool,
    provider: Provider<Http>,
    interval: Duration,
}

impl Config {
    pub async fn indexer(self, pool: PgPool) -> Result<Indexer, Report> {
        let provider = Provider::<Http>::try_from(self.url.clone().as_str()).unwrap();

        info!("fetching chain-id from node");
        let chain_id = (|| {
            debug!(?provider, "retry fetching chain-id from node");
            provider.get_chainid()
        })
        .retry(&crate::expo_backoff())
        .await?
        .as_u64();

        let indexing_span = info_span!("indexer", chain_id = chain_id);
        async move {
            let chain_id = postgres::fetch_or_insert_chain_id(&pool, chain_id.to_string())
                .await?
                .get_inner_logged();

            Ok(Indexer {
                chain_id,
                pool,
                provider,
                interval: self.interval,
            })
        }
        .instrument(indexing_span)
        .await
    }
}

impl Indexer {
    pub async fn index(self) -> Result<(), Report> {
        let indexing_span = info_span!("indexer", chain_id = self.chain_id.canonical);

        async move {
            info!("starting fork-indexing");
            async fn fetch_and_compare_block(
                chain_id: ChainId,
                hash: String,
                height: i32,
                provider: Provider<Http>,
            ) -> Result<Option<BlockInsert>, Report> {
                let block = BlockInsert::from_provider_retried(
                    chain_id,
                    BlockId::Number(height.into()),
                    provider,
                )
                .await?;

                if block.hash != hash {
                    info!(
                        "found forked block {} in the db, replacing with {}",
                        &hash, &block.hash
                    );
                    return Ok(Some(block));
                }
                Ok(None)
            }

            loop {
                let logs = postgres::get_last_n_logs(&self.pool, self.chain_id, 32)?;
                let blocks: Vec<BlockInsert> = logs
                    .map_err(Report::from)
                    .try_filter_map(|(hash, height)| {
                        fetch_and_compare_block(self.chain_id, hash, height, self.provider.clone())
                    })
                    .map(futures::future::ready)
                    .buffered(32)
                    .try_collect()
                    .await?;
                let mut tx = self.pool.begin().await?;
                crate::postgres::update_batch_logs(&mut tx, blocks.into_iter().map(Into::into))
                    .await?;
                tx.commit().await?;
                tokio::time::sleep(self.interval).await;
            }
        }
        .instrument(indexing_span)
        .await
    }
}
