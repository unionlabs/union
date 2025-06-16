use std::fmt::Display;

use itertools::Itertools;
use sqlx::Postgres;
use tracing::trace;

use crate::indexer::api::{BlockHeight, BlockRange, IndexerId};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct BlockFixStatus {
    pub id: i64,
    pub range: BlockRange,
    pub next: BlockHeight,
}

impl Display for BlockFixStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{} (id: {})", self.range, self.next, self.id)
    }
}

pub async fn get_block_fix_status(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: &IndexerId,
) -> sqlx::Result<Option<BlockFixStatus>> {
    trace!("get_block_fix_status: {indexer_id}");

    if !has_block_fix(tx, indexer_id).await? {
        trace!("get_block_fix_status: {indexer_id} => no fixer");

        return Ok(None); // no fixer for this indexer
    };

    let active = get_block_fix_status_active_with_counts(tx, indexer_id).await?;
    let result = match active.len() {
        1 => {
            // one active fixer. check if it's still with the lowest start and
            // use it. if other fixers are about to start, merge them

            let active = active.into_iter().next().expect("one");
            trace!(
                "get_block_fix_status: {indexer_id} - single: {} (before: {}, at: {})",
                active.block_fix_status,
                active.count_others_start_before_next,
                active.count_others_start_at_next
            );

            if active.count_others_start_before_next != 0 {
                trace!("get_block_fix_status: {indexer_id} - others before next");
                // fixers with start before active next found: reset active fixers and start over
                get_block_fix_status_inactive(tx, indexer_id).await?
            } else if active.count_others_start_at_next != 0 {
                trace!("get_block_fix_status: {indexer_id} - others at next");
                // fixers with start at active next found: merge them into the active fixer
                Some(merge_into_active_block_fix_status(tx, active.block_fix_status).await?)
            } else {
                trace!("get_block_fix_status: {indexer_id} - no merge required");
                // single active fixer, no merge required
                Some(active.block_fix_status)
            }
        }
        count => {
            trace!("get_block_fix_status: {indexer_id} - fetch inactive (count: {count})");
            // zero or more than one active fixer => reset all fixers (to be sure)
            // and find the one with minimal start height
            get_block_fix_status_inactive(tx, indexer_id).await?
        }
    };

    trace!(
        "get_block_fix_status: {indexer_id} => {}",
        match &result {
            Some(val) => val.to_string(),
            None => "-".to_string(),
        }
    );

    Ok(result)
}

async fn get_block_fix_status_inactive(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: &IndexerId,
) -> sqlx::Result<Option<BlockFixStatus>> {
    trace!("get_block_fix_status_inactive: {indexer_id}");

    reset_active_block_fix_status(tx, indexer_id).await?;

    get_block_fix_status_at_start(tx, indexer_id).await
}

async fn has_block_fix(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: &IndexerId,
) -> sqlx::Result<bool> {
    trace!("has_block_fix: {indexer_id}");

    Ok(sqlx::query!(
        "
            SELECT COUNT(*) as count
            FROM hubble.block_fix
            WHERE indexer_id = $1
            ",
        indexer_id,
    )
    .fetch_one(tx.as_mut())
    .await?
    .count
    .unwrap_or_default()
        > 0)
}

struct BlockFixStatusWithCounts {
    block_fix_status: BlockFixStatus,
    count_others_start_before_next: u64,
    count_others_start_at_next: u64,
}

async fn get_block_fix_status_active_with_counts(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: &IndexerId,
) -> sqlx::Result<Vec<BlockFixStatusWithCounts>> {
    trace!("get_block_fix_status_active_with_counts: {indexer_id}");

    Ok(sqlx::query!(
        "
            SELECT 
                bf.id,
                bf.start_height,
                bf.next_height,
                bf.end_height,
                (SELECT COUNT(*) FROM hubble.block_fix c WHERE indexer_id = $1 AND bf.id <> c.id AND c.start_height < bf.next_height) AS count_others_start_before_next,
                (SELECT COUNT(*) FROM hubble.block_fix c WHERE indexer_id = $1 AND bf.id <> c.id AND c.start_height = bf.next_height) AS count_others_start_at_next
            FROM hubble.block_fix bf
            WHERE indexer_id = $1
            AND bf.next_height > bf.start_height
            FOR UPDATE
            ",
        indexer_id,
    )
    .fetch_all(tx.as_mut())
    .await?
    .into_iter()
    .map(|r| {
        BlockFixStatusWithCounts {
            block_fix_status: BlockFixStatus {
                id: r.id,

                range: BlockRange {
                    start_inclusive: BlockHeight::try_from(r.start_height).expect("start_height fits"),
                    end_exclusive: BlockHeight::try_from(r.end_height).expect("end_height fits")
                },
                next: BlockHeight::try_from(r.next_height).expect("next_height fits") },
            count_others_start_before_next: BlockHeight::try_from(r.count_others_start_before_next.expect("count")).expect("count_others_start_before_next fits"),
            count_others_start_at_next: BlockHeight::try_from(r.count_others_start_at_next.expect("count")).expect("count_others_start_at_next fits"),
        }
    })
    .collect_vec())
}

async fn reset_active_block_fix_status(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: &IndexerId,
) -> sqlx::Result<()> {
    trace!("reset_active_block_fix_status: {indexer_id}");

    sqlx::query!(
        "
            UPDATE hubble.block_fix
            SET next_height = start_height
            WHERE indexer_id = $1
            ",
        indexer_id,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}

async fn get_block_fix_status_at_start(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: &IndexerId,
) -> sqlx::Result<Option<BlockFixStatus>> {
    trace!("get_block_fix_status_at_start: {indexer_id}");

    let block_fix_statuses = sqlx::query!(
        "
            SELECT 
                bf.id, 
                bf.start_height, 
                bf.next_height, 
                bf.end_height
            FROM hubble.block_fix bf
            WHERE indexer_id = $1
            AND bf.start_height = (SELECT MIN(start_height) FROM hubble.block_fix WHERE indexer_id = $1)
            FOR UPDATE
            ",
        indexer_id,
    )
    .fetch_all(tx.as_mut())
    .await?
    .into_iter()
    .map(|r| {
        BlockFixStatus {
            id: r.id,
            range: BlockRange {
                start_inclusive: BlockHeight::try_from(r.start_height).expect("start_height fits"),
                end_exclusive: BlockHeight::try_from(r.end_height).expect("end_height fits")
            },
            next: BlockHeight::try_from(r.next_height).expect("next_height fits")
        }
    })
    .collect_vec();

    trace!(
        "get_block_fix_status_at_start: found ids: {}",
        block_fix_statuses
            .iter()
            .map(|b| b.id.to_string())
            .join(", ")
    );

    Ok(match block_fix_statuses.len() {
        0 => None,
        1 => Some(block_fix_statuses.into_iter().next().expect("one")),
        _ => Some(merge_inactive_block_fix_status(tx, indexer_id, &block_fix_statuses).await?),
    })
}

async fn merge_inactive_block_fix_status(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: &IndexerId,
    block_fix_statuses: &[BlockFixStatus],
) -> sqlx::Result<BlockFixStatus> {
    trace!(
        "merge_inactive_block_fix_status: ids: {}",
        block_fix_statuses
            .iter()
            .map(|b| b.id.to_string())
            .join(", ")
    );

    let start_height = block_fix_statuses
        .iter()
        .map(|b| b.range.start_inclusive)
        .min()
        .expect("one");
    let end_height = block_fix_statuses
        .iter()
        .map(|b| b.range.end_exclusive)
        .max()
        .expect("one");

    sqlx::query!(
        "
        DELETE FROM hubble.block_fix WHERE ID = ANY($1)
        ",
        &block_fix_statuses.iter().map(|b| b.id).collect_vec(),
    )
    .execute(tx.as_mut())
    .await?;

    let id = sqlx::query!(
        "
        INSERT INTO hubble.block_fix(indexer_id, start_height, next_height, end_height)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        ",
        indexer_id,
        i64::try_from(start_height).expect("start_height fits"),
        i64::try_from(start_height).expect("start_height fits"), // next == start
        i64::try_from(end_height).expect("end_height fits"),
    )
    .fetch_one(tx.as_mut())
    .await?
    .id;

    Ok(BlockFixStatus {
        id,
        range: BlockRange {
            start_inclusive: start_height,
            end_exclusive: end_height,
        },
        next: start_height,
    })
}

async fn merge_into_active_block_fix_status(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    active_block_fix_status: BlockFixStatus,
) -> sqlx::Result<BlockFixStatus> {
    trace!("merge_into_active_block_fix_status: {active_block_fix_status}");

    // update active end_height to the maximum height of inactive records with start at active next
    let end_height: BlockHeight = sqlx::query!(
        "
        UPDATE hubble.block_fix
        SET end_height = GREATEST(
            -- end height of the current active record
            end_height, 
            -- maximum end height of inactive records that start at next height of active record
            (
                SELECT MAX(inactive_bf.end_height)
                FROM hubble.block_fix active_bf
                JOIN hubble.block_fix inactive_bf
                    ON active_bf.indexer_id = inactive_bf.indexer_id          -- same chain
                    AND active_bf.next_height = inactive_bf.start_height  -- start at active next
                WHERE active_bf.id = $1                                   -- current record
            )
        )
        WHERE id = $1
        RETURNING end_height
        ",
        active_block_fix_status.id,
    )
    .fetch_one(tx.as_mut())
    .await?
    .end_height
    .try_into()
    .expect("end height fits");

    trace!("merge_into_active_block_fix_status: {active_block_fix_status} - new end height: {end_height}");

    // delete inactive records with start at active next
    sqlx::query!(
        "
        DELETE FROM hubble.block_fix
        WHERE id IN 
            (
                SELECT inactive_bf.id
                FROM hubble.block_fix active_bf
                JOIN hubble.block_fix inactive_bf
                    ON active_bf.indexer_id = inactive_bf.indexer_id     -- same chain
                    AND active_bf.next_height = inactive_bf.start_height -- start at active next
                WHERE active_bf.id = $1                                  -- current record
            )
        ",
        &active_block_fix_status.id,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(BlockFixStatus {
        id: active_block_fix_status.id,
        range: BlockRange {
            start_inclusive: active_block_fix_status.range.start_inclusive,
            end_exclusive: end_height,
        },
        next: active_block_fix_status.next,
    })
}

pub async fn delete_block_range_to_fix(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    block_fix_status: &BlockFixStatus,
) -> sqlx::Result<()> {
    trace!("delete_block_range_to_fix: {block_fix_status}");

    sqlx::query!(
        "
        DELETE FROM hubble.block_fix
        WHERE id = $1
        ",
        block_fix_status.id,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}

pub async fn update_block_range_to_fix_start_and_next(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    block_fix_status: &BlockFixStatus,
    new_start_and_next: BlockHeight,
) -> sqlx::Result<()> {
    trace!("update_block_range_to_fix_start_and_next: {block_fix_status} - {new_start_and_next}");

    let new_start_and_next: i64 = new_start_and_next.try_into().unwrap();

    sqlx::query!(
        "
        UPDATE hubble.block_fix
        SET start_height = $2
          , next_height = $2
        WHERE id = $1
        ",
        block_fix_status.id,
        new_start_and_next,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}

pub async fn update_block_range_to_fix_next(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    block_fix_status: &BlockFixStatus,
    new_next: BlockHeight,
) -> sqlx::Result<()> {
    trace!("update_block_range_to_fix_next: {block_fix_status} - {new_next}");

    let new_next: i64 = new_next.try_into().unwrap();

    sqlx::query!(
        "
        UPDATE hubble.block_fix
        SET next_height = $2
        WHERE id = $1
        ",
        block_fix_status.id,
        new_next,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}
