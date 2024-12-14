use std::cmp::min;

use color_eyre::eyre::Report;
use tokio::time::{sleep, Duration};
use tracing::{debug, info, info_span, trace, warn, Instrument};

use super::{
    api::{BlockRange, FetcherClient, IndexerError},
    postgres::{get_next_block_to_monitor, delete_block_status, get_block_range_to_finalize, get_block_status_hash, update_block_status},
    Indexer,
};
use crate::indexer::{
    api::{BlockHandle, BlockHeight, BlockSelection, FetchMode},
    HappyRangeFetcher,
};

impl<T: FetcherClient> Indexer<T> {
    /// Main loop for finalizing blocks.
    pub async fn run_finalizer(&self, fetcher_client: T) -> Result<(), IndexerError> {
        let chunk_size: u64 = self.chunk_size.into();

        loop {
            match self.block_range_to_finalize().await? {
                Some(block_range_to_finalize) => {
                    info!("Starting finalization for range: {}", block_range_to_finalize);

                    match self.process_finalization(fetcher_client.clone(), block_range_to_finalize, chunk_size).await {
                        Ok(_) => debug!("Finalization completed for range."),
                        Err(err) => warn!("Error during finalization: {}. Retrying later.", err),
                    }
                }
                None => {
                    info!("No blocks to finalize. Retrying later.");
                    self.sleep_with_backoff().await;
                }
            }
        }
    }

    /// Handles the main finalization logic.
    async fn process_finalization(
        &self,
        fetcher_client: T,
        block_range_to_finalize: BlockRange,
        chunk_size: u64,
    ) -> Result<(), IndexerError> {
        match fetcher_client
            .fetch_single(BlockSelection::LastFinalized, FetchMode::Lazy)
            .await
        {
            Ok(last_finalized) => {
                let reference = last_finalized.reference();
                let consensus_height_with_margin = self.calculate_consensus_height(reference.height);

                if self.should_finalize_blocks(&block_range_to_finalize, consensus_height_with_margin) {
                    self.finalize_block_range(
                        &last_finalized,
                        block_range_to_finalize,
                        consensus_height_with_margin,
                        chunk_size,
                    )
                    .await?;
                } else {
                    trace!(
                        "No blocks to finalize for range: {} with consensus height: {}",
                        block_range_to_finalize,
                        consensus_height_with_margin
                    );
                }

                if let Some(height) = self.next_block_to_monitor(consensus_height_with_margin).await? {
                    self.monitor_block_range(
                        &last_finalized,
                        height,
                        chunk_size,
                        block_range_to_finalize.end_exclusive,
                    )
                    .await?;
                } else {
                    info!("No blocks to monitor for range: {}", block_range_to_finalize);
                    self.sleep_with_backoff().await;
                }
            }
            Err(IndexerError::NoBlock(_)) => {
                info!("No finalized height available. Retrying later.");
                self.sleep_with_backoff().await;
            }
            Err(err) => {
                warn!("Error fetching finalized height: {}. Retrying later.", err);
                self.sleep_with_backoff().await;
            }
        }

        Ok(())
    }

    /// Finalizes a range of blocks.
    async fn finalize_block_range(
        &self,
        last_finalized: &T::BlockHandle,
        block_range_to_finalize: BlockRange,
        consensus_height: BlockHeight,
        chunk_size: u64,
    ) -> Result<(), IndexerError> {
        let end = self.calculate_range_end(
            block_range_to_finalize.start_inclusive,
            chunk_size,
            consensus_height,
            block_range_to_finalize.end_exclusive,
        );

        let range_to_finalize: BlockRange = (block_range_to_finalize.start_inclusive..end).into();
        debug!("Finalizing block range: {}", range_to_finalize);

        self.finalize_blocks(last_finalized, range_to_finalize, consensus_height)
            .instrument(info_span!("finalize"))
            .await?;

        Ok(())
    }

    /// Monitors a range of blocks.
    async fn monitor_block_range(
        &self,
        last_finalized: &T::BlockHandle,
        start_height: BlockHeight,
        chunk_size: u64,
        end_height: BlockHeight,
    ) -> Result<(), IndexerError> {
        let end = min(start_height + chunk_size, end_height);
        let range_to_monitor: BlockRange = (start_height..end).into();

        debug!("Monitoring block range: {}", range_to_monitor);

        self.finalize_blocks(last_finalized, range_to_monitor, start_height)
            .instrument(info_span!("monitor"))
            .await?;

        Ok(())
    }

    /// Calculates the end of a block range.
    fn calculate_range_end(
        &self,
        start: BlockHeight,
        chunk_size: u64,
        consensus_height: BlockHeight,
        max_height: BlockHeight,
    ) -> BlockHeight {
        min(start + chunk_size, min(consensus_height + 1, max_height))
    }

    /// Checks whether blocks need finalization.
    fn should_finalize_blocks(&self, block_range: &BlockRange, consensus_height: BlockHeight) -> bool {
        block_range.start_inclusive <= consensus_height
    }

    /// Calculates the consensus height considering the safety margin.
    fn calculate_consensus_height(&self, finalized_height: BlockHeight) -> BlockHeight {
        finalized_height.saturating_sub(self.finalizer_config.delay_blocks.into())
    }

    /// Finalizes blocks in a given range.
    async fn finalize_blocks(
        &self,
        last_finalized: &T::BlockHandle,
        block_range: BlockRange,
        consensus_height: BlockHeight,
    ) -> Result<(), Report> {
        last_finalized
            .fetch_range_expect_all(block_range.clone(), FetchMode::Lazy, |block| {
                self.finalize_block(block, consensus_height)
            })
            .instrument(info_span!("finalize"))
            .await?;

        info!("Finalized block range: {}", block_range);
        Ok(())
    }

    /// Finalizes an individual block.
    async fn finalize_block(
        &self,
        block: T::BlockHandle,
        last_finalized_height: BlockHeight,
    ) -> Result<(), Report> {
        let reference = block.reference();
        debug!("Finalizing block: {}", reference);

        let mut tx = self.pg_pool.begin().await?;
        let is_finalized = last_finalized_height >= reference.height;

        if let Some(old_hash) = match is_finalized {
            true => delete_block_status(&mut tx, self.indexer_id.clone(), reference.height).await?,
            false => get_block_status_hash(&mut tx, self.indexer_id.clone(), reference.height).await?,
        } {
            self.update_block_if_needed(&mut tx, block, &reference, old_hash, is_finalized).await?;
        } else {
            warn!("Missing block status for block: {}", reference);
        }

        tx.commit().await?;
        debug!("Finalized block: {}", reference);

        Ok(())
    }

    /// Updates block status if necessary.
    async fn update_block_if_needed(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        block: T::BlockHandle,
        reference: &BlockReference,
        old_hash: String,
        is_finalized: bool,
    ) -> Result<(), Report> {
        if is_finalized && self.finalizer_config.reload {
            debug!("Reloading finalized block: {}", reference.height);
            block.update(tx).instrument(info_span!("reload")).await?;
        } else if old_hash != reference.hash {
            debug!(
                "Updating block: {} (hash changed from {} to {})",
                reference.height, old_hash, reference.hash
            );
            block.update(tx).instrument(info_span!("update")).await?;
        }

        if !is_finalized {
            debug!("Updating block status for: {}", reference);
            update_block_status(tx, self.indexer_id.clone(), reference.height, reference.hash.clone(), reference.timestamp).await?;
        }

        Ok(())
    }

    /// Retrieves the next block to monitor.
    async fn next_block_to_monitor(&self, consensus_height: BlockHeight) -> Result<Option<BlockHeight>, Report> {
        let mut tx = self.pg_pool.begin().await?;
        let result = get_next_block_to_monitor(&mut tx, self.indexer_id.clone(), consensus_height, self.finalizer_config.min_duration_between_monitor_checks).await?;
        tx.commit().await?;
        Ok(result)
    }

    /// Retrieves the block range to finalize.
    async fn block_range_to_finalize(&self) -> Result<Option<BlockRange>, Report> {
        let mut tx = self.pg_pool.begin().await?;
        let result = get_block_range_to_finalize(&mut tx, self.indexer_id.clone()).await?;
        tx.commit().await?;
        Ok(result)
    }

    /// Implements exponential backoff for retries.
    async fn sleep_with_backoff(&self) {
        sleep(self.finalizer_config.retry_later_sleep).await;
    }
}
