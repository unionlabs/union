use std::time::Duration;

use sqlx::Postgres;
use time::OffsetDateTime;

use crate::indexer::{
    api::{BlockHash, BlockHeight, BlockRange, IndexerId},
    event::types::MessageHash,
};

pub async fn get_block_range_to_finalize(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: IndexerId,
) -> sqlx::Result<Option<BlockRange>> {
    let record = sqlx::query!(
        "
        SELECT MIN(height) min_height, MAX(height) max_height
        FROM hubble.block_status
        WHERE indexer_id = $1
        ",
        indexer_id,
    )
    .fetch_one(tx.as_mut())
    .await?;

    Ok(match (record.min_height, record.max_height) {
        (Some(min), Some(max)) => {
            let min_inclusive: BlockHeight = min.try_into().unwrap();
            let max_inclusive: BlockHeight = max.try_into().unwrap();
            let max_exclusive = max_inclusive + 1;
            Some((min_inclusive..max_exclusive).into())
        }
        (None, None) => None,
        _ => unreachable!("expecting min_height and max_height to be either null or available"),
    })
}

pub async fn get_next_block_to_monitor(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: IndexerId,
    consensus_height: BlockHeight,
    min_duration_between_monitor_checks: Duration,
) -> sqlx::Result<Option<BlockHeight>> {
    let consensus_height: i64 = consensus_height.try_into().unwrap();
    let record = sqlx::query!(
        "
        SELECT height height
        FROM hubble.block_status
        WHERE indexer_id = $1 AND height > $2
        AND updated_at < $3
        ORDER BY updated_at
        ",
        indexer_id,
        consensus_height,
        OffsetDateTime::now_utc() - min_duration_between_monitor_checks,
    )
    .fetch_optional(tx.as_mut())
    .await?;

    Ok(record.map(|r| r.height.try_into().unwrap()))
}

#[derive(sqlx::FromRow)]
pub struct BlockStatus {
    pub block_hash: BlockHash,
    pub message_hash: Option<MessageHash>,
}

pub async fn delete_block_status(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: IndexerId,
    height: BlockHeight,
) -> sqlx::Result<Option<BlockStatus>> {
    let height: i64 = height.try_into().unwrap();
    Ok(sqlx::query!(
        "
        DELETE FROM hubble.block_status
        WHERE indexer_id = $1 AND height = $2
        RETURNING hash as block_hash, message_hash
        ",
        indexer_id,
        height,
    )
    .fetch_optional(tx.as_mut())
    .await?
    .map(|record| BlockStatus {
        block_hash: record.block_hash,
        message_hash: record.message_hash.map(|message_hash| message_hash.into()),
    }))
}

pub async fn get_block_status_hash(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: IndexerId,
    height: BlockHeight,
) -> sqlx::Result<Option<BlockStatus>> {
    let height: i64 = height.try_into().unwrap();
    Ok(sqlx::query!(
        "
        SELECT hash as block_hash, message_hash
        FROM hubble.block_status
        WHERE indexer_id = $1 AND height = $2
        ",
        indexer_id,
        height,
    )
    .fetch_optional(tx.as_mut())
    .await?
    .map(|record| BlockStatus {
        block_hash: record.block_hash,
        message_hash: record.message_hash.map(|message_hash| message_hash.into()),
    }))
}

pub async fn update_block_status(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: IndexerId,
    height: BlockHeight,
    hash: BlockHash,
    timestamp: OffsetDateTime,
    message_hash: Option<MessageHash>,
) -> sqlx::Result<()> {
    let height: i64 = height.try_into().unwrap();

    sqlx::query!(
        "
        INSERT INTO hubble.block_status (indexer_id, height, hash, timestamp, message_hash)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (indexer_id, height) DO 
        UPDATE SET
            hash = excluded.hash,
            timestamp = excluded.timestamp,
            message_hash = 
                CASE
                    WHEN block_status.message_hash IS NOT NULL AND excluded.message_hash IS NULL THEN block_status.message_hash
                    ELSE excluded.message_hash
                END
        ",
        indexer_id,
        height,
        hash,
        timestamp,
        message_hash.map(|message_hash|Into::<Vec<u8>>::into(message_hash)),
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}
