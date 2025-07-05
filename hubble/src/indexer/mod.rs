pub mod api;
// pub mod aptos;
mod consumer;
pub mod dummy;
mod enrich;
pub mod ethereum;
pub mod event;
mod fetcher;
mod finalizer;
mod fixer;
mod handler;
pub mod nats;
mod postgres;
mod publisher;
mod record;
pub mod tendermint;

use std::{future::Future, time::Duration};

use api::{
    BlockHandle, BlockHeight, BlockRange, FetchMode, FetcherClient, IndexerError, IndexerId,
};
use color_eyre::eyre::Report;
use futures::{pin_mut, StreamExt};
use serde::{Deserialize, Deserializer};
use tokio::{task::JoinSet, time::sleep};
use tracing::{error, info, info_span, Instrument};

use crate::indexer::{event::types::UniversalChainId, nats::NatsConnection};

enum EndOfRunResult {
    Exit,
    Restart,
}

#[derive(Clone)]
pub struct Indexer<T: FetcherClient> {
    pub pg_pool: sqlx::PgPool,
    pub nats: Option<NatsConnection>,
    pub indexer_id: IndexerId,
    pub universal_chain_id: UniversalChainId,
    pub start_height: BlockHeight,
    pub chunk_size: usize,
    pub finalizer_config: FinalizerConfig,
    pub fixer_config: FixerConfig,
    pub publisher_config: PublisherConfig,
    pub consumer_config: ConsumerConfig,
    pub context: T::Context,
    pub drain: bool,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct PublisherConfig {
    // sleep time (in milliseconds) when there is nothing to publish.
    // default: 100 millis
    #[serde(
        rename = "retry_later_sleep_millis",
        default = "PublisherConfig::default_retry_later_sleep",
        deserialize_with = "PublisherConfig::deserialize_millis"
    )]
    pub retry_later_sleep: Duration,
    // sleep time (in milliseconds) when there is error publishing.
    // default: 5 seconds
    #[serde(
        rename = "retry_error_sleep_millis",
        default = "PublisherConfig::default_retry_error_sleep",
        deserialize_with = "PublisherConfig::deserialize_millis"
    )]
    pub retry_error_sleep: Duration,
    // number of messages read from the database that are pushed in one database transaction.
    // default: 1
    #[serde(default = "PublisherConfig::default_batch_size")]
    pub batch_size: usize,
}

impl PublisherConfig {
    pub fn default_retry_later_sleep() -> Duration {
        Duration::from_millis(100)
    }

    pub fn default_retry_error_sleep() -> Duration {
        Duration::from_secs(5)
    }

    pub fn default_batch_size() -> usize {
        1
    }

    fn deserialize_millis<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

impl Default for PublisherConfig {
    fn default() -> Self {
        PublisherConfig {
            retry_later_sleep: PublisherConfig::default_retry_later_sleep(),
            retry_error_sleep: PublisherConfig::default_retry_error_sleep(),
            batch_size: PublisherConfig::default_batch_size(),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct ConsumerConfig {
    // sleep time (in milliseconds) when there is error publishing.
    // default: 5 seconds
    #[serde(
        rename = "retry_error_sleep_millis",
        default = "ConsumerConfig::default_retry_error_sleep",
        deserialize_with = "ConsumerConfig::deserialize_millis"
    )]
    pub retry_error_sleep: Duration,
    // number of messages read from the database that are pushed in one database transaction.
    // default: 1
    #[serde(default = "ConsumerConfig::default_batch_size")]
    pub batch_size: usize,
}

impl ConsumerConfig {
    pub fn default_retry_error_sleep() -> Duration {
        Duration::from_secs(5)
    }

    pub fn default_batch_size() -> usize {
        1
    }

    fn deserialize_millis<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

impl Default for ConsumerConfig {
    fn default() -> Self {
        ConsumerConfig {
            retry_error_sleep: ConsumerConfig::default_retry_error_sleep(),
            batch_size: ConsumerConfig::default_batch_size(),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct FinalizerConfig {
    // how many blocks to wait until a block is considered finalized (ie. there should be no reorgs).
    // this safety margin compensates for height differences between rpcs
    // default: 5
    #[serde(default = "FinalizerConfig::default_delay_blocks")]
    pub delay_blocks: usize,
    // reload all block data after a block is considered finalized
    // compensates for rpcs returning inconsistent results for non-finalized blocks.
    // default: true
    #[serde(default = "FinalizerConfig::default_reload")]
    pub reload: bool,
    // minimum time (in seconds) between checking hash changes of non finalized blocks.
    // default: 1 minute
    #[serde(
        rename = "min_seconds_between_monitor_checks",
        default = "FinalizerConfig::default_min_duration_between_monitor_checks",
        deserialize_with = "FinalizerConfig::deserialize_seconds"
    )]
    pub min_duration_between_monitor_checks: Duration,
    // sleep time (in seconds) when there is nothing to finalize.
    // default: 5 seconds
    #[serde(
        rename = "retry_later_sleep_seconds",
        default = "FinalizerConfig::default_retry_later_sleep",
        deserialize_with = "FinalizerConfig::deserialize_seconds"
    )]
    pub retry_later_sleep: Duration,
    // sleep time (in seconds) when there is an error.
    // default: 5 seconds
    #[serde(
        rename = "retry_error_sleep_seconds",
        default = "FinalizerConfig::default_retry_error_sleep",
        deserialize_with = "FinalizerConfig::deserialize_seconds"
    )]
    pub retry_error_sleep: Duration,
}

impl FinalizerConfig {
    pub fn default_delay_blocks() -> usize {
        5
    }

    pub fn default_reload() -> bool {
        true
    }

    pub fn default_min_duration_between_monitor_checks() -> Duration {
        Duration::from_secs(60)
    }

    pub fn default_retry_later_sleep() -> Duration {
        Duration::from_secs(5)
    }

    pub fn default_retry_error_sleep() -> Duration {
        Duration::from_secs(5)
    }

    fn deserialize_seconds<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(seconds))
    }
}

impl Default for FinalizerConfig {
    fn default() -> Self {
        FinalizerConfig {
            delay_blocks: FinalizerConfig::default_delay_blocks(),
            reload: FinalizerConfig::default_reload(),
            min_duration_between_monitor_checks:
                FinalizerConfig::default_min_duration_between_monitor_checks(),
            retry_later_sleep: FinalizerConfig::default_retry_later_sleep(),
            retry_error_sleep: FinalizerConfig::default_retry_error_sleep(),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct FixerConfig {
    // sleep time (in seconds) when there is nothing to fix.
    // default: 5 seconds
    #[serde(
        rename = "retry_later_sleep_seconds",
        default = "FixerConfig::default_retry_later_sleep",
        deserialize_with = "FixerConfig::deserialize_seconds"
    )]
    pub retry_later_sleep: Duration,

    // sleep time (in seconds) when there is an error.
    // default: 5 seconds
    #[serde(
        rename = "retry_error_sleep_seconds",
        default = "FixerConfig::default_retry_error_sleep",
        deserialize_with = "FixerConfig::deserialize_seconds"
    )]
    pub retry_error_sleep: Duration,

    // maximum number of blocks to send in one message. An empty block is sent if no
    // events are found after this amount of blocks.
    // default: 1000
    #[serde(
        rename = "max_blocks_in_message",
        default = "FixerConfig::default_max_blocks_in_message"
    )]
    pub max_blocks_in_message: u64,
}

impl FixerConfig {
    pub fn default_retry_later_sleep() -> Duration {
        Duration::from_secs(5)
    }

    pub fn default_retry_error_sleep() -> Duration {
        Duration::from_secs(5)
    }

    pub fn default_max_blocks_in_message() -> u64 {
        1000
    }

    fn deserialize_seconds<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(seconds))
    }
}

impl Default for FixerConfig {
    fn default() -> Self {
        FixerConfig {
            retry_later_sleep: FixerConfig::default_retry_later_sleep(),
            retry_error_sleep: FixerConfig::default_retry_error_sleep(),
            max_blocks_in_message: FixerConfig::default_max_blocks_in_message(),
        }
    }
}

impl<T> Indexer<T>
where
    T: FetcherClient,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pg_pool: sqlx::PgPool,
        nats: Option<NatsConnection>,
        indexer_id: IndexerId,
        universal_chain_id: UniversalChainId,
        start_height: BlockHeight,
        chunk_size: usize,
        finalizer_config: FinalizerConfig,
        fixer_config: FixerConfig,
        publisher_config: PublisherConfig,
        consumer_config: ConsumerConfig,
        context: T::Context,
        drain: bool,
    ) -> Self {
        Indexer {
            pg_pool,
            nats,
            indexer_id,
            universal_chain_id,
            start_height,
            chunk_size,
            finalizer_config,
            fixer_config,
            publisher_config,
            consumer_config,
            context,
            drain,
        }
    }

    pub async fn index(&self) -> Result<(), Report> {
        loop {
            let mut join_set = JoinSet::new();

            match self
                .create_fetcher_client(&mut join_set, self.context.clone())
                .instrument(info_span!("creator"))
                .await
            {
                Some(fetcher_client) => {
                    let self_clone = self.clone();
                    let fetcher_client_clone = fetcher_client.clone();
                    join_set.spawn(
                        async move { self_clone.run_fetcher(fetcher_client_clone).await }
                            .instrument(info_span!("fetcher")),
                    );

                    let self_clone = self.clone();
                    let fetcher_client_clone = fetcher_client.clone();
                    join_set.spawn(
                        async move { self_clone.run_finalizer(fetcher_client_clone).await }
                            .instrument(info_span!("finalizer")),
                    );

                    let self_clone = self.clone();
                    let fetcher_client_clone = fetcher_client.clone();
                    join_set.spawn(
                        async move { self_clone.run_fixer(fetcher_client_clone).await }
                            .instrument(info_span!("fixer")),
                    );

                    let self_clone = self.clone();
                    join_set.spawn(
                        async move { self_clone.run_publisher().await }
                            .instrument(info_span!("publisher")),
                    );

                    let self_clone = self.clone();
                    join_set.spawn(
                        async move { self_clone.run_consumer().await }
                            .instrument(info_span!("consumer")),
                    );

                    if let EndOfRunResult::Exit = self
                        .handle_end_of_run(&mut join_set, fetcher_client)
                        .instrument(info_span!("terminator"))
                        .await?
                    {
                        return Ok(());
                    }
                }
                None => {
                    // can't create client => try again later
                    sleep(Duration::from_millis(1000)).await;
                }
            }
        }
    }

    async fn create_fetcher_client(
        &self,
        join_set: &mut JoinSet<Result<(), IndexerError>>,
        context: T::Context,
    ) -> Option<T> {
        info!("creating client (context: {})", self.context);
        match T::create(self.pg_pool.clone(), join_set, context).await {
            Ok(client) => {
                info!("created client: {}", client);
                Some(client)
            }
            Err(error) => {
                error!(
                    "error creating client: {:?} (context: {})",
                    error, self.context
                );
                None
            }
        }
    }

    async fn handle_end_of_run(
        &self,
        join_set: &mut JoinSet<Result<(), IndexerError>>,
        fetcher_client: T,
    ) -> Result<EndOfRunResult, Report> {
        while let Some(res) = join_set.join_next().await {
            match res {
                Ok(Err(err)) => {
                    error!(
                        "{}: error: {:?}. re-initialize (client: {}, context: {})",
                        self.indexer_id, err, fetcher_client, self.context
                    );
                    join_set.abort_all();
                    sleep(Duration::from_secs(1)).await;
                    info!("{}: restarting", self.indexer_id);
                    return Ok(EndOfRunResult::Restart);
                }
                Err(err) => return Err(err.into()),
                Ok(Ok(_)) => {
                    info!("{}: indexer exited gracefully", self.indexer_id);
                }
            }
        }
        Ok(EndOfRunResult::Exit)
    }
}

// Utility that verifies that received blocks have the expected height.
// 'f' can assume that the height is verified, so it only needs to
// implement the 'happy path'.
pub trait HappyRangeFetcher<T: BlockHandle> {
    async fn fetch_range_expect_all<F, Fut>(
        &self,
        range: BlockRange,
        mode: FetchMode,
        f: F,
    ) -> Result<(), IndexerError>
    where
        F: Fn(T) -> Fut,
        Fut: Future<Output = Result<(), Report>>;
}

impl<T: BlockHandle> HappyRangeFetcher<T> for T {
    async fn fetch_range_expect_all<F, Fut>(
        &self,
        range: BlockRange,
        mode: FetchMode,
        f: F,
    ) -> Result<(), IndexerError>
    where
        F: Fn(T) -> Fut,
        Fut: Future<Output = Result<(), Report>>,
    {
        let stream = self.fetch_range(range.clone(), mode)?;
        pin_mut!(stream);

        let expected_block_heights = range.clone().into_iter();

        for expected_block_height in expected_block_heights {
            match stream.next().await {
                Some(Ok(block)) => {
                    let actual_block_height = block.reference().height;

                    match expected_block_height == actual_block_height {
                        true => f(block).await?,
                        false => {
                            error!(
                                "{}: unexpected height (actual {}, expecting {})",
                                actual_block_height, actual_block_height, expected_block_height
                            );
                            return Err(IndexerError::UnexpectedHeightRange(
                                expected_block_height,
                                range,
                                actual_block_height,
                            ));
                        }
                    }
                }
                Some(Err(error)) => {
                    error!(
                        "{}: error reading block: {:?})",
                        expected_block_height, error
                    );
                    return Err(IndexerError::ErrorReadingBlock(
                        expected_block_height,
                        range,
                        Box::new(error.into()),
                    ));
                }
                None => {
                    error!(
                        "{}: missing block: {})",
                        expected_block_height, expected_block_height
                    );
                    return Err(IndexerError::MissingBlock(expected_block_height, range));
                }
            }
        }

        if let Some(result) = stream.next().await {
            error!("{}: too many blocks", range);
            return Err(match result {
                Ok(block) => IndexerError::TooManyBlocks(range, block.reference()),
                Err(error) => {
                    IndexerError::TooManyBlocksError(range, Box::new(Report::from(error)))
                }
            });
        }

        Ok(())
    }
}
