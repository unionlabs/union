use sqlx::Postgres;
use time::OffsetDateTime;

use crate::indexer::api::{BlockHeight, IndexerId};

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

    Ok(record.map(|h| h.height.try_into().unwrap()))
}

pub async fn update_current_height(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: IndexerId,
    height: BlockHeight,
    timestamp: OffsetDateTime,
) -> sqlx::Result<()> {
    let height: i64 = height.try_into().unwrap();

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
        height,
        timestamp,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}
