use tokio::time::sleep;
use tracing::{debug, warn};

use super::{
    api::{FetcherClient, IndexerError},
    Indexer,
};
use crate::indexer::{
    nats::{subject_for_block, NatsConnection},
    postgres::nats::next_to_publish,
};

enum PublisherLoopResult {
    RunAgain,
    TryAgainLater,
}

impl<T: FetcherClient> Indexer<T> {
    pub async fn run_publisher(&self) -> Result<(), IndexerError> {
        let Some(nats) = &self.nats else {
            debug!("no nats configuration => no need to create publisher");
            return Ok(());
        };

        loop {
            match self.run_publisher_loop(nats).await {
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
    }

    async fn run_publisher_loop(
        &self,
        nats: &NatsConnection,
    ) -> Result<PublisherLoopResult, IndexerError> {
        debug!("begin");
        let mut tx = self.pg_pool.begin().await?;

        let subject = subject_for_block(&self.universal_chain_id);

        let messages = next_to_publish(&mut tx, &subject, self.publisher_config.batch_size).await?;

        if messages.is_empty() {
            debug!("nothing scheduled to publish => retry later");

            return Ok(PublisherLoopResult::TryAgainLater);
        }

        debug!("sending (count: {})", messages.len());
        for message in messages {
            let ack = nats.publish(&self.universal_chain_id, &message).await?;

            debug!("{}: acked {ack}", message.id);
        }

        debug!("commit");
        tx.commit().await?;
        debug!("done");
        Ok(PublisherLoopResult::RunAgain)
    }
}
