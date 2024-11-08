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

impl<T: FetcherClient> Indexer<T> {
    pub async fn run_finalizer(&self, fetcher_client: T) -> Result<(), IndexerError> {
        loop {
            if let Some(block_range_to_finalize) = self.block_range_to_finalize().await? {
                info!("{}: begin", block_range_to_finalize);

                match fetcher_client
                    .fetch_single(BlockSelection::LastFinalized, FetchMode::Lazy)
                    .await
                {
                    Ok(last_finalized) => {
                        let reference = last_finalized.reference();
                        trace!(
                            "{}: current finalized: {}",
                            block_range_to_finalize,
                            reference
                        );

                        // consider the block to be finalized if it's >= than the consensus height, considering the finalization delay blocks.
                        let consensus_height_with_safety_margin = reference
                            .height
                            .saturating_sub(self.finalizer_config.delay_blocks as u64);

                        let some_blocks_needs_to_be_finalized = block_range_to_finalize
                            .start_inclusive
                            <= consensus_height_with_safety_margin;
                        if some_blocks_needs_to_be_finalized {
                            // find the end of the range to finalize
                            let end_of_chunk = block_range_to_finalize.start_inclusive
                                + self.chunk_size as BlockHeight;
                            let end_until_finalized = consensus_height_with_safety_margin + 1;
                            let end_until_last_tracked_block =
                                block_range_to_finalize.end_exclusive;

                            let range_to_finalize_end_exclusive = min(
                                end_of_chunk,
                                min(end_until_finalized, end_until_last_tracked_block),
                            );

                            let range_to_finalize = (block_range_to_finalize.start_inclusive
                                ..range_to_finalize_end_exclusive)
                                .into();
                            debug!(
                                "{}: finalizing: {}",
                                block_range_to_finalize, range_to_finalize
                            );

                            self.finalize_blocks(
                                &last_finalized,
                                range_to_finalize,
                                consensus_height_with_safety_margin,
                            )
                            .instrument(info_span!("finalize"))
                            .await?;
                        } else {
                            trace!(
                                "{}: nothing to finalize (before finalized {})",
                                block_range_to_finalize,
                                reference
                            );
                        }

                        if let Some(height) = self
                            .next_block_to_monitor(consensus_height_with_safety_margin)
                            .await?
                        {
                            let range_to_monitor = (height
                                ..(min(
                                    height + self.chunk_size as BlockHeight,
                                    block_range_to_finalize.end_exclusive,
                                )))
                                .into();
                            debug!(
                                "{}: monitoring: {}",
                                block_range_to_finalize, range_to_monitor
                            );

                            self.finalize_blocks(
                                &last_finalized,
                                range_to_monitor,
                                consensus_height_with_safety_margin,
                            )
                            .instrument(info_span!("monitor"))
                            .await?;
                        } else {
                            trace!("{}: nothing to update", block_range_to_finalize);

                            if !some_blocks_needs_to_be_finalized {
                                info!("idle finalize run => retry later");
                                sleep(self.finalizer_config.retry_later_sleep).await;
                            }
                        }
                    }
                    Err(IndexerError::NoBlock(_)) => {
                        info!("no finalized height => retry later");
                        sleep(self.finalizer_config.retry_later_sleep).await;
                    }
                    Err(error) => {
                        warn!("error fetching finalized height ({}) => retry later", error);
                        sleep(self.finalizer_config.retry_later_sleep).await;
                    }
                }
            } else {
                info!("nothing to finalize => retry later");
                sleep(self.finalizer_config.retry_later_sleep).await;
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
