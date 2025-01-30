use std::cmp::min;

use color_eyre::eyre::Report;
use tokio::time::sleep;
use tracing::{debug, info, info_span, trace, warn, Instrument};

use super::{
    api::{BlockRange, FetcherClient, IndexerError},
    postgres::get_next_block_to_monitor,
    Indexer,
};
use crate::indexer::{
    api::{BlockHandle, BlockHeight, BlockSelection, FetchMode},
    postgres::{
        delete_block_status, get_block_range_to_finalize, get_block_status_hash,
        update_block_status,
    },
    HappyRangeFetcher,
};

enum FinalizerLoopResult {
    RunAgain,
    TryAgainLater,
}

impl<T: FetcherClient> Indexer<T> {
    pub async fn run_finalizer(&self, fetcher_client: T) -> Result<(), IndexerError> {
        loop {
            match self.run_finalizer_loop(&fetcher_client).await {
                Ok(FinalizerLoopResult::RunAgain) => {
                    debug!("run again");
                }
                Ok(FinalizerLoopResult::TryAgainLater) => {
                    debug!(
                        "try again later (sleep {}s)",
                        self.finalizer_config.retry_later_sleep.as_secs()
                    );
                    sleep(self.finalizer_config.retry_later_sleep).await;
                }
                Err(error) => {
                    warn!(
                        "error in finalizer loop: {error} => try again later (sleep {}s)",
                        self.finalizer_config.retry_later_sleep.as_secs()
                    );
                    sleep(self.finalizer_config.retry_later_sleep).await;
                }
            }
        }
    }

    async fn run_finalizer_loop(
        &self,
        fetcher_client: &T,
    ) -> Result<FinalizerLoopResult, IndexerError> {
        let chunk_size: u64 = self.chunk_size.try_into().unwrap();

        match self.block_range_to_finalize().await {
            Ok(Some(block_range_to_finalize)) => {
                info!("{block_range_to_finalize}: begin");

                match fetcher_client
                    .fetch_single(BlockSelection::LastFinalized, FetchMode::Lazy)
                    .await
                {
                    Ok(last_finalized) => {
                        let reference = last_finalized.reference();
                        trace!("{block_range_to_finalize}: current finalized: {reference}");

                        // consider the block to be finalized if it's >= than the consensus height, considering the finalization delay blocks.
                        let consensus_height_with_safety_margin = reference
                            .height
                            .saturating_sub(self.finalizer_config.delay_blocks.try_into().unwrap());

                        let some_blocks_needs_to_be_finalized = block_range_to_finalize
                            .start_inclusive
                            <= consensus_height_with_safety_margin;
                        if some_blocks_needs_to_be_finalized {
                            // find the end of the range to finalize
                            let end_of_chunk = block_range_to_finalize.start_inclusive + chunk_size;
                            let end_until_finalized = consensus_height_with_safety_margin + 1;
                            let end_until_last_tracked_block =
                                block_range_to_finalize.end_exclusive;

                            let range_to_finalize_end_exclusive = min(
                                end_of_chunk,
                                min(end_until_finalized, end_until_last_tracked_block),
                            );

                            let range_to_finalize: BlockRange = (block_range_to_finalize
                                .start_inclusive
                                ..range_to_finalize_end_exclusive)
                                .into();

                            debug!("{block_range_to_finalize}: finalizing: {range_to_finalize}");

                            self.finalize_blocks(
                                &last_finalized,
                                range_to_finalize.clone(),
                                consensus_height_with_safety_margin,
                            )
                            .instrument(info_span!("finalize"))
                            .await?;
                        } else {
                            trace!("{block_range_to_finalize}: nothing to finalize (before finalized {reference})");
                        }

                        match self
                            .next_block_to_monitor(consensus_height_with_safety_margin)
                            .await
                        {
                            Ok(Some(height)) => {
                                let range_to_monitor = (height
                                    ..(min(
                                        height + chunk_size,
                                        block_range_to_finalize.end_exclusive,
                                    )))
                                    .into();
                                debug!("{block_range_to_finalize}: monitoring: {range_to_monitor}");

                                self.finalize_blocks(
                                    &last_finalized,
                                    range_to_monitor,
                                    consensus_height_with_safety_margin,
                                )
                                .instrument(info_span!("monitor"))
                                .await?;

                                Ok(FinalizerLoopResult::RunAgain)
                            }
                            Ok(None) => {
                                trace!("{}: nothing to update", block_range_to_finalize);

                                match some_blocks_needs_to_be_finalized {
                                    true => Ok(FinalizerLoopResult::RunAgain),
                                    false => Ok(FinalizerLoopResult::TryAgainLater),
                                }
                            }
                            Err(error) => {
                                warn!("{block_range_to_finalize}: error fetching next block to monitor {error} => retry later");
                                Ok(FinalizerLoopResult::TryAgainLater)
                            }
                        }
                    }
                    Err(IndexerError::NoBlock(_)) => {
                        info!("no finalized height => retry later");
                        Ok(FinalizerLoopResult::TryAgainLater)
                    }
                    Err(error) => {
                        warn!("error fetching finalized height ({}) => retry later", error);
                        Err(error)
                    }
                }
            }
            Ok(None) => {
                info!("nothing to finalize => retry later");
                Ok(FinalizerLoopResult::TryAgainLater)
            }
            Err(error) => {
                warn!("error trying to fetch block range to finalize ({error}) => retry later");
                Err(IndexerError::ProviderError(error))
            }
        }
    }

    async fn finalize_blocks(
        &self,
        last_finalized: &T::BlockHandle,
        block_range: BlockRange,
        height_considered_to_be_finalized: BlockHeight,
    ) -> Result<(), Report> {
        last_finalized
            .fetch_range_expect_all(block_range.clone(), FetchMode::Lazy, |block| {
                self.finalize_block(block, height_considered_to_be_finalized)
            })
            .instrument(info_span!("finalize"))
            .await?;

        info!("{}: done", block_range);

        Ok(())
    }

    async fn finalize_block(
        &self,
        block: T::BlockHandle,
        last_finalized_height: BlockHeight,
    ) -> Result<(), Report> {
        let reference = block.reference();
        debug!("{}: finalizing", reference);

        let mut tx = self.pg_pool.begin().await?;

        let is_finalized = last_finalized_height >= reference.height;

        if let Some(old_hash) = match is_finalized {
            true => delete_block_status(&mut tx, self.indexer_id.clone(), reference.height).await?,
            false => {
                get_block_status_hash(&mut tx, self.indexer_id.clone(), reference.height).await?
            }
        } {
            if is_finalized && self.finalizer_config.reload {
                debug!("{}: finalized (reloading)", reference.height,);
                block
                    .update(&mut tx)
                    .instrument(info_span!("reload"))
                    .await?;
            } else if old_hash != reference.hash {
                debug!(
                    "{}: changed ({} > {} => updating)",
                    reference.height, old_hash, reference.hash,
                );
                block
                    .update(&mut tx)
                    .instrument(info_span!("update"))
                    .await?;
            }

            if !is_finalized {
                debug!("{}: update status", reference);
                update_block_status(
                    &mut tx,
                    self.indexer_id.clone(),
                    reference.height,
                    reference.hash.clone(),
                    reference.timestamp,
                )
                .await?;
            }
        } else {
            warn!(
                "{}: expecting block-status, but there was none at height",
                reference
            );
        }

        tx.commit().await?;

        debug!("{}: finalized", reference);

        Ok(())
    }

    async fn block_range_to_finalize(&self) -> Result<Option<BlockRange>, Report> {
        let mut tx = self.pg_pool.begin().await?;
        let result = get_block_range_to_finalize(&mut tx, self.indexer_id.clone()).await?;
        tx.commit().await?;

        Ok(result)
    }

    async fn next_block_to_monitor(
        &self,
        consensus_height: BlockHeight,
    ) -> Result<Option<BlockHeight>, Report> {
        let mut tx = self.pg_pool.begin().await?;
        let result = get_next_block_to_monitor(
            &mut tx,
            self.indexer_id.clone(),
            consensus_height,
            self.finalizer_config.min_duration_between_monitor_checks,
        )
        .await?;
        tx.commit().await?;

        Ok(result)
    }
}
