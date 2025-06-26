use sqlx::{Postgres, Transaction};
use tracing::{debug, trace};

use crate::indexer::{
    api::IndexerError,
    event::{supported::SupportedBlockEvent, types::BlockHeight},
    record::{
        channel_open_ack_record::ChannelOpenAckRecord,
        channel_open_confirm_record::ChannelOpenConfirmRecord,
        channel_open_init_record::ChannelOpenInitRecord,
        channel_open_try_record::ChannelOpenTryRecord,
        connection_open_ack_record::ConnectionOpenAckRecord,
        connection_open_confirm_record::ConnectionOpenConfirmRecord,
        connection_open_init_record::ConnectionOpenInitRecord,
        connection_open_try_record::ConnectionOpenTryRecord,
        create_client_record::CreateClientRecord,
        create_lens_client_record::CreateLensClientRecord, packet_ack_record::PacketAckRecord,
        packet_recv_record::PacketRecvRecord, packet_send_record::PacketSendRecord,
        packet_timeout_record::PacketTimeoutRecord,
        token_bucket_update_record::TokenBucketUpdateRecord,
        update_client_record::UpdateClientRecord,
        wallet_mutation_entry_record::WalletMutationEntryRecord, write_ack_record::WriteAckRecord,
        ChainContext, InternalChainId,
    },
};

pub async fn delete_event_data_at_height(
    tx: &mut Transaction<'_, Postgres>,
    internal_chain_id: InternalChainId,
    height: BlockHeight,
) -> Result<bool, IndexerError> {
    debug!("delete_event_data_at_height: {internal_chain_id}@{height}");
    let deleted = if has_event_data_at_height(tx, internal_chain_id, height).await? {
        debug!("delete_event_data_at_height: {internal_chain_id}@{height} => deleting");

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
            ),
            delete_evm_logs_decoded AS (
                DELETE FROM v2_evm.logs_decoded WHERE internal_chain_id = $1 AND height = $2
            )
            DELETE FROM v2_evm.logs WHERE internal_chain_id = $1 AND height = $2
            ",
            internal_chain_id.pg_value_integer()?,
            height.pg_value_bigint()?,
        )
        .execute(tx.as_mut())
        .await?;

        // would be nice to detect if we're forgetting a record type. maybe not required. depends
        // on the update strategy. Maybe we'll first fetch all existing records to see the impact
        // of deleting them. then we'll have references to all records, so we can delete them
        // one by one.
        ChannelOpenInitRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        ChannelOpenTryRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        ChannelOpenAckRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        ChannelOpenConfirmRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        ConnectionOpenInitRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        ConnectionOpenTryRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        ConnectionOpenAckRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        ConnectionOpenConfirmRecord::delete_by_chain_and_height(tx, internal_chain_id, height)
            .await?;
        CreateClientRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        CreateLensClientRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        UpdateClientRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        PacketSendRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        PacketRecvRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        WriteAckRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        PacketAckRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        PacketTimeoutRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        TokenBucketUpdateRecord::delete_by_chain_and_height(tx, internal_chain_id, height).await?;
        WalletMutationEntryRecord::delete_by_chain_and_height(tx, internal_chain_id, height)
            .await?;

        true
    } else {
        debug!("delete_event_data_at_height: {internal_chain_id}@{height} => nothing to delete");
        false
    };

    Ok(deleted)
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
        internal_chain_id.pg_value_integer()?,
        height.pg_value_bigint()?,
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
) -> Result<bool, IndexerError> {
    let mut any_data_changes = false;

    for block_event in block_events {
        let event_changed_data = handle_block_event(tx, chain_context, block_event).await?;
        any_data_changes |= event_changed_data
    }

    Ok(any_data_changes)
}

async fn handle_block_event(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    chain_context: &ChainContext,
    block_event: &SupportedBlockEvent,
) -> Result<bool, IndexerError> {
    trace!("handling: {block_event:?}");

    match block_event {
        SupportedBlockEvent::EthereumLog {
                        internal_chain_id,
                        block_hash,
                        data,
                        height,
                        time,
            } => sqlx::query!(
                "
            INSERT INTO v2_evm.logs (internal_chain_id, block_hash, data, height, time)
            VALUES ($1, $2, $3, $4, $5)
            ",
                internal_chain_id,
                block_hash,
                data,
                height.pg_value_bigint()?,
                time
            )
            .execute(tx.as_mut())
            .await
            .map(|_| ())
            .map_err(IndexerError::DatabaseError),
        SupportedBlockEvent::EthereumDecodedLog {
                internal_chain_id,
                block_hash,
                height,
                log_index,
                timestamp,
                transaction_hash,
                transaction_index,
                transaction_log_index,
                raw_log,
                log_to_jsonb,
            } => sqlx::query!(
                "
            INSERT INTO v2_evm.logs_decoded (
                internal_chain_id,
                block_hash,
                height,
                log_index,
                timestamp,
                transaction_hash,
                transaction_index,
                transaction_log_index,
                raw_log,
                log_to_jsonb)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ",
                internal_chain_id,
                block_hash,
                height.pg_value_bigint()?,
                log_index,
                timestamp,
                transaction_hash,
                transaction_index,
                transaction_log_index,
                raw_log,
                log_to_jsonb,
            )
            .execute(tx.as_mut())
            .await
            .map(|_| ())
            .map_err(IndexerError::DatabaseError),
        SupportedBlockEvent::TendermintBlock {
                internal_chain_id,
                hash,
                data,
                height,
                time,
            } => sqlx::query!(
                "
            INSERT INTO v2_cosmos.blocks (internal_chain_id, hash, data, height, time)
            VALUES ($1, $2, $3, $4, $5)
            ",
                internal_chain_id,
                hash,
                data,
                height.pg_value_bigint()?,
                time
            )
            .execute(tx.as_mut())
            .await
            .map(|_| ())
            .map_err(IndexerError::DatabaseError),
        SupportedBlockEvent::TendermintTransaction {
                internal_chain_id,
                block_hash,
                height,
                hash,
                data,
                index,
            } => sqlx::query!("
            INSERT INTO v2_cosmos.transactions (internal_chain_id, block_hash, height, hash, data, index) 
            VALUES ($1, $2, $3, $4, $5, $6)
            ",
                internal_chain_id, block_hash, height.pg_value_bigint()?, hash, data, index)
            .execute(tx.as_mut())
            .await
            .map(|_| ())
            .map_err(IndexerError::DatabaseError),
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
            } =>     sqlx::query!("
            INSERT INTO v2_cosmos.events (internal_chain_id, block_hash, height, transaction_hash, index, transaction_index, data, time, flow)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ",
                internal_chain_id,
                block_hash,
                height.pg_value_bigint()?,
                transaction_hash as _,
                index,
                transaction_index as _,
                data,
                time,
                flow)
            .execute(tx.as_mut())
            .await
            .map(|_| ())
            .map_err(IndexerError::DatabaseError),
        SupportedBlockEvent::ChannelOpenInit { inner } => chain_context.with_event(inner).handle(tx).await,
        SupportedBlockEvent::ChannelOpenTry { inner } => chain_context.with_event(inner).handle(tx).await,
        SupportedBlockEvent::ChannelOpenAck { inner } => chain_context.with_event(inner).handle(tx).await,
        SupportedBlockEvent::ChannelOpenConfirm { inner } => chain_context.with_event(inner).handle(tx).await,
        SupportedBlockEvent::ConnectionOpenInit { inner } => chain_context.with_event(inner).handle(tx).await,
        SupportedBlockEvent::ConnectionOpenTry { inner } => chain_context.with_event(inner).handle(tx).await,
        SupportedBlockEvent::ConnectionOpenAck { inner } => chain_context.with_event(inner).handle(tx).await,
        SupportedBlockEvent::ConnectionOpenConfirm { inner } => chain_context.with_event(inner).handle(tx).await,
        SupportedBlockEvent::CreateClient { inner } => chain_context.with_event(inner).handle(tx).await,
        SupportedBlockEvent::CreateLensClient { inner } => chain_context.with_event(inner).handle(tx).await,
        SupportedBlockEvent::UpdateClient { inner } => chain_context.with_event(inner).handle(tx).await,
        SupportedBlockEvent::PacketSend { inner } => chain_context.with_event(inner).handle(tx).await,
        SupportedBlockEvent::PacketRecv { inner } => chain_context.with_event(inner).handle(tx).await,
        SupportedBlockEvent::WriteAck { inner } => chain_context.with_event(inner).handle(tx).await,
        SupportedBlockEvent::PacketAck { inner } => chain_context.with_event(inner).handle(tx).await,
        SupportedBlockEvent::PacketTimeout { inner } => chain_context.with_event(inner).handle(tx).await,
        SupportedBlockEvent::TokenBucketUpdate { inner } => chain_context.with_event(inner).handle(tx).await,
        SupportedBlockEvent::WalletMutationEntry { inner } => chain_context.with_event(inner).handle(tx).await,
    }?;

    Ok(true)
}
