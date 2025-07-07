use sqlx::Postgres;
use tokio::time::sleep;
use tracing::{debug, info, trace, warn};

use crate::indexer::{
    api::{FetcherClient, IndexerError},
    enrich::{delete_enriched_data_for_block, enrich},
    event::types::BlockHeight,
    postgres::{
        block_enrich::{
            block_enrich_delete, block_enrich_status, block_enrich_update, next_height_to_enrich,
        },
        lock::try_lock_block,
    },
    record::{change_counter::Changes, packet_send_record::PacketSendRecord},
    Indexer,
};

enum EnricherLoopResult {
    RunAgain,
    TryAgainLater,
}

impl<T: FetcherClient> Indexer<T> {
    pub async fn run_enricher(&self, fetcher_client: T) -> Result<(), IndexerError> {
        if self.drain {
            return Ok(());
        }

        loop {
            match self.run_enricher_loop(&fetcher_client).await {
                Ok(EnricherLoopResult::RunAgain) => {
                    debug!("run again");
                }
                Ok(EnricherLoopResult::TryAgainLater) => {
                    debug!(
                        "try again later (sleep {}s)",
                        self.enricher_config.retry_later_sleep.as_secs()
                    );
                    sleep(self.enricher_config.retry_later_sleep).await;
                }
                Err(error) => {
                    warn!(
                        "error in enricher loop: {error} => try again later (sleep {}s)",
                        self.enricher_config.retry_error_sleep.as_secs()
                    );
                    sleep(self.enricher_config.retry_error_sleep).await;
                }
            }
        }
    }

    async fn run_enricher_loop(
        &self,
        _fetcher_client: &T,
    ) -> Result<EnricherLoopResult, IndexerError> {
        let start_time = std::time::Instant::now();

        let mut tx = self.pg_pool.begin().await?;

        info!("begin");

        if let Some(block_range_to_enrich) =
            block_enrich_status(&mut tx, &self.universal_chain_id).await?
        {
            info!("{block_range_to_enrich}: determine next height");

            let changes = match next_height_to_enrich(
                &mut tx,
                &self.universal_chain_id,
                &block_range_to_enrich,
            )
            .await?
            {
                Some(height_to_enrich) => {
                    trace!("{block_range_to_enrich} : enrich {height_to_enrich}");

                    trace!("{block_range_to_enrich} : locking {height_to_enrich}");
                    // we're locking to prevent that the fetcher will write new data for this block
                    try_lock_block(&mut tx, &self.universal_chain_id, height_to_enrich).await?;

                    trace!("{block_range_to_enrich} : enriching {height_to_enrich}");
                    let changes = self.enrich_height(&mut tx, &height_to_enrich).await?;

                    trace!("{block_range_to_enrich} : update status {height_to_enrich}");
                    let new_start_height = height_to_enrich.next();
                    if new_start_height == block_range_to_enrich.range.end_exclusive.into() {
                        block_enrich_delete(&mut tx, &block_range_to_enrich).await?;
                    } else {
                        block_enrich_update(&mut tx, &block_range_to_enrich, &new_start_height)
                            .await?;
                    }

                    changes
                }
                None => {
                    info!("{block_range_to_enrich} : nothing to enrich");

                    block_enrich_delete(&mut tx, &block_range_to_enrich).await?;

                    Changes::default()
                }
            };

            trace!("{block_range_to_enrich} : committing");

            tx.commit().await?;

            trace!(
                "{block_range_to_enrich} : done ({:.2}ms) {changes}",
                start_time.elapsed().as_secs_f64() * 1000.0
            );

            Ok(EnricherLoopResult::RunAgain)
        } else {
            debug!("nothing scheduled to enrich => retry later");
            Ok(EnricherLoopResult::TryAgainLater)
        }
    }

    async fn enrich_height(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        height: &BlockHeight,
    ) -> Result<Changes, IndexerError> {
        info!("enrich_height : {height} begin");

        let deleted = delete_enriched_data_for_block(tx, &self.universal_chain_id, height).await?;
        debug!("enrich_height : {height} deleted: {deleted}");

        let mut inserted = Changes::default();

        for packet_send_record in
            PacketSendRecord::find_by_chain_and_height(tx, &self.universal_chain_id, height).await?
        {
            debug!(
                "enrich_height : {height} packet-hash: {}",
                hex::encode(&packet_send_record.packet_hash)
            );

            inserted += enrich(tx, packet_send_record).await?;
        }

        debug!("enrich_height : {height} inserted: {inserted}");

        Ok(deleted + inserted)
    }
}
