use std::{cmp::min, time::Duration};

use color_eyre::eyre::Report;
use tokio::time::sleep;
use tracing::{debug, info, info_span, trace, warn, Instrument};

use super::{
    api::{BlockRange, FetcherClient, IndexerError},
    postgres::get_next_block_to_refresh,
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

                        if block_range_to_finalize.start_inclusive <= reference.height {
                            // find the end of the range to finalize
                            let end_of_chunk = block_range_to_finalize.start_inclusive
                                + self.chunk_size as BlockHeight;
                            let end_until_finalized = reference.height + 1;
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

                            self.finalize_blocks(&last_finalized, range_to_finalize)
                                .instrument(info_span!("finalize"))
                                .await?;
                        } else {
                            trace!(
                                "{}: nothing to finalize (before finalized {})",
                                block_range_to_finalize,
                                reference
                            );
                        }

                        if let Some(height) = self.next_block_to_refresh(reference.height).await? {
                            let range_to_refresh = (height
                                ..(min(
                                    height + self.chunk_size as BlockHeight,
                                    block_range_to_finalize.end_exclusive,
                                )))
                                .into();
                            info!("{}: finalizing", range_to_refresh);

                            self.finalize_blocks(&last_finalized, range_to_refresh)
                                .instrument(info_span!("monitor"))
                                .await?;
                        } else {
                            trace!("{}: nothing to update", block_range_to_finalize);
                        }
                    }
                    Err(IndexerError::NoBlock(_)) => {
                        info!("no finalized height => retry later");
                        sleep(Duration::from_millis(1000)).await;
                    }
                    Err(error) => {
                        warn!("error fetching finalized height ({}) => retry later", error);
                        sleep(Duration::from_millis(1000)).await;
                    }
                }

                if block_range_to_finalize.len() < self.chunk_size as u64 {
                    info!("not much to finalize => retry later");
                    sleep(Duration::from_millis(1000)).await;
                }
            } else {
                info!("nothing to finalize => retry later");
                sleep(Duration::from_millis(1000)).await;
            }
        }
    }

    async fn finalize_blocks(
        &self,
        last_finalized: &T::BlockHandle,
        block_range: BlockRange,
    ) -> Result<(), Report> {
        let last_finalized_reference = last_finalized.reference();

        last_finalized
            .fetch_range_expect_all(block_range.clone(), FetchMode::Lazy, |block| {
                self.finalize_block(block, last_finalized_reference.height)
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

        // consider the block to be finalized if it's >= than the consensus height, considering the finalization delay blocks.
        let is_finalized = last_finalized_height >= reference.height
            && last_finalized_height - reference.height
                >= self.finalizer_config.delay_blocks as u64;

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

    async fn next_block_to_refresh(
        &self,
        consensus_height: BlockHeight,
    ) -> Result<Option<BlockHeight>, Report> {
        let mut tx = self.pg_pool.begin().await?;
        let result =
            get_next_block_to_refresh(&mut tx, self.indexer_id.clone(), consensus_height).await?;
        tx.commit().await?;

        Ok(result)
    }
}
