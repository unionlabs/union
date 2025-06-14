use std::str::from_utf8;

use async_nats::jetstream::consumer::{pull::Config, Consumer};
use bytes::Bytes;
use futures::StreamExt;
use tokio::time::sleep;
use tracing::{debug, info, trace, warn};

use super::{
    api::{FetcherClient, IndexerError},
    Indexer,
};
use crate::indexer::nats::consume;

impl<T: FetcherClient> Indexer<T> {
    pub async fn run_consumer(&self) -> Result<(), IndexerError> {
        if let Some(nats) = &self.nats {
            info!("connection: {nats}");

            let consumer = nats.create_consumer(&self.indexer_id).await?;

            debug!("consumer: {consumer:?}");
            loop {
                match self.run_consumer_loop(&consumer).await {
                    Ok(_) => {
                        debug!("run again");
                    }
                    Err(error) => {
                        warn!(
                            "error in consumer loop: {error} => try again later (sleep {}ms)",
                            self.consumer_config.retry_error_sleep.as_millis()
                        );
                        sleep(self.consumer_config.retry_error_sleep).await;
                    }
                }
            }
        } else {
            info!("no nats configuration => no need to create consumer");
        };

        Ok(())
    }

    async fn run_consumer_loop(&self, consumer: &Consumer<Config>) -> Result<(), IndexerError> {
        debug!("begin");

        let mut messages = consumer
            .batch()
            .max_messages(self.consumer_config.batch_size)
            .messages()
            .await?;

        info!("messages");

        while let Some(message) = messages.next().await {
            let message = message.map_err(IndexerError::NatsNextError)?;
            consume(message, |subject, payload| {
                self.handle_message(subject, payload)
            })
            .await?;
        }

        info!("done");
        Ok(())
    }

    async fn handle_message(&self, subject: String, payload: Bytes) -> Result<(), IndexerError> {
        info!("begin");
        let tx = self.pg_pool.begin().await?;

        info!(
            "got message on subject {} with payload size {}",
            subject,
            payload.len(),
        );

        trace!(
            "got message on subject {} with payload {:?}",
            subject,
            from_utf8(&payload).map_err(|e| IndexerError::InternalError(e.into()))?
        );

        info!("commit");
        tx.commit().await?;

        info!("done");
        Ok(())
    }
}
