use std::str::from_utf8;

use async_nats::jetstream::consumer::{pull::Config, Consumer};
use futures::StreamExt;
use tokio::time::sleep;
use tracing::{debug, info, warn};

use super::{
    api::{FetcherClient, IndexerError},
    Indexer,
};

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
            debug!("message");
            let message = message.map_err(IndexerError::NatsNextError)?;
            let meta = message.info().map_err(IndexerError::NatsMetaError)?;

            debug!("Stream sequence number: {}", meta.stream_sequence);
            debug!("Consumer sequence number: {}", meta.consumer_sequence);

            let tx = self.pg_pool.begin().await?;

            info!(
                "got message on subject {} with payload {:?}",
                message.subject,
                from_utf8(&message.payload).expect("conversion")
            );

            debug!("committing");
            tx.commit().await?;

            debug!("acking");
            message.ack().await.map_err(IndexerError::NatsAckError)?;
            debug!("acked");
        }

        info!("done");
        Ok(())
    }
}
