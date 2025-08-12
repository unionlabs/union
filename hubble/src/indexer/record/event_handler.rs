use sqlx::{Postgres, Transaction};
use tracing::{debug, trace};

use crate::indexer::{
    api::IndexerError,
    event::{supported::SupportedBlockEvent, types::BlockHeight},
    record::{
        change_counter::{Changes, LegacyRecord},
        channel_open_ack_record::ChannelOpenAckRecord,
        channel_open_confirm_record::ChannelOpenConfirmRecord,
        channel_open_init_record::ChannelOpenInitRecord,
        channel_open_try_record::ChannelOpenTryRecord,
        connection_open_ack_record::ConnectionOpenAckRecord,
        connection_open_confirm_record::ConnectionOpenConfirmRecord,
        connection_open_init_record::ConnectionOpenInitRecord,
        connection_open_try_record::ConnectionOpenTryRecord,
        create_client_record::CreateClientRecord,
        create_lens_client_record::CreateLensClientRecord,
        create_wrapped_token_record::CreateWrappedTokenRecord,
        create_wrapped_token_relation_record::CreateWrappedTokenRelationRecord,
        packet_ack_record::PacketAckRecord,
        packet_recv_record::PacketRecvRecord,
        packet_send_decoded_record::PacketSendDecodedRecord,
        packet_send_instructions_search_record::PacketSendInstructionsSearchRecord,
        packet_send_record::PacketSendRecord,
        packet_send_transfers_record::PacketSendTransfersRecord,
        packet_timeout_record::PacketTimeoutRecord,
        token_bucket_update_record::TokenBucketUpdateRecord,
        update_client_record::UpdateClientRecord,
        wallet_mutation_entry_record::WalletMutationEntryRecord,
        write_ack_record::WriteAckRecord,
        ChainContext, InternalChainId, PgValue,
    },
};

pub async fn delete_event_data_at_height(
    tx: &mut Transaction<'_, Postgres>,
    internal_chain_id: InternalChainId,
    height: BlockHeight,
) -> Result<Changes, IndexerError> {
    debug!("delete_event_data_at_height: {internal_chain_id}@{height}");
    let mut changes = Changes::default();

    if has_event_data_at_height(tx, internal_chain_id, height).await? {
        debug!("delete_event_data_at_height: {internal_chain_id}@{height} => deleting");

        changes += Changes::with_single_delete::<LegacyRecord>(); // count doesn't matter, we'll delete these tables once we're in production

        sqlx::query!(
            "
            WITH delete_cosmos_events AS (
                DELETE FROM v2_cosmos.events WHERE internal_chain_id = $1 AND height = $2
            ),
            delete_cosmos_transactions AS (
                DELETE FROM v2_cosmos.transactions WHERE internal_chain_id = $1 AND height = $2
            ),
            delete_cosmos_blocks AS (
                DELETE FROM v2_cosmos.blocks WHERE internal_chain_id = $1 AND height = $2
            )
            DELETE FROM v2_evm.logs WHERE internal_chain_id = $1 AND height = $2
            ",
            internal_chain_id.pg_value()?,
            height.pg_value()?,
        )
        .execute(tx.as_mut())
        .await?;

        // would be nice to detect if we're forgetting a record type. maybe not required. depends
        // on the update strategy. Maybe we'll first fetch all existing records to see the impact
        // of deleting them. then we'll have references to all records, so we can delete them
        // one by one.

        changes += ChannelOpenInitRecord::delete_by_chain_and_height(tx, internal_chain_id, height)
            .await?;
        changes +=
            ChannelOpenTryRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        changes +=
            ChannelOpenAckRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        changes +=
            ChannelOpenConfirmRecord::delete_by_chain_and_height(tx, internal_chain_id, height)
                .await?;
        changes +=
            ConnectionOpenInitRecord::delete_by_chain_and_height(tx, internal_chain_id, height)
                .await?;
        changes +=
            ConnectionOpenTryRecord::delete_by_chain_and_height(tx, internal_chain_id, height)
                .await?;
        changes +=
            ConnectionOpenAckRecord::delete_by_chain_and_height(tx, internal_chain_id, height)
                .await?;
        changes +=
            ConnectionOpenConfirmRecord::delete_by_chain_and_height(tx, internal_chain_id, height)
                .await?;
        changes +=
            CreateClientRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        changes +=
            CreateLensClientRecord::delete_by_chain_and_height(tx, internal_chain_id, height)
                .await?;
        changes +=
            UpdateClientRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        changes +=
            PacketSendRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        changes +=
            PacketRecvRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        changes +=
            WriteAckRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        changes +=
            PacketAckRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        changes +=
            PacketTimeoutRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        changes +=
            TokenBucketUpdateRecord::delete_by_chain_and_height(tx, internal_chain_id, height)
                .await?;
        changes +=
            WalletMutationEntryRecord::delete_by_chain_and_height(tx, internal_chain_id, height)
                .await?;
        changes +=
            PacketSendDecodedRecord::delete_by_chain_and_height(tx, internal_chain_id, height)
                .await?;
        changes +=
            PacketSendTransfersRecord::delete_by_chain_and_height(tx, internal_chain_id, height)
                .await?;
        changes += PacketSendInstructionsSearchRecord::delete_by_chain_and_height(
            tx,
            internal_chain_id,
            height,
        )
        .await?;
        changes +=
            CreateWrappedTokenRecord::delete_by_chain_and_height(tx, internal_chain_id, height)
                .await?;
        changes += CreateWrappedTokenRelationRecord::delete_by_chain_and_height(
            tx,
            internal_chain_id,
            height,
        )
        .await?;
    } else {
        debug!("delete_event_data_at_height: {internal_chain_id}@{height} => nothing to delete");
    };

    Ok(changes)
}

async fn has_event_data_at_height(
    tx: &mut Transaction<'_, Postgres>,
    internal_chain_id: InternalChainId,
    height: BlockHeight,
) -> Result<bool, IndexerError> {
    debug!("has_event_data_at_height: {internal_chain_id}@{height}");

    Ok(sqlx::query!(
        "
        SELECT NOT delete AS exists
        FROM hubble.block_update
        WHERE universal_chain_id = (SELECT family || '.' || chain_id AS universal_chain_id FROM config.chains WHERE id = $1)
          AND height = $2
        UNION ALL
        SELECT TRUE AS exists FROM v2_cosmos.events WHERE internal_chain_id = $1 AND height = $2
        UNION ALL
        SELECT TRUE AS exists FROM v2_cosmos.transactions WHERE internal_chain_id = $1 AND height = $2
        UNION ALL
        SELECT TRUE AS exists FROM v2_cosmos.blocks WHERE internal_chain_id = $1 AND height = $2
        UNION ALL
        SELECT TRUE AS exists FROM v2_evm.logs WHERE internal_chain_id = $1 AND height = $2
        LIMIT 1
        ",
        internal_chain_id.pg_value()?,
        height.pg_value()?,
    )
    .fetch_optional(tx.as_mut())
    .await?
    .map(|record| record.exists.expect("exists column value"))
    .unwrap_or(false))
}

pub async fn handle_block_events(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    chain_context: &ChainContext,
    block_events: &[&SupportedBlockEvent],
) -> Result<Changes, IndexerError> {
    let mut changes = Changes::default();

    for block_event in block_events {
        changes += handle_block_event(tx, chain_context, block_event).await?;
    }

    Ok(changes)
}

async fn handle_block_event(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    chain_context: &ChainContext,
    block_event: &SupportedBlockEvent,
) -> Result<Changes, IndexerError> {
    trace!("handling: {block_event:?}");

    Ok(match block_event {
        SupportedBlockEvent::EthereumLog {
                        internal_chain_id,
                        block_hash,
                        data,
                        height,
                        time,
            } => {
                sqlx::query!(
                    "
                INSERT INTO v2_evm.logs (internal_chain_id, block_hash, data, height, time)
                VALUES ($1, $2, $3, $4, $5)
                ",
                    internal_chain_id,
                    block_hash,
                    data,
                    height.pg_value()?,
                    time
                )
                .execute(tx.as_mut())
                .await
                .map(|_| Changes::with_single_insert::<LegacyRecord>())
                .map_err(IndexerError::DatabaseError)?
            },
        SupportedBlockEvent::TendermintBlock {
                internal_chain_id,
                hash,
                data,
                height,
                time,
            } => {
                sqlx::query!(
                    "
                INSERT INTO v2_cosmos.blocks (internal_chain_id, hash, data, height, time)
                VALUES ($1, $2, $3, $4, $5)
                ",
                    internal_chain_id,
                    hash,
                    data,
                    height.pg_value()?,
                    time
                )
                .execute(tx.as_mut())
                .await
                .map(|_| Changes::with_single_insert::<LegacyRecord>())
                .map_err(IndexerError::DatabaseError)?
            },
        SupportedBlockEvent::TendermintTransaction {
                internal_chain_id,
                block_hash,
                height,
                hash,
                data,
                index,
            } => {
                sqlx::query!("
                INSERT INTO v2_cosmos.transactions (internal_chain_id, block_hash, height, hash, data, index) 
                VALUES ($1, $2, $3, $4, $5, $6)
                ",
                    internal_chain_id, block_hash, height.pg_value()?, hash, data, index)
                .execute(tx.as_mut())
                .await
                .map(|_| Changes::with_single_insert::<LegacyRecord>())
                .map_err(IndexerError::DatabaseError)?
            },
        SupportedBlockEvent::TendermintEvent {
                internal_chain_id,
                block_hash,
                height,
                transaction_hash,
                index,
                transaction_index,
                data,
                time,
                flow,
            } => {
                sqlx::query!("
                INSERT INTO v2_cosmos.events (internal_chain_id, block_hash, height, transaction_hash, index, transaction_index, data, time, flow)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ",
                    internal_chain_id,
                    block_hash,
                    height.pg_value()?,
                    transaction_hash as _,
                    index,
                    transaction_index as _,
                    data,
                    time,
                    flow)
                .execute(tx.as_mut())
                .await
                .map(|_| Changes::with_single_insert::<LegacyRecord>())
                .map_err(IndexerError::DatabaseError)?
            },
        SupportedBlockEvent::EthereumDecodedLog { .. } => {
            trace!("ignore decoded log");
            Changes::default()
        },
        SupportedBlockEvent::ChannelOpenInit { inner } => {
            chain_context.with_event(inner).handle(tx).await?
        },
        SupportedBlockEvent::ChannelOpenTry { inner } => {
            chain_context.with_event(inner).handle(tx).await?
        },
        SupportedBlockEvent::ChannelOpenAck { inner } => {
            chain_context.with_event(inner).handle(tx).await?
        },
        SupportedBlockEvent::ChannelOpenConfirm { inner } => {
            chain_context.with_event(inner).handle(tx).await?
        },
        SupportedBlockEvent::ConnectionOpenInit { inner } => {
            chain_context.with_event(inner).handle(tx).await?
        },
        SupportedBlockEvent::ConnectionOpenTry { inner } => {
            chain_context.with_event(inner).handle(tx).await?
        },
        SupportedBlockEvent::ConnectionOpenAck { inner } => {
            chain_context.with_event(inner).handle(tx).await?
        },
        SupportedBlockEvent::ConnectionOpenConfirm { inner } => {
            chain_context.with_event(inner).handle(tx).await?
        },
        SupportedBlockEvent::CreateClient { inner } => {
            chain_context.with_event(inner).handle(tx).await?
        },
        SupportedBlockEvent::CreateLensClient { inner } => {
            chain_context.with_event(inner).handle(tx).await?
        },
        SupportedBlockEvent::UpdateClient { inner } => {
            chain_context.with_event(inner).handle(tx).await?
        },
        SupportedBlockEvent::PacketSend { inner } => {
            chain_context.with_event(inner).handle(tx).await?
        },
        SupportedBlockEvent::PacketRecv { inner } => {
            chain_context.with_event(inner).handle(tx).await?
        },
        SupportedBlockEvent::WriteAck { inner } => {
            chain_context.with_event(inner).handle(tx).await?
        },
        SupportedBlockEvent::PacketAck { inner } => {
            chain_context.with_event(inner).handle(tx).await?
        },
        SupportedBlockEvent::PacketTimeout { inner } => {
            chain_context.with_event(inner).handle(tx).await?
        },
        SupportedBlockEvent::TokenBucketUpdate { inner } => {
            chain_context.with_event(inner).handle(tx).await?
        },
        SupportedBlockEvent::WalletMutationEntry { inner } => {
            chain_context.with_event(inner).handle(tx).await?
        },
        SupportedBlockEvent::CreateWrappedToken { inner } => {
            chain_context.with_event(inner).handle(tx).await?
        },
    })
}
