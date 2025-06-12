use async_nats::jetstream::Context;
use tokio::time::sleep;
use tracing::{debug, info, warn};

use super::{
    api::{FetcherClient, IndexerError},
    Indexer,
};
use crate::indexer::{nats::subject_for_block, postgres::next_to_publish};

enum PublisherLoopResult {
    RunAgain,
    TryAgainLater,
}

impl<T: FetcherClient> Indexer<T> {
    pub async fn run_publisher(&self) -> Result<(), IndexerError> {
        if let Some(nats) = &self.nats {
            loop {
                match self.run_publisher_loop(&nats.context).await {
                    Ok(PublisherLoopResult::RunAgain) => {
                        debug!("run again");
                    }
                    Ok(PublisherLoopResult::TryAgainLater) => {
                        debug!(
                            "try again later (sleep {}ms)",
                            self.publisher_config.retry_later_sleep.as_millis()
                        );
                        sleep(self.publisher_config.retry_later_sleep).await;
                    }
                    Err(error) => {
                        warn!(
                            "error in publisher loop: {error} => try again later (sleep {}ms)",
                            self.publisher_config.retry_error_sleep.as_millis()
                        );
                        sleep(self.publisher_config.retry_error_sleep).await;
                    }
                }
            }
        } else {
            info!("no nats configuration => no need to create publisher");
        };

        Ok(())
    }

    async fn run_publisher_loop(
        &self,
        nats: &Context,
    ) -> Result<PublisherLoopResult, IndexerError> {
        debug!("begin");
        let mut tx = self.pg_pool.begin().await?;

        let subject = subject_for_block(&self.indexer_id);

        let messages = next_to_publish(&mut tx, &subject, self.publisher_config.batch_size).await?;

        if messages.is_empty() {
            info!("nothing scheduled to publish => retry later");

            return Ok(PublisherLoopResult::TryAgainLater);
        }

        info!("sending (count: {})", messages.len());
        for message in messages {
            info!("{}: sending", message.id);

            let ack_future = nats
                .publish_with_headers(message.subject, message.headers, message.data)
                .await?;

            info!("{}: acking", message.id);
            let ack = ack_future.await?;

            info!("{}: acked (sequence: {})", message.id, ack.sequence);
        }

        debug!("commit");
        tx.commit().await?;
        info!("done");
        Ok(PublisherLoopResult::RunAgain)
    }
}
