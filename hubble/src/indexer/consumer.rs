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
        api::IndexerId,
        event::{
            hubble::HubbleEvent,
            supported::SupportedBlockEvent,
            types::{
                BlockHeight, MessageHash, MessageSequence, NatsConsumerSequence,
                NatsStreamSequence, Range, UniversalChainId,
            },
        },
        nats::MessageMeta,
        postgres::{
            block_update::{
                get_block_updates, insert_block_update, max_event_height, update_block_update,
            },
            chain_context::fetch_chain_context_for_universal_chain_id,
            replication_reset::{schedule_enrich_reset, schedule_replication_reset},
        },
        record::{
            change_counter::{Changes, RecordKind},
            event_handler::{delete_event_data_at_height, handle_block_events},
            ChainContext,
        },
    },
    utils::human_readable::human_readable_bytes,
};

pub struct BlockUpdate {
    pub universal_chain_id: UniversalChainId,
    pub height: BlockHeight,
    pub message_sequence: MessageSequence,
    pub delete: bool,
    pub message_hash: MessageHash,
    pub nats_stream_sequence: NatsStreamSequence,
    pub nats_consumer_sequence: NatsConsumerSequence,
}

impl fmt::Display for BlockUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}@{} {}[{}|{}|{}] ({})",
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
        let Some(nats) = &self.nats else {
            info!("no nats configuration => no need to create consumer");
            return Ok(());
        };

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
    }

    async fn run_consumer_loop(&self, consumer: &Consumer<Config>) -> Result<(), IndexerError> {
        debug!("begin");

        let mut messages = consumer
            .batch()
            .max_messages(self.consumer_config.batch_size)
            .messages()
            .await?;

        debug!("waiting");

        while let Some(message) = messages.next().await {
            let message = message.map_err(IndexerError::NatsNextError)?;
            consume(message, |message_sequence, payload| {
                self.handle_message(message_sequence, payload)
            })
            .await?;
        }

        debug!("done");
        Ok(())
    }

    async fn handle_message(
        &self,
        message_meta: MessageMeta,
        payload: Bytes,
    ) -> Result<(), IndexerError> {
        let start_time = std::time::Instant::now();
        debug!("begin");
        let mut tx = self.pg_pool.begin().await?;

        // todo: after splitting hubble load upon begin
        let chain_context =
            fetch_chain_context_for_universal_chain_id(&mut tx, &self.universal_chain_id).await?;

        debug!(
            "got message {message_meta} with payload size {}",
            payload.len(),
        );

        let message: HubbleEvent = serde_json::from_slice(&payload)?;

        info!(
            "got message ({}) {message_meta} with details {message}",
            human_readable_bytes(payload.len())
        );

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
        let max_event_height = &max_event_height(&mut tx, &message_meta.universal_chain_id).await?;
        debug!(
            "handling {message_meta} - actions: {} (max_event_height: {max_event_height})",
            actions.len()
        );

        // there are two kinds of resets:
        // - replication-reset: handled by sync postgres cron jobs (legacy)
        // - enrich-reset: handled by hubble (based on events)

        // keep track if we've already scheduled a replication reset. we don't need to schedule another
        // one thereafter, because we're processing blocks from low to high
        // replication resets will be removed once all events are directly inserted
        let mut did_schedule_replication_reset = false;

        // keep track if we've already scheduled an enrich reset. we don't need to schedule another
        // one thereafter, because we're processing blocks from low to high enrich resets will be
        // removed once all events are directly inserted
        let mut did_schedule_enrich_reset = false;

        for action in &actions {
            let changes = process(&mut tx, &chain_context, action).await?;

            let action_height = &action.height();
            let did_change_before_or_at_latest_height = action_height <= max_event_height;

            // --------------------------------
            // legacy: notify postgres sync job
            // --------------------------------
            let should_schedule_replication_reset = !did_schedule_replication_reset
                && changes.has_changes_for(&[RecordKind::Legacy])
                && did_change_before_or_at_latest_height;

            debug!("handling {action} - reset replication => {should_schedule_replication_reset} (d: {did_schedule_replication_reset}, c: {changes}, h: {action_height}, m: {max_event_height})");
            if should_schedule_replication_reset {
                schedule_replication_reset_for_action(&mut tx, &chain_context, action).await?;

                did_schedule_replication_reset = true;
            }

            // ---------------
            // notify enricher
            // ---------------
            let should_schedule_enrich_reset = !did_schedule_enrich_reset
                && changes.has_changes_matching(should_trigger_enrich_reset)
                && did_change_before_or_at_latest_height;

            debug!("handling {action} - reset enrich => {should_schedule_enrich_reset} (d: {did_schedule_enrich_reset}, c: {changes}, h: {action_height}, m: {max_event_height})");
            if should_schedule_enrich_reset {
                schedule_enrich_reset_for_action(
                    &mut tx,
                    &self.indexer_id,
                    action,
                    max_event_height,
                    &changes,
                )
                .await?;

                did_schedule_enrich_reset = true;
            }
        }

        debug!("commit");
        tx.commit().await?;

        let duration = start_time.elapsed();
        info!("done (took {:.2}ms)", duration.as_secs_f64() * 1000.0);
        Ok(())
    }
}

// filter on changes that affect enrichment
fn should_trigger_enrich_reset(kind: RecordKind) -> bool {
    use RecordKind::*;
    match kind {
        // ignore legacy record changes
        Legacy => false,
        // connection details are used in enrichement
        ChannelOpenInit => true,
        ChannelOpenTry => true,
        ChannelOpenAck => true,
        ChannelOpenConfirm => true,
        // connection details are used in enrichement
        ConnectionOpenInit => true,
        ConnectionOpenTry => true,
        ConnectionOpenAck => true,
        ConnectionOpenConfirm => true,
        // client details are used in enrichement
        CreateClient => true,
        // lens-client does not affect enrichment
        CreateLensClient => false,
        // client updates do not affect enrichment
        UpdateClient => false,
        // packet-send is enriched upon insertion
        PacketSend => false,
        // non-send packet events are not enriched
        PacketRecv => false,
        WriteAck => false,
        PacketAck => false,
        PacketTimeout => false,
        // not related to packets
        TokenBucketUpdate => false,
        WalletMutationEntry => false,
        // ignore enriched records
        PacketSendDecoded => false,
        PacketSendTransfers => false,
        PacketSendInstructionsSearch => false,
    }
}

// return true if data was changed. this is use to determine if a sync reset is required (will be removed when removing sync process)
async fn process<'a>(
    tx: &mut sqlx::Transaction<'static, sqlx::Postgres>,
    chain_context: &ChainContext,
    action: &Action<'a>,
) -> Result<Changes, IndexerError> {
    let start_time = std::time::Instant::now();

    info!("handling {action}");

    let height = action.height();

    let result = Ok(match action {
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
            delete_block(tx, chain_context, height).await?
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

            delete_block(tx, chain_context, height).await?
                + insert_block(tx, chain_context, block_events).await?
        }
        Action::Insert(_, new, block_events) => {
            insert_block_update(tx, new).await?;

            // we don't have records before introducing nats, so we need to delete be sure no
            // old data exists. ultimately we can generate block-update records for each known
            // block so this it not required
            delete_block(tx, chain_context, height).await?
                + insert_block(tx, chain_context, block_events).await?
        }
    });

    let duration = start_time.elapsed();
    info!(
        "handling {action} - done (took {:.2}ms) - {}",
        duration.as_secs_f64() * 1000.0,
        match &result {
            Ok(changes) => format!("changes: {changes}"),
            Err(err) => format!("error: {err}"),
        },
    );

    result
}

async fn delete_block(
    tx: &mut sqlx::Transaction<'static, sqlx::Postgres>,
    chain_context: &ChainContext,
    height: BlockHeight,
) -> Result<Changes, IndexerError> {
    delete_event_data_at_height(tx, chain_context.internal_chain_id, height).await
}

async fn insert_block(
    tx: &mut sqlx::Transaction<'static, sqlx::Postgres>,
    chain_context: &ChainContext,
    block_events: &[&SupportedBlockEvent],
) -> Result<Changes, IndexerError> {
    handle_block_events(tx, chain_context, block_events).await
}

async fn schedule_replication_reset_for_action<'a>(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    chain_context: &ChainContext,
    action: &Action<'a>,
) -> Result<(), IndexerError> {
    schedule_replication_reset(
        tx,
        chain_context,
        action.height(),
        &format!("block reorg ({action})"),
    )
    .await?;

    Ok(())
}

async fn schedule_enrich_reset_for_action<'a>(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    indexer_id: &IndexerId,
    action: &Action<'a>,
    end_inclusive: &BlockHeight,
    changes: &Changes,
) -> Result<(), IndexerError> {
    schedule_enrich_reset(
        tx,
        indexer_id,
        &Range::new_from_start_inclusive_end_inclusive(&action.height(), end_inclusive),
        &format!("{action} => {changes}"),
    )
    .await?;

    Ok(())
}

// what should we do with a block at a specific height.
enum Action<'a> {
    Delete(&'a MessageMeta, &'a BlockUpdate),
    Update(
        &'a MessageMeta,
        &'a BlockUpdate,
        &'a Vec<&'a SupportedBlockEvent>,
    ), // events are guaranteed to belong to the same block height
    Insert(
        #[allow(dead_code)] &'a MessageMeta,
        BlockUpdate,
        &'a Vec<&'a SupportedBlockEvent>,
    ),
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
            Action::Delete(_, delete) => {
                write!(f, "delete: {delete}")
            }
            Action::Update(_, update, events) => {
                write!(f, "update: {update} - {} events", events.len())
            }
            Action::Insert(_, insert, events) => {
                write!(f, "insert: {insert} - {} events", events.len(),)
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
        .map(|meta| {
            (
                NatsStreamSequence::from(meta.stream_sequence),
                NatsConsumerSequence::from(meta.consumer_sequence),
            )
        })
        .map_err(IndexerError::NatsMetaError)?;

    if let Some(header_map) = &message.headers {
        let message_sequence = match header_map.get("Message-Sequence") {
            Some(message_sequence) => message_sequence
                .as_str()
                .parse::<u64>()
                .map(|s| s.into())
                .map_err(|e| {
                    IndexerError::NatsUnparsableMessageSequence(
                        message_sequence.as_str().to_string(),
                        nats_stream_sequence,
                        nats_consumer_sequence,
                        Box::new(e),
                    )
                }),
            None => Err(IndexerError::NatsMissingMessageSequence(
                nats_stream_sequence,
                nats_consumer_sequence,
            )),
        }?;

        let message_hash = match header_map.get("Message-Hash") {
            Some(message_hash) => message_hash.as_str().parse::<MessageHash>().map_err(|e| {
                IndexerError::NatsUnparsableMessageHash(
                    message_hash.as_str().to_string(),
                    nats_stream_sequence,
                    nats_consumer_sequence,
                    Box::new(e),
                )
            }),
            None => Err(IndexerError::NatsMissingMessageHash(
                nats_stream_sequence,
                nats_consumer_sequence,
            )),
        }?;

        let universal_chain_id = match header_map.get("Universal-Chain-Id") {
            Some(universal_chain_id) => Ok(universal_chain_id.as_str().to_string().into()),
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
