use sqlx::{Postgres, Transaction};
use tracing::debug;

use crate::indexer::{
    api::{BlockHeight, UniversalChainId},
    event::SupportedBlockEvent,
};

pub async fn delete_event_data_at_height(
    tx: &mut Transaction<'_, Postgres>,
    universal_chain_id: &UniversalChainId,
    height: BlockHeight,
) -> sqlx::Result<bool> {
    debug!("delete_event_data_at_height: {universal_chain_id}@{height}");
    let deleted = if has_event_data_at_height(tx, universal_chain_id, height).await? {
        debug!("delete_event_data_at_height: {universal_chain_id}@{height} => deleting");
        sqlx::query!(
            "
            WITH delete_cosmos_events AS (
                DELETE FROM v2_cosmos.events WHERE internal_chain_id = (SELECT id FROM config.chains c WHERE c.family || '.' || c.chain_id = $1) AND height = $2
            ),
            delete_cosmos_transactions AS (
                DELETE FROM v2_cosmos.transactions WHERE internal_chain_id = (SELECT id FROM config.chains c WHERE c.family || '.' || c.chain_id = $1) AND height = $2
            ),
            delete_cosmos_blocks AS (
                DELETE FROM v2_cosmos.blocks WHERE internal_chain_id = (SELECT id FROM config.chains c WHERE c.family || '.' || c.chain_id = $1) AND height = $2
            )
            DELETE FROM v2_evm.logs WHERE internal_chain_id = (SELECT id FROM config.chains c WHERE c.family || '.' || c.chain_id = $1) AND height = $2
            ",
            universal_chain_id,
            i64::try_from(height).expect("height fits"),
        )
        .execute(tx.as_mut())
        .await?;

        true
    } else {
        debug!("delete_event_data_at_height: {universal_chain_id}@{height} => nothing to delete");
        false
    };

    Ok(deleted)
}

async fn has_event_data_at_height(
    tx: &mut Transaction<'_, Postgres>,
    universal_chain_id: &UniversalChainId,
    height: BlockHeight,
) -> sqlx::Result<bool> {
    debug!("has_event_data_at_height: {universal_chain_id}@{height}");

    Ok(sqlx::query!(
        "
        SELECT 1 AS exists FROM v2_cosmos.events WHERE internal_chain_id = (SELECT id FROM config.chains c WHERE c.family || '.' || c.chain_id = $1) AND height = $2
        UNION ALL
        SELECT 1 AS exists FROM v2_cosmos.transactions WHERE internal_chain_id = (SELECT id FROM config.chains c WHERE c.family || '.' || c.chain_id = $1) AND height = $2
        UNION ALL
        SELECT 1 AS exists FROM v2_cosmos.blocks WHERE internal_chain_id = (SELECT id FROM config.chains c WHERE c.family || '.' || c.chain_id = $1) AND height = $2
        UNION ALL
        SELECT 1 AS exists FROM v2_evm.logs WHERE internal_chain_id = (SELECT id FROM config.chains c WHERE c.family || '.' || c.chain_id = $1) AND height = $2
        LIMIT 1
        ",
        universal_chain_id,
        i64::try_from(height).expect("height fits"),
    )
    .fetch_optional(tx.as_mut())
    .await?
    .is_some())
}

pub async fn handle_block_events(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    block_events: &[&SupportedBlockEvent],
) -> sqlx::Result<bool> {
    let mut change = false;

    for block_event in block_events {
        change = change || handle_block_event(tx, block_event).await?
    }

    Ok(change)
}

pub async fn handle_block_event(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    block_event: &SupportedBlockEvent,
) -> sqlx::Result<bool> {
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
            i64::try_from(*height).expect("height fits"),
            time
        )
        .execute(tx.as_mut()),
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
            i64::try_from(*height).expect("height fits"),
            time
        )
        .execute(tx.as_mut()),
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
            internal_chain_id, block_hash, i64::try_from(*height).expect("height fits"), hash, data, index)
        .execute(tx.as_mut()),
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
            i64::try_from(*height).expect("height fits"),
            transaction_hash as _,
            index,
            transaction_index as _,
            data,
            time,
            flow)
        .execute(tx.as_mut()),
    }
    .await?;

    Ok(true)
}

pub async fn max_event_height(
    tx: &mut Transaction<'_, Postgres>,
    universal_chain_id: &UniversalChainId,
) -> sqlx::Result<BlockHeight> {
    debug!("max_event_height: {universal_chain_id}");

    Ok(sqlx::query!(
        "
        SELECT GREATEST(
            (SELECT MAX(height) FROM v2_cosmos.events WHERE internal_chain_id = (SELECT id FROM config.chains c WHERE c.family || '.' || c.chain_id = $1)),
            (SELECT MAX(height) FROM v2_cosmos.transactions WHERE internal_chain_id = (SELECT id FROM config.chains c WHERE c.family || '.' || c.chain_id = $1)),
            (SELECT MAX(height) FROM v2_cosmos.blocks WHERE internal_chain_id = (SELECT id FROM config.chains c WHERE c.family || '.' || c.chain_id = $1)),
            (SELECT MAX(height) FROM v2_evm.logs WHERE internal_chain_id = (SELECT id FROM config.chains c WHERE c.family || '.' || c.chain_id = $1))
        ) AS max_height
         ",
        universal_chain_id,
    )
    .fetch_optional(tx.as_mut())
    .await?
    .map(|record| {
        BlockHeight::try_from(record.max_height.unwrap_or_default()).expect("height fits")
    }).unwrap_or_default())
}
