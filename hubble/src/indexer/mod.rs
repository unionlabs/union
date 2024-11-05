pub mod api;
pub mod aptos;
pub mod dummy;
pub mod eth;
mod fetcher;
mod finalizer;
mod fixer;
mod postgres;
pub mod tm;

use std::{future::Future, time::Duration};

use api::{
    BlockHandle, BlockHeight, BlockRange, FetchMode, FetcherClient, IndexerError, IndexerId,
};
use color_eyre::eyre::Report;
use futures::{pin_mut, StreamExt};
use tokio::{task::JoinSet, time::sleep};
use tracing::{error, info, info_span, Instrument};

enum EndOfRunResult {
    Exit,
    Restart,
}

#[derive(Clone)]
pub struct Indexer<T: FetcherClient> {
    pub pg_pool: sqlx::PgPool,
    pub indexer_id: IndexerId,
    pub start_height: BlockHeight,
    pub chunk_size: usize,
    pub finalizer_config: FinalizerConfig,
    pub context: T::Context,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct FinalizerConfig {
    // how many blocks to wait until a block is considered finalized (ie. there should be no reorgs).
    // compensates for height differences between rpcs
    // default: 5
    #[serde(default = "FinalizerConfig::default_delay_blocks")]
    pub delay_blocks: usize,
    // reload all block data after a block is considered finalized
    // compensates for rpcs returning inconsistent results for non-finalized blocks.
    // default: true
    #[serde(default = "FinalizerConfig::default_reload")]
    pub reload: bool,
}

impl FinalizerConfig {
    pub fn default_delay_blocks() -> usize {
        5
    }

    pub fn default_reload() -> bool {
        true
    }
}

impl Default for FinalizerConfig {
    fn default() -> Self {
        FinalizerConfig {
            delay_blocks: FinalizerConfig::default_delay_blocks(),
            reload: FinalizerConfig::default_reload(),
        }
    }
}

impl<T> Indexer<T>
where
    T: FetcherClient,
{
    pub fn new(
        pg_pool: sqlx::PgPool,
        indexer_id: IndexerId,
        start_height: BlockHeight,
        chunk_size: usize,
        finalizer_config: FinalizerConfig,
        context: T::Context,
    ) -> Self {
        Indexer {
            pg_pool,
            indexer_id,
            start_height,
            chunk_size,
            finalizer_config,
            context,
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
                        error.into(),
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
                Err(error) => IndexerError::TooManyBlocksError(range, Report::from(error)),
            });
        }

        Ok(())
    }
}
