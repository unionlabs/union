use std::cmp::min;

use color_eyre::eyre::{eyre, Report};
use tokio::time::sleep;
use tracing::{debug, info, info_span, trace, warn, Instrument};

use super::{
    api::{BlockRange, FetcherClient, IndexerError},
    Indexer,
};
use crate::indexer::{
    api::{BlockHandle, BlockSelection, FetchMode},
    event::Range,
    postgres::block_fix::{
        delete_block_range_to_fix, get_block_fix_status, update_block_range_to_fix_next,
        update_block_range_to_fix_start_and_next,
    },
    HappyRangeFetcher,
};

enum FixerLoopResult {
    RunAgain,
    TryAgainLater,
}

impl<T: FetcherClient> Indexer<T> {
    pub async fn run_fixer(&self, fetcher_client: T) -> Result<(), IndexerError> {
        loop {
            match self.run_fixer_loop(&fetcher_client).await {
                Ok(FixerLoopResult::RunAgain) => {
                    debug!("run again");
                }
                Ok(FixerLoopResult::TryAgainLater) => {
                    debug!(
                        "try again later (sleep {}s)",
                        self.fixer_config.retry_later_sleep.as_secs()
                    );
                    sleep(self.fixer_config.retry_later_sleep).await;
                }
                Err(error) => {
                    warn!(
                        "error in fixer loop: {error} => try again later (sleep {}s)",
                        self.fixer_config.retry_error_sleep.as_secs()
                    );
                    sleep(self.fixer_config.retry_error_sleep).await;
                }
            }
        }
    }

    async fn run_fixer_loop(&self, fetcher_client: &T) -> Result<FixerLoopResult, IndexerError> {
        let chunk_size: u64 = self.chunk_size.try_into().unwrap();

        if let Some(block_range_to_fix) = self.block_range_to_fix().await? {
            info!("{block_range_to_fix}: begin");

            match fetcher_client
                .fetch_single(BlockSelection::LastFinalized, FetchMode::Lazy)
                .await
            {
                Ok(last_finalized) => {
                    let last_finalized_reference = last_finalized.reference();

                    trace!("{block_range_to_fix}: current finalized: {last_finalized_reference}");

                    if block_range_to_fix.start_inclusive <= last_finalized_reference.height {
                        // find the end of the range to fix
                        let end_of_chunk_exclusive =
                            block_range_to_fix.start_inclusive + chunk_size;
                        let end_until_finalized = last_finalized_reference.height + 1;
                        let end_until_last_block_to_fix = block_range_to_fix.end_exclusive;

                        let range_to_fix_end = min(
                            end_of_chunk_exclusive,
                            min(end_until_finalized, end_until_last_block_to_fix),
                        );

                        let range_to_fix: BlockRange =
                            (block_range_to_fix.start_inclusive..range_to_fix_end).into();
                        debug!("{block_range_to_fix}: fixing: {range_to_fix}");

                        self.fix_blocks(&last_finalized, range_to_fix.clone())
                            .instrument(info_span!("fix"))
                            .await?;
                    }

                    Ok(FixerLoopResult::RunAgain)
                }
                Err(IndexerError::NoBlock(_)) => {
                    info!("{block_range_to_fix}: no block to fix => retry later");
                    Ok(FixerLoopResult::TryAgainLater)
                }
                Err(error) => {
                    warn!(
                        "{block_range_to_fix}: error finding block to fix ({error}) => retry later"
                    );
                    Err(error)
                }
            }
        } else {
            debug!("nothing scheduled to fix => retry later");
            Ok(FixerLoopResult::TryAgainLater)
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

        info!("{block_range}: done");

        Ok(())
    }

    async fn fix_block(&self, block: T::BlockHandle) -> Result<(), Report> {
        let reference = block.reference();
        debug!("{reference}: fixing");

        let mut tx = self.pg_pool.begin().await?;

        if let Some(block_fix_status) = get_block_fix_status(&mut tx, &self.indexer_id).await? {
            if block_fix_status.next != block.reference().height {
                warn!("block fix status next {} does not align with the expected height {}, probably due to manual action. reset fixer", block_fix_status.next, block.reference().height);
                return Err(eyre!(
                    "block fix status next {} does not align with the expected height {}",
                    block_fix_status.next,
                    block.reference().height
                ));
            }

            let events = block
                .update(&mut tx)
                .instrument(info_span!("rewrite"))
                .await?;

            let new_next = block.reference().height + 1;
            let last_block = block_fix_status.range.end_exclusive == new_next;
            let has_events = events.is_some();
            let block_count = new_next - block_fix_status.range.start_inclusive;
            let max_blocks = block_count >= self.fixer_config.max_blocks_in_message;

            if last_block || has_events || max_blocks {
                debug!("{reference}: schedule event (last: {last_block}, events: {has_events}, max_blocks: {max_blocks})");

                self.schedule_message(
                    &mut tx,
                    Range {
                        start_inclusive: block_fix_status.range.start_inclusive,
                        end_exclusive: new_next, // new_next is current + 1 (so exclusive)
                    },
                    events,
                )
                .await?;

                if last_block {
                    debug!("{reference}: last block => delete fixer");

                    delete_block_range_to_fix(&mut tx, &block_fix_status).await?;
                } else {
                    // we're updating the start, because we've just sent a message
                    // with all events from start until the current next
                    debug!("{reference}: more blocks => update start and next");

                    update_block_range_to_fix_start_and_next(&mut tx, &block_fix_status, new_next)
                        .await?;
                }
            } else {
                debug!("{reference}: nothing to do (last: {last_block}, events: {has_events}, max_blocks: {max_blocks}) => update next");

                update_block_range_to_fix_next(&mut tx, &block_fix_status, new_next).await?;
            }
        } else {
            warn!("block status record disappeared, probably due to manual action. reset fixer");
            return Err(eyre!("block status record disappeared"));
        }

        tx.commit().await?;

        debug!("{reference}: fixed");

        Ok(())
    }

    async fn block_range_to_fix(&self) -> Result<Option<BlockRange>, Report> {
        let mut tx = self.pg_pool.begin().await?;
        let result = get_block_fix_status(&mut tx, &self.indexer_id)
            .await?
            .map(|b| BlockRange {
                start_inclusive: b.next, // block range to fix starts with 'next'. start points to the last block without events
                end_exclusive: b.range.end_exclusive,
            });
        tx.commit().await?;

        Ok(result)
    }
}
