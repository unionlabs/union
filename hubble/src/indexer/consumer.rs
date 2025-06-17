use std::{future::Future, time::Duration};

use async_nats::jetstream::{
    self,
    consumer::{pull::Config, Consumer},
};
use bytes::Bytes;
use futures::StreamExt;
use lz4_flex::decompress_size_prepended;
use tokio::time::sleep;
use tracing::{debug, info, trace, warn};

use super::{
    api::{FetcherClient, IndexerError},
    Indexer,
};
use crate::indexer::{
    event::{HubbleEvent, MessageHash},
    nats::MessageMeta,
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
            let message = message.map_err(IndexerError::NatsNextError)?;
            consume(message, |message_sequence, payload| {
                self.handle_message(message_sequence, payload)
            })
            .await?;
        }

        info!("done");
        Ok(())
    }

    async fn handle_message(
        &self,
        message_meta: MessageMeta,
        payload: Bytes,
    ) -> Result<(), IndexerError> {
        info!("begin");
        let tx = self.pg_pool.begin().await?;

        info!(
            "got message {message_meta} with payload size {}",
            payload.len(),
        );

        let message: HubbleEvent = serde_json::from_slice(&payload)?;

        trace!("got message {message_meta} with details {message}");

        info!("commit");
        tx.commit().await?;

        info!("done");
        Ok(())
    }
}

async fn consume<F, Fut>(
    message: async_nats::jetstream::Message,
    handler: F,
) -> Result<(), IndexerError>
where
    F: Fn(MessageMeta, Bytes) -> Fut,
    Fut: Future<Output = Result<(), IndexerError>>,
{
    trace!("consume - raw: {} bytes (encoded)", message.payload.len());

    let message_meta = get_message_meta(&message)?;
    debug!("consume - meta: {message_meta}");

    let message_data = get_message_data(&message).await?;
    trace!("consume - data: {} byte (decoded)", message_data.len());

    match handler(message_meta, message_data).await {
        Ok(_) => {
            debug!("acking");
            message.ack().await.map_err(IndexerError::NatsAckError)?;
        }
        Err(e) => {
            warn!("nacking: {e:?}");
            message
                .ack_with(jetstream::AckKind::Nak(Some(Duration::from_secs(60))))
                .await
                .map_err(IndexerError::NatsNackError)?;
        }
    }

    Ok(())
}

async fn get_message_data(message: &async_nats::jetstream::Message) -> Result<Bytes, IndexerError> {
    let payload = &message.payload;

    if let Some(encoding) = message
        .headers
        .as_ref()
        .and_then(|h| h.get("Content-Encoding"))
    {
        match encoding.as_str() {
            "lz4" => Ok(decompress_size_prepended(payload)?.into()),
            _ => {
                warn!("nacking - unsupported encoding: {encoding}");

                // TODO: improve nack flow
                message
                    .ack_with(jetstream::AckKind::Nak(Some(Duration::from_secs(60))))
                    .await
                    .map_err(IndexerError::NatsNackError)?;

                Err(IndexerError::NatsUnsupportedEncoding(encoding.to_string()))
            }
        }
    } else {
        Ok(payload.clone())
    }
}

fn get_message_meta(message: &async_nats::jetstream::Message) -> Result<MessageMeta, IndexerError> {
    let (nats_stream_sequence, nats_consumer_sequence) = message
        .info()
        .map(|meta| (meta.stream_sequence, meta.consumer_sequence))
        .map_err(IndexerError::NatsMetaError)?;

    if let Some(header_map) = &message.headers {
        let message_sequence = match header_map.get("Message-Sequence") {
            Some(message_sequence) => message_sequence.as_str().parse::<u64>().map_err(|e| {
                IndexerError::NatsUnparseableMessageSequence(
                    message_sequence.as_str().to_string(),
                    nats_stream_sequence,
                    nats_consumer_sequence,
                    e,
                )
            }),
            None => Err(IndexerError::NatsMissingMessageSequence(
                nats_stream_sequence,
                nats_consumer_sequence,
            )),
        }?;

        let message_hash = match header_map.get("Message-Hash") {
            Some(message_hash) => message_hash.as_str().parse::<MessageHash>().map_err(|e| {
                IndexerError::NatsUnparseableMessageHash(
                    message_hash.as_str().to_string(),
                    nats_stream_sequence,
                    nats_consumer_sequence,
                    e,
                )
            }),
            None => Err(IndexerError::NatsMissingMessageHash(
                nats_stream_sequence,
                nats_consumer_sequence,
            )),
        }?;

        Ok(MessageMeta {
            message_sequence,
            message_hash,
            nats_stream_sequence,
            nats_consumer_sequence,
            subject: message.subject.to_string(),
        })
    } else {
        Err(IndexerError::NatsMissingMessageHeaders(
            nats_stream_sequence,
            nats_consumer_sequence,
        ))
    }
}
