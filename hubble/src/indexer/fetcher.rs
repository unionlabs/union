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
            debug!("fetching last finalized block");
            match fetcher_client
                .fetch_single(BlockSelection::LastFinalized, FetchMode::Lazy)
                .await
            {
                Ok(last_finalized) => {
                    let next_height = self.next_height().await?;
                    if next_height + self.chunk_size as u64 > last_finalized.reference().height {
                        info!("near finalized height (current: {} finalized: {}) => start 'run to tip'", next_height, last_finalized.reference());
                        return Ok(());
                    }

                    let catch_up_range: BlockRange =
                        (next_height..last_finalized.reference().height).into();
                    info!("missing blocks: {}", catch_up_range);

                    for slice in catch_up_range.range_chunks(self.chunk_size).into_iter() {
                        info!("{}: handling chunk", slice);

                        last_finalized
                            .fetch_range_expect_all(slice.clone(), FetchMode::Eager, |block| {
                                self.store_block(block)
                            })
                            .instrument(info_span!("store"))
                            .await?;

                        info!("{}: handled chunk", &slice);
                    }
                }
                Err(IndexerError::NoBlock(_)) => {
                    info!("no finalized block => start 'run to tip'");
                    return Ok(());
                }
                Err(error) => {
                    info!("error reading: {}", error);
                    return Err(error);
                }
            };
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
                }
                Err(IndexerError::NoBlock(_)) => {
                    debug!("{}: no block yet => sleep", next_height);
                    sleep(Duration::from_millis(1000)).await;
                }
                Err(_) => {
                    warn!("{}: error reading block => sleep", next_height);
                    sleep(Duration::from_millis(1000)).await;
                }
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
