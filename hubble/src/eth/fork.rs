use std::time::Duration;

use alloy::{
    eips::BlockId,
    providers::{ProviderBuilder, RootProvider},
    transports::http::{Client, Http},
};
use backon::Retryable;
use color_eyre::Report;
use futures::{stream, stream::FuturesOrdered, StreamExt, TryStreamExt};
use sqlx::PgPool;
use tracing::{debug, info, info_span, Instrument};
use url::Url;

use crate::{
    eth::{BlockInsert, PgLog},
    postgres::{self, ChainId},
    race_client::RaceClient,
};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub label: String,
    pub urls: Vec<Url>,
    #[serde(default = "default_interval")]
    pub interval: Duration,
    #[serde(default)]
    pub start_height: Option<i32>,
    #[serde(default)]
    pub chunk_size: Option<usize>,
}

fn default_interval() -> Duration {
    Duration::from_secs(12)
}

pub struct Indexer {
    chain_id: ChainId,
    pool: PgPool,
    provider: RaceClient<RootProvider<Http<Client>>>,
    interval: Duration,
    start_height: Option<i32>,
    chunk_size: usize,
}

impl Config {
    pub async fn indexer(self, pool: PgPool) -> Result<Indexer, Report> {
        let provider = RaceClient::new(
            self.urls
                .into_iter()
                .map(|url| ProviderBuilder::new().on_http(url))
                .collect(),
        );

        info!("fetching chain-id from node");
        let chain_id = (|| {
            debug!(?provider, "retry fetching chain-id from node");
            provider.get_chain_id()
        })
        .retry(&crate::expo_backoff())
        .await?;

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
                start_height: self.start_height,
                chunk_size: self.chunk_size.unwrap_or(32),
            })
        }
        .instrument(indexing_span)
        .await
    }
}

impl Indexer {
    pub async fn index(self) -> Result<(), Report> {
        use crate::postgres::InsertMode;

        let indexing_span = info_span!("indexer", chain_id = self.chain_id.canonical);

        async move {
            info!("starting fork-indexing");
            async fn fetch_and_compare_block(
                chain_id: ChainId,
                hash: String,
                height: i32,
                provider: RaceClient<RootProvider<Http<Client>>>,
            ) -> Result<Option<BlockInsert>, Report> {
                let height: u64 = height as u64;
                let block = BlockInsert::from_provider_retried_filtered(
                    chain_id,
                    BlockId::Number(height.into()),
                    provider.clone(),
                    None,
                )
                .await?
                .expect_left("with filter None a block should always be returned");

                if block.hash != hash {
                    info!(
                        "found forked block {} in the db, replacing with {}",
                        &hash, &block.hash
                    );
                    return Ok(Some(block));
                }
                Ok(None)
            }

            let chunk_size = self
                .chunk_size
                .try_into()
                .expect("chunk_size should not exceed i64 in size");

            // Re-indexes starting at from until reaching tip.
            if let Some(mut height) = self.start_height {
                loop {
                    let logs: Vec<(String, i32)> = crate::postgres::get_n_logs_from(
                        &self.pool,
                        self.chain_id,
                        height,
                        chunk_size,
                    )?
                    .try_collect()
                    .await?;

                    // We're caught up to the tip.
                    if logs.is_empty() {
                        break;
                    }

                    height = logs.last().unwrap().1;

                    let blocks: Vec<BlockInsert> = futures::stream::iter(logs.into_iter())
                        .map(Ok::<_, Report>)
                        .try_filter_map(|(hash, height)| {
                            fetch_and_compare_block(
                                self.chain_id,
                                hash,
                                height,
                                self.provider.clone(),
                            )
                        })
                        .map(futures::future::ready)
                        .buffered(self.chunk_size)
                        .try_collect()
                        .await?;

                    if !blocks.is_empty() {
                        crate::postgres::update_batch_logs(
                            &self.pool,
                            blocks.into_iter().map(Into::into),
                        )
                        .await?;
                    }
                }
            }

            // Re-indexes the tip.
            loop {
                let logs: Vec<(String, i32)> =
                    postgres::get_last_n_logs(&self.pool, self.chain_id, chunk_size)?
                        .try_collect()
                        .await?;
                let start = logs[0].1;
                let end = logs.last().unwrap().1;

                let blocks: Vec<BlockInsert> = stream::iter(logs.into_iter().map(Ok::<_, Report>))
                    .try_filter_map(|(hash, height)| {
                        fetch_and_compare_block(self.chain_id, hash, height, self.provider.clone())
                    })
                    .map(futures::future::ready)
                    .buffered(chunk_size as usize)
                    .try_collect()
                    .await?;

                // If we encounter any forked block in the batch, we recalculate the entire batch.
                if !blocks.is_empty() {
                    let logs: Vec<PgLog<_>> =
                        FuturesOrdered::from_iter((start..=end).map(|height| {
                            let provider_clone = self.provider.clone();
                            async move {
                                BlockInsert::from_provider_retried_filtered(
                                    self.chain_id,
                                    height as u64,
                                    provider_clone,
                                    None,
                                )
                                .await
                                .map(|b| b.unwrap_left().into())
                            }
                        }))
                        .try_collect()
                        .await?;

                    let mut tx = self.pool.begin().await?;
                    crate::postgres::insert_batch_logs(
                        &mut tx,
                        logs.into_iter(),
                        InsertMode::Upsert,
                    )
                    .await?;
                    tx.commit().await?;
                }

                tokio::time::sleep(self.interval).await;
            }
        }
        .instrument(indexing_span)
        .await
    }
}
