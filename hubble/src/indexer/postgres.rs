use std::{collections::HashMap, time::Duration};

use async_nats::HeaderMap;
use bytes::Bytes;
use sqlx::Postgres;
use time::OffsetDateTime;

use crate::indexer::{
    api::{BlockHash, BlockHeight, BlockRange, IndexerId},
    nats::Message,
};

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
        let start_inclusive: BlockHeight = r.start_height.try_into().unwrap();
        let end_exclusive: BlockHeight = r
            .end_height
            .expect("end_height column value")
            .try_into()
            .unwrap();
        (start_inclusive..end_exclusive).into()
    }))
}

pub async fn update_block_range_to_fix(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: IndexerId,
    range: BlockRange,
) -> sqlx::Result<()> {
    let start_inclusive: i64 = range.start_inclusive.try_into().unwrap();
    let end_exclusive: i64 = range.end_exclusive.try_into().unwrap();
    // update start of ranges
    sqlx::query!(
        "
        UPDATE hubble.block_fix
        SET start_height = $3
        WHERE indexer_id = $1
        AND   start_height = $2
        ",
        indexer_id,
        start_inclusive,
        end_exclusive,
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
        end_exclusive,
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
    let height: i64 = height.try_into().unwrap();
    let record = sqlx::query!(
        "
        DELETE FROM hubble.block_status
        WHERE indexer_id = $1 AND height = $2
        RETURNING hash
        ",
        indexer_id,
        height,
    )
    .fetch_optional(tx.as_mut())
    .await?;

    Ok(record.map(|r| r.hash))
}

pub async fn get_block_status_hash(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: IndexerId,
    height: BlockHeight,
) -> sqlx::Result<Option<BlockHash>> {
    let height: i64 = height.try_into().unwrap();
    let record = sqlx::query!(
        "
        SELECT hash FROM hubble.block_status
        WHERE indexer_id = $1 AND height = $2
        ",
        indexer_id,
        height,
    )
    .fetch_optional(tx.as_mut())
    .await?;

    Ok(record.map(|r| r.hash))
}

pub async fn update_block_status(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: IndexerId,
    height: BlockHeight,
    hash: BlockHash,
    timestamp: OffsetDateTime,
) -> sqlx::Result<()> {
    let height: i64 = height.try_into().unwrap();
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
        height,
        hash,
        timestamp,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}

pub async fn schedule(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    subject: &str,
    data: bytes::Bytes,
) -> sqlx::Result<i64> {
    let record = sqlx::query!(
        "
        INSERT INTO nats.out(subject, data)
        VALUES ($1, $2)
        RETURNING id
        ",
        subject,
        data.as_ref(),
    )
    .fetch_one(tx.as_mut())
    .await?;

    Ok(record.id)
}

pub async fn next_to_publish(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    subject: &str,
    batch_size: usize,
) -> sqlx::Result<Vec<Message>> {
    let raw_rows = sqlx::query!(
        r#"
        WITH to_publish AS (
            SELECT id
            FROM nats.out
            WHERE subject = $1
            ORDER BY id
            FOR UPDATE SKIP LOCKED
            LIMIT $2
        ),
        deleted AS (
            DELETE FROM nats.out
            USING to_publish
            WHERE nats.out.id = to_publish.id
            RETURNING nats.out.id, nats.out.subject, nats.out.headers, nats.out.data
        )
        SELECT id, subject, headers, data
        FROM deleted
        ORDER BY id;
        "#,
        subject,
        i64::try_from(batch_size).expect("batch-size < i64 max"),
    )
    .fetch_all(tx.as_mut())
    .await?;

    let result: Vec<Message> = raw_rows
        .into_iter()
        .map(|row| {
            let id: i64 = row.id;
            let subject: String = row.subject;
            let data: Bytes = row.data.into();

            let raw_headers: HashMap<String, Vec<String>> = serde_json::from_value(row.headers)
                .map_err(|e| sqlx::Error::ColumnDecode {
                    index: "headers".into(),
                    source: Box::new(e),
                })?;

            let mut headers = HeaderMap::new();
            for (key, values) in raw_headers {
                for value in values {
                    headers.insert(key.clone(), value);
                }
            }

            Ok(Message::new(id, subject, headers, data))
        })
        .collect::<Result<_, sqlx::Error>>()?;

    Ok(result)
}
