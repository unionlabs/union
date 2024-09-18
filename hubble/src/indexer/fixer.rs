use std::{cmp::min, time::Duration};

use color_eyre::eyre::Report;
use tokio::time::sleep;
use tracing::{debug, info, info_span, trace, warn, Instrument};

use super::{
    api::{BlockRange, FetcherClient, IndexerError},
    postgres::{get_block_range_to_fix, update_block_range_to_fix},
    Indexer,
};
use crate::indexer::{
    api::{BlockHandle, BlockHeight, BlockSelection, FetchMode},
    HappyRangeFetcher,
};

impl<T: FetcherClient> Indexer<T> {
    pub async fn run_fixer(&self, fetcher_client: T) -> Result<(), IndexerError> {
        loop {
            if let Some(block_range_to_fix) = self.block_range_to_fix().await? {
                info!("{}: begin", block_range_to_fix);

                match fetcher_client
                    .fetch_single(BlockSelection::LastFinalized, FetchMode::Lazy)
                    .await
                {
                    Ok(last_finalized) => {
                        let last_finalized_reference = last_finalized.reference();

                        trace!(
                            "{}: current finalized: {}",
                            block_range_to_fix,
                            last_finalized_reference,
                        );

                        if block_range_to_fix.start_inclusive <= last_finalized_reference.height {
                            // find the end of the range to fix
                            let end_of_chunk_exclusive =
                                block_range_to_fix.start_inclusive + self.chunk_size as BlockHeight;
                            let end_until_finalized = last_finalized_reference.height + 1;
                            let end_until_last_block_to_fix = block_range_to_fix.end_exclusive;

                            let range_to_fix_end = min(
                                end_of_chunk_exclusive,
                                min(end_until_finalized, end_until_last_block_to_fix),
                            );

                            let range_to_fix: BlockRange =
                                (block_range_to_fix.start_inclusive..range_to_fix_end).into();
                            debug!("{}: fixing: {}", block_range_to_fix, range_to_fix);

                            self.fix_blocks(&last_finalized, range_to_fix.clone())
                                .instrument(info_span!("fix"))
                                .await?;

                            self.remove_blocks_to_fix(range_to_fix).await?
                        }
                    }
                    Err(IndexerError::NoBlock(_)) => {
                        info!("{}: no finalized block => retry later", block_range_to_fix);
                        sleep(Duration::from_millis(1000)).await;
                    }
                    Err(error) => {
                        warn!(
                            "{}: error finding finalized block ({}) => retry later",
                            block_range_to_fix, error,
                        );
                        sleep(Duration::from_millis(1000)).await;
                    }
                }
            } else {
                info!("nothing scheduled to fix => retry later");
                sleep(Duration::from_millis(1000)).await;
            }
        }
    }

    async fn fix_blocks(
        &self,
        last_finalized: &T::BlockHandle,
        block_range: BlockRange,
    ) -> Result<(), Report> {
        last_finalized
            .fetch_range_expect_all(block_range.clone(), FetchMode::Eager, |block| {
                self.fix_block(block)
            })
            .instrument(info_span!("fix"))
            .await?;

        info!("{}: done", &block_range);

        Ok(())
    }

    async fn fix_block(&self, block: T::BlockHandle) -> Result<(), Report> {
        let reference = block.reference();
        debug!("{}: fixing", reference);

        let mut tx = self.pg_pool.begin().await?;

        block
            .update(&mut tx)
            .instrument(info_span!("rewrite"))
            .await?;

        tx.commit().await?;

        debug!("{}: fixed", reference);

        Ok(())
    }

    async fn block_range_to_fix(&self) -> Result<Option<BlockRange>, Report> {
        let mut tx = self.pg_pool.begin().await?;
        let result = get_block_range_to_fix(&mut tx, self.indexer_id.clone()).await?;
        tx.commit().await?;

        Ok(result)
    }

    async fn remove_blocks_to_fix(&self, range: BlockRange) -> Result<(), Report> {
        let mut tx = self.pg_pool.begin().await?;
        update_block_range_to_fix(&mut tx, self.indexer_id.clone(), range).await?;
        tx.commit().await?;

        Ok(())
    }
}
