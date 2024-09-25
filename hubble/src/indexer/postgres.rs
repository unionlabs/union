use sqlx::Postgres;
use time::OffsetDateTime;

use crate::indexer::api::{BlockHash, BlockHeight, BlockRange, IndexerId};

pub async fn get_current_height(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: IndexerId,
) -> sqlx::Result<Option<BlockHeight>> {
    let record = sqlx::query!(
        "
        SELECT height
        FROM hubble.indexer_status
        WHERE indexer_id = $1
        LIMIT 1
        ",
        indexer_id,
    )
    .fetch_optional(tx.as_mut())
    .await?;

    Ok(record.map(|h| h.height as BlockHeight))
}

pub async fn update_current_height(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: IndexerId,
    height: BlockHeight,
    timestamp: OffsetDateTime,
) -> sqlx::Result<()> {
    sqlx::query!(
        "
        INSERT INTO hubble.indexer_status (indexer_id, height, timestamp)
        VALUES ($1, $2, $3)
        ON CONFLICT (indexer_id) DO 
        UPDATE SET
            height = excluded.height,
            timestamp = excluded.timestamp
        ",
        indexer_id,
        height as i64,
        timestamp,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}

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
        (Some(min), Some(max)) => Some((min as BlockHeight..max as BlockHeight + 1).into()),
        (None, None) => None,
        _ => unreachable!("expecting min_height and max_height to be either null or available"),
    })
}

pub async fn get_next_block_to_refresh(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: IndexerId,
    consensus_height: BlockHeight,
) -> sqlx::Result<Option<BlockHeight>> {
    let record = sqlx::query!(
        "
        SELECT height height
        FROM hubble.block_status
        WHERE indexer_id = $1 AND height > $2
        ORDER BY updated_at
        ",
        indexer_id,
        consensus_height as i64,
    )
    .fetch_optional(tx.as_mut())
    .await?;

    Ok(record.map(|r| r.height as BlockHeight))
}

pub async fn get_block_range_to_fix(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: IndexerId,
) -> sqlx::Result<Option<BlockRange>> {
    let record = sqlx::query!(
        "
        SELECT start_height, max(end_height) as end_height
        FROM hubble.block_fix
        WHERE start_height = (
            SELECT min(start_height)
            FROM hubble.block_fix
            WHERE indexer_id = $1
        )
        GROUP BY start_height
        ",
        indexer_id,
    )
    .fetch_optional(tx.as_mut())
    .await?;

    Ok(record.map(|r| {
        (r.start_height as BlockHeight
            ..r.end_height.expect("end_height column value") as BlockHeight)
            .into()
    }))
}

pub async fn update_block_range_to_fix(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: IndexerId,
    range: BlockRange,
) -> sqlx::Result<()> {
    // update start of ranges
    sqlx::query!(
        "
        UPDATE hubble.block_fix
        SET start_height = $3
        WHERE indexer_id = $1
        AND   start_height = $2
        ",
        indexer_id,
        range.start_inclusive as i64,
        range.end_exclusive as i64,
    )
    .execute(tx.as_mut())
    .await?;

    // remove empty ranges
    sqlx::query!(
        "
        DELETE FROM hubble.block_fix
        WHERE indexer_id = $1
        AND   start_height = $2
        AND   end_height <= $2
        ",
        indexer_id,
        range.end_exclusive as i64,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}

pub async fn delete_block_status(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: IndexerId,
    height: BlockHeight,
) -> sqlx::Result<Option<BlockHash>> {
    let record = sqlx::query!(
        "
        DELETE FROM hubble.block_status
        WHERE indexer_id = $1 AND height = $2
        RETURNING hash
        ",
        indexer_id,
        height as i64,
    )
    .fetch_optional(tx.as_mut())
    .await?;

    Ok(record.map(|r| r.hash as BlockHash))
}

pub async fn get_block_status_hash(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: IndexerId,
    height: BlockHeight,
) -> sqlx::Result<Option<BlockHash>> {
    let record = sqlx::query!(
        "
        SELECT hash FROM hubble.block_status
        WHERE indexer_id = $1 AND height = $2
        ",
        indexer_id,
        height as i64,
    )
    .fetch_optional(tx.as_mut())
    .await?;

    Ok(record.map(|r| r.hash as BlockHash))
}

pub async fn update_block_status(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: IndexerId,
    height: BlockHeight,
    hash: BlockHash,
    timestamp: OffsetDateTime,
) -> sqlx::Result<()> {
    sqlx::query!(
        "
        INSERT INTO hubble.block_status (indexer_id, height, hash, timestamp)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (indexer_id, height) DO 
        UPDATE SET
            hash = excluded.hash,
            timestamp = excluded.timestamp
        ",
        indexer_id,
        height as i64,
        hash,
        timestamp,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}
