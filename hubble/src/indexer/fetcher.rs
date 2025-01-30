use std::time::Duration;

use color_eyre::eyre::Report;
use tokio::time::sleep;
use tracing::{debug, error, info, info_span, warn, Instrument};

use super::{
    api::{BlockHeight, FetcherClient},
    Indexer,
};
use crate::indexer::{
    api::{BlockHandle, BlockRange, BlockSelection, FetchMode, IndexerError},
    postgres::{get_current_height, update_block_status, update_current_height},
    HappyRangeFetcher,
};

enum RunToFinalizedLoopResult {
    RunAgain,
    Finished,
}

enum RunToTipLoopResult {
    RunAgain,
    TryAgainLater,
}

impl<T: FetcherClient> Indexer<T> {
    pub async fn run_fetcher(&self, fetcher_client: T) -> Result<(), IndexerError> {
        self.run_to_finalized(&fetcher_client)
            .instrument(info_span!("run-to-finalized"))
            .await?;
        self.run_to_tip(&fetcher_client)
            .instrument(info_span!("run-to-tip"))
            .await
    }

    async fn run_to_finalized(&self, fetcher_client: &T) -> Result<(), IndexerError> {
        loop {
            match self.run_to_finalized_loop(fetcher_client).await {
                Ok(RunToFinalizedLoopResult::RunAgain) => {
                    debug!("run again");
                }
                Ok(RunToFinalizedLoopResult::Finished) => {
                    debug!("finished");
                    return Ok(());
                }
                Err(error) => {
                    warn!("error in finalized loop: {error} => try again later (sleep 1s)");
                    sleep(Duration::from_secs(1)).await;
                }
            };
        }
    }

    async fn run_to_finalized_loop(
        &self,
        fetcher_client: &T,
    ) -> Result<RunToFinalizedLoopResult, IndexerError> {
        let chunk_size: u64 = self.chunk_size.try_into().unwrap();
        let delay_blocks: u64 = self.finalizer_config.delay_blocks.try_into().unwrap();

        debug!("fetching last finalized block");
        match fetcher_client
            .fetch_single(BlockSelection::LastFinalized, FetchMode::Lazy)
            .await
        {
            Ok(last_finalized) => {
                let next_height = self.next_height().await?;
                if next_height + chunk_size + delay_blocks > last_finalized.reference().height {
                    info!("near finalized height (current: {} + chunk: {} + delay: {} > finalized: {}) => start 'run to tip'", next_height, self.chunk_size, self.finalizer_config.delay_blocks, last_finalized.reference());
                    return Ok(RunToFinalizedLoopResult::Finished);
                }

                let catch_up_range: BlockRange =
                    (next_height..last_finalized.reference().height).into();
                info!("missing blocks: {catch_up_range}");

                for slice in catch_up_range.range_chunks(self.chunk_size) {
                    info!("{slice}: handling chunk");

                    last_finalized
                        .fetch_range_expect_all(slice.clone(), FetchMode::Eager, |block| {
                            self.store_block(block)
                        })
                        .instrument(info_span!("store"))
                        .await?;

                    info!("{slice}: handled chunk");
                }
                Ok(RunToFinalizedLoopResult::RunAgain)
            }
            Err(IndexerError::NoBlock(_)) => {
                info!("no finalized block => start 'run to tip'");
                Ok(RunToFinalizedLoopResult::Finished)
            }
            Err(error) => Err(error),
        }
    }

    async fn store_block(&self, block_handle: T::BlockHandle) -> Result<(), Report> {
        let reference = block_handle.reference();
        debug!("store: {}", reference);

        let mut tx = self.pg_pool.begin().await?;

        block_handle.insert(&mut tx).await?;

        update_current_height(
            &mut tx,
            self.indexer_id.clone(),
            reference.height,
            reference.timestamp,
        )
        .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn run_to_tip(&self, fetcher_client: &T) -> Result<(), IndexerError> {
        loop {
            match self.run_to_tip_loop(fetcher_client).await {
                Ok(RunToTipLoopResult::RunAgain) => {
                    debug!("run again");
                }
                Ok(RunToTipLoopResult::TryAgainLater) => {
                    debug!("try again later (sleep 1s)");
                    sleep(Duration::from_secs(1)).await;
                }
                Err(error) => {
                    warn!("error in run to tip loop: {error} => try again later (sleep 1s)");
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    async fn run_to_tip_loop(
        &self,
        fetcher_client: &T,
    ) -> Result<RunToTipLoopResult, IndexerError> {
        let next_height = self.next_height().await?;
        info!("{}: fetching", next_height);

        match fetcher_client
            .fetch_single(BlockSelection::Height(next_height), FetchMode::Eager)
            .await
        {
            Ok(block_handle) => {
                let reference = &block_handle.reference();
                debug!("{}: handling", reference);

                if next_height != reference.height {
                    error!(
                        "{}: unexpected height (actual {}, expecting {})",
                        reference.height, reference.height, next_height
                    );
                    return Err(IndexerError::UnexpectedHeightSingle(
                        next_height,
                        reference.height,
                    ));
                }

                let mut tx = self.pg_pool.begin().await?;
                block_handle
                    .insert(&mut tx)
                    .instrument(info_span!("insert"))
                    .await?;

                debug!("{}: update height", reference);
                update_current_height(
                    &mut tx,
                    self.indexer_id.clone(),
                    reference.height,
                    reference.timestamp,
                )
                .await?;

                debug!("{}: update status", reference);
                update_block_status(
                    &mut tx,
                    self.indexer_id.clone(),
                    reference.height,
                    reference.hash.clone(),
                    reference.timestamp,
                )
                .await?;

                tx.commit().await?;
                debug!("{}: handled", reference);
                Ok(RunToTipLoopResult::RunAgain)
            }
            Err(IndexerError::NoBlock(_)) => {
                debug!("{}: no block yet => sleep", next_height);
                Ok(RunToTipLoopResult::TryAgainLater)
            }
            Err(err) => {
                warn!("{}: error reading block => sleep : {:?}", next_height, err);
                Err(err)
            }
        }
    }

    async fn next_height(&self) -> Result<BlockHeight, Report> {
        let mut tx = self.pg_pool.begin().await?;
        let result = get_current_height(&mut tx, self.indexer_id.clone())
            .await?
            .map(|current_height| current_height + 1) // we store the last indexed height, so next is one higher
            .unwrap_or(self.start_height);
        tx.commit().await?;

        Ok(result)
    }
}
