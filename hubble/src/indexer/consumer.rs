use std::{collections::HashMap, fmt, future::Future, time::Duration};

use async_nats::jetstream::{
    self,
    consumer::{pull::Config, Consumer},
};
use bytes::Bytes;
use futures::StreamExt;
use itertools::Itertools;
use lz4_flex::decompress_size_prepended;
use tokio::time::sleep;
use tracing::{debug, info, trace, warn};

use super::{
    api::{FetcherClient, IndexerError},
    Indexer,
};
use crate::{
    indexer::{
        api::{BlockHeight, UniversalChainId},
        event::{BlockEvent, HubbleEvent, MessageHash},
        nats::MessageMeta,
        postgres::{
            block_update::{get_block_updates, insert_block_update, update_block_update},
            event_data::{
                delete_event_data_at_height, insert_event_data_with_events, max_event_height,
            },
        },
    },
    postgres::{fetch_internal_chain_id_for_universal_chain_id, schedule_replication_reset},
};

pub struct BlockUpdate {
    pub universal_chain_id: UniversalChainId,
    pub height: BlockHeight,
    pub message_sequence: u64,
    pub delete: bool,
    pub message_hash: MessageHash,
    pub nats_stream_sequence: u64,
    pub nats_consumer_sequence: u64,
}

impl fmt::Display for BlockUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}@{} {}[m{}|s{}|c{}] ({})",
            self.universal_chain_id,
            self.height,
            if self.delete { "-" } else { "+" },
            self.message_sequence,
            self.nats_stream_sequence,
            self.nats_consumer_sequence,
            self.message_hash,
        )
    }
}

impl<T: FetcherClient> Indexer<T> {
    pub async fn run_consumer(&self) -> Result<(), IndexerError> {
        if let Some(nats) = &self.nats {
            info!("connection: {nats}");

            let consumer = nats.create_consumer(&self.universal_chain_id).await?;

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
        info!("begin");

        let mut messages = consumer
            .batch()
            .max_messages(self.consumer_config.batch_size)
            .messages()
            .await?;

        info!("waiting");

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
        debug!("begin");
        let mut tx = self.pg_pool.begin().await?;

        debug!(
            "got message {message_meta} with payload size {}",
            payload.len(),
        );

        let message: HubbleEvent = serde_json::from_slice(&payload)?;

        info!("got message {message_meta} with details {message}");

        let block_events = message.events_by_height();

        let block_updates: HashMap<BlockHeight, BlockUpdate> =
            get_block_updates(&mut tx, &message.universal_chain_id, &message.range)
                .await?
                .into_iter()
                .map(|b| (b.height, b))
                .collect();

        // blocks that we already received and updated by this message
        let updates = block_updates
            .values()
            // only consider updating a block when the message sequence is before this message
            .filter(|block_update| block_update.message_sequence < message_meta.message_sequence)
            .map(
                |block_update| match block_events.get(&block_update.height) {
                    Some(block_events) => Action::Update(&message_meta, block_update, block_events),
                    None => Action::Delete(&message_meta, block_update),
                },
            )
            .collect_vec();

        // blocks that we never received and inserted by this message
        let inserts = block_events
            .iter()
            .filter_map(|(block_height, block_events)| {
                match block_updates
                    .contains_key(&block_events.first().expect("at least one event").height())
                {
                    // block is already registered, so action is already in blocks_to_update or it
                    // is ignored, because block has higher message sequence
                    true => None,
                    false => {
                        let block_update = BlockUpdate {
                            universal_chain_id: message_meta.universal_chain_id.clone(),
                            height: *block_height,
                            message_sequence: message_meta.message_sequence,
                            delete: false,
                            message_hash: message_meta.message_hash.clone(),
                            nats_stream_sequence: message_meta.nats_stream_sequence,
                            nats_consumer_sequence: message_meta.nats_consumer_sequence,
                        };

                        Some(Action::Insert(&message_meta, block_update, block_events))
                    }
                }
            })
            .collect_vec();

        let actions = updates
            .into_iter()
            .chain(inserts.into_iter())
            .sorted_by_key(|action| action.height())
            .collect_vec();

        // fetch the maximum height of currently stored data. we should trigger a sync when
        // changing data at or before this height
        let max_event_height = max_event_height(&mut tx, &message_meta.universal_chain_id).await?;
        info!(
            "handling {message_meta} - actions: {} (max_event_height: {max_event_height})",
            actions.len()
        );

        // keep track if we've already scheduled a replication reset. we don't need to schedule another
        // one thereafter, because we're processing blocks from low to high
        // replication resets will be removed once all events are directly inserted
        let mut did_schedule_replication_reset = false;

        for action in actions {
            let data_changed = process(&mut tx, &action).await?;

            let should_schedule_reset = !did_schedule_replication_reset
                && data_changed
                && action.height() <= max_event_height;

            debug!("handling {action} - reset replication => {should_schedule_reset} (d: {did_schedule_replication_reset}, c: {data_changed}, h: {}, m: {max_event_height})", action.height());
            if should_schedule_reset {
                schedule_replication_reset_for_action(
                    &mut tx,
                    &message_meta.universal_chain_id,
                    action,
                )
                .await?;

                did_schedule_replication_reset = true;
            }
        }

        info!("commit");
        tx.commit().await?;

        info!("done");
        Ok(())
    }
}

// return true if data was changed. this is use to determine if a sync reset is required (will be removed when removing sync process)
async fn process<'a>(
    tx: &mut sqlx::Transaction<'static, sqlx::Postgres>,
    action: &Action<'a>,
) -> Result<bool, IndexerError> {
    info!("handling {action}");

    let height = action.height();

    Ok(match action {
        Action::Delete(message_meta, current) => {
            let new = BlockUpdate {
                universal_chain_id: current.universal_chain_id.clone(),
                height: current.height,
                message_sequence: message_meta.message_sequence,
                delete: true,
                message_hash: message_meta.message_hash.clone(),
                nats_stream_sequence: message_meta.nats_stream_sequence,
                nats_consumer_sequence: message_meta.nats_consumer_sequence,
            };

            update_block_update(tx, new).await?;
            delete_block(tx, &current.universal_chain_id, height).await?
        }
        Action::Update(message_meta, current, block_events) => {
            let new = BlockUpdate {
                universal_chain_id: current.universal_chain_id.clone(),
                height,
                message_sequence: message_meta.message_sequence,
                delete: false,
                message_hash: message_meta.message_hash.clone(),
                nats_stream_sequence: message_meta.nats_stream_sequence,
                nats_consumer_sequence: message_meta.nats_consumer_sequence,
            };

            update_block_update(tx, new).await?;

            delete_block(tx, &current.universal_chain_id, height).await?;
            insert_block(tx, block_events).await?
        }
        Action::Insert(message_meta, new, block_events) => {
            insert_block_update(tx, new).await?;

            // we don't have records before introducing nats, so we need to delete be sure no
            // old data exists. ultimately we can generate block-update records for each known
            // block so this it not required
            delete_block(tx, &message_meta.universal_chain_id, height).await?;
            insert_block(tx, block_events).await?
        }
    })
}

async fn delete_block(
    tx: &mut sqlx::Transaction<'static, sqlx::Postgres>,
    universal_chain_id: &UniversalChainId,
    height: u64,
) -> Result<bool, IndexerError> {
    Ok(delete_event_data_at_height(tx, universal_chain_id, height).await?)
}

async fn insert_block(
    tx: &mut sqlx::Transaction<'static, sqlx::Postgres>,
    block_events: &[&BlockEvent],
) -> Result<bool, IndexerError> {
    Ok(insert_event_data_with_events(tx, block_events).await?)
}

async fn schedule_replication_reset_for_action<'a>(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    universal_chain_id: &UniversalChainId,
    action: Action<'a>,
) -> sqlx::Result<()> {
    let internal_chain_id =
        fetch_internal_chain_id_for_universal_chain_id(tx, universal_chain_id).await?;

    schedule_replication_reset(
        tx,
        internal_chain_id,
        i64::try_from(action.height()).expect("height fits"),
        &format!("block reorg ({action})"),
    )
    .await?;

    Ok(())
}

// what should we do with a block at a specific height.
enum Action<'a> {
    Delete(&'a MessageMeta, &'a BlockUpdate),
    Update(&'a MessageMeta, &'a BlockUpdate, &'a Vec<&'a BlockEvent>), // events are guaranteed to belong to the same block height
    Insert(&'a MessageMeta, BlockUpdate, &'a Vec<&'a BlockEvent>),
}

impl<'a> Action<'a> {
    fn height(&self) -> BlockHeight {
        match self {
            Action::Delete(_, block_update) => block_update.height,
            Action::Update(_, block_update, _) => block_update.height,
            Action::Insert(_, block_update, _) => block_update.height,
        }
    }
}

impl<'a> fmt::Display for Action<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Delete(meta, delete) => {
                write!(f, "delete: {meta}: {delete}")
            }
            Action::Update(meta, update, events) => {
                write!(f, "update: {meta}: {update} - {} events", events.len())
            }
            Action::Insert(meta, insert, events) => {
                write!(f, "insert: {meta} - {insert} - {} events", events.len(),)
            }
        }
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

        let universal_chain_id = match header_map.get("Universal-Chain-Id") {
            Some(message_hash) => Ok(message_hash.as_str().to_string()),
            None => Err(IndexerError::NatsMissingUniversalChainId(
                nats_stream_sequence,
                nats_consumer_sequence,
            )),
        }?;

        Ok(MessageMeta {
            subject: message.subject.to_string(),
            universal_chain_id,
            message_sequence,
            message_hash,
            nats_stream_sequence,
            nats_consumer_sequence,
        })
    } else {
        Err(IndexerError::NatsMissingMessageHeaders(
            nats_stream_sequence,
            nats_consumer_sequence,
        ))
    }
}
