use std::fmt::Display;

use itertools::Itertools;
use sqlx::Postgres;
use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::types::{BlockHeight, Range, UniversalChainId},
    record::PgValue,
};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct BlockEnrichStatus {
    pub id: i64,
    pub range: Range,
}

impl Display for BlockEnrichStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.range)
    }
}

pub async fn block_enrich_status(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    universal_chain_id: &UniversalChainId,
) -> Result<Option<BlockEnrichStatus>, IndexerError> {
    trace!("block_enrich_status");

    let statuses = sqlx::query!(
        "
            SELECT 
                id, 
                start_height, 
                end_height
            FROM hubble.block_enrich
            WHERE universal_chain_id = $1
            FOR UPDATE SKIP LOCKED
        ",
        universal_chain_id.pg_value()?,
    )
    .fetch_all(tx.as_mut())
    .await?
    .into_iter()
    .map(|record| {
        Ok(BlockEnrichStatus {
            id: record.id,
            range: Range::new_from_start_inclusive_end_exclusive(
                &record.start_height.try_into()?,
                &record.end_height.try_into()?,
            ),
        })
    })
    .collect::<Result<Vec<BlockEnrichStatus>, IndexerError>>()?;

    Ok(match statuses.as_slice() {
        [] => None,
        [single] => Some(single.clone()),
        rest => {
            // Sort statuses by start height to make merging easier
            let mut sorted_statuses = rest.to_vec();
            sorted_statuses.sort_by_key(|s| s.range.start_inclusive);

            // Check if there are any overlapping or adjacent ranges
            let mut has_overlaps = false;
            for i in 0..sorted_statuses.len() - 1 {
                if sorted_statuses[i].range.end_exclusive
                    >= sorted_statuses[i + 1].range.start_inclusive
                {
                    trace!(
                        "block_enrich_status - detected overlap: {} - {} => merge required",
                        sorted_statuses[i],
                        sorted_statuses[i + 1]
                    );

                    has_overlaps = true;
                    break;
                }
            }

            if !has_overlaps {
                trace!(
                    "block_enrich_status - no overlaps in {}, returning first {}",
                    sorted_statuses
                        .iter()
                        .map(|status| status.to_string())
                        .join(", "),
                    sorted_statuses[0],
                );
                // No overlaps or adjacent ranges, return the first one
                return Ok(Some(sorted_statuses[0].clone()));
            }

            trace!(
                "block_enrich_status - merging => {}",
                sorted_statuses
                    .iter()
                    .map(|status| status.to_string())
                    .join(", ")
            );

            // Find overlapping and adjacent ranges and merge them

            // maps a range to existing range ids that were used to construct it
            // if there is just one id, then the range is unchanged, but if there
            // are more, then all ranges will be deleted an a new range will be
            // inserted.
            let mut result_ranges = Vec::new();

            // range that is currently being constructed
            let mut current_range = sorted_statuses[0].range.clone();
            // range ids that were used to construct current_range
            let mut current_ids = vec![sorted_statuses[0].id];

            for status in &sorted_statuses[1..] {
                trace!("block_enrich_status - {current_range} - {status} - processing");

                // Check if ranges overlap or are adjacent
                if current_range.end_exclusive >= status.range.start_inclusive {
                    trace!("block_enrich_status - {current_range} - {status} - absorb range");

                    // Merge ranges - extend current range to include this one
                    current_range.end_exclusive =
                        current_range.end_exclusive.max(status.range.end_exclusive);
                    current_ids.push(status.id);
                } else {
                    trace!("block_enrich_status - {current_range} - {status} - new range");

                    // No overlap/adjacency - save current range and start new one
                    result_ranges.push((current_range.clone(), current_ids.clone()));
                    current_range = status.range.clone();
                    current_ids = vec![status.id];
                }
            }
            // Add the last range
            result_ranges.push((current_range, current_ids));

            // Process ranges: delete and recreate merged ones, keep single ones
            let mut new_records = Vec::new();
            for (range, ids) in result_ranges {
                trace!(
                    "block_enrich_status - {range} (ids: {})",
                    ids.iter().join(", ")
                );

                if ids.len() > 1 {
                    // This is a merged range - delete old records and insert new one
                    trace!(
                        "block_enrich_status - {range} (ids: {}) - merging",
                        ids.iter().join(", ")
                    );

                    for id in &ids {
                        sqlx::query!("DELETE FROM hubble.block_enrich WHERE id = $1", id)
                            .execute(tx.as_mut())
                            .await?;
                    }

                    let record = sqlx::query!(
                        "INSERT INTO hubble.block_enrich (universal_chain_id, start_height, end_height, reason) 
                         VALUES ($1, $2, $3, $4) 
                         RETURNING id, start_height, end_height",
                        universal_chain_id.pg_value()?,
                        i64::try_from(range.start_inclusive).map_err(|_| IndexerError::InternalCannotMapFromDatabaseDomain("start_height".to_string(), range.start_inclusive.to_string()))?,
                        i64::try_from(range.end_exclusive).map_err(|_| IndexerError::InternalCannotMapFromDatabaseDomain("end_height".to_string(), range.end_exclusive.to_string()))?,
                        format!("merging: {}", ids.iter().map(|id|id.to_string()).join(","))
                    )
                    .fetch_one(tx.as_mut())
                    .await?;

                    new_records.push(BlockEnrichStatus {
                        id: record.id,
                        range: Range::new_from_start_inclusive_end_exclusive(
                            &record.start_height.try_into()?,
                            &record.end_height.try_into()?,
                        ),
                    });
                } else {
                    trace!(
                        "block_enrich_status - {range} (ids: {}) - keeping",
                        ids.iter().join(", ")
                    );

                    // Single range that didn't merge with anything - keep original
                    let original_status = sorted_statuses.iter().find(|s| s.id == ids[0]).unwrap();
                    new_records.push(original_status.clone());
                }
            }

            // Return the first (lowest height) record
            new_records
                .into_iter()
                .min_by_key(|r| r.range.start_inclusive)
        }
    })
}

pub async fn next_height_to_enrich(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    universal_chain_id: &UniversalChainId,
    block_enrich_status: &BlockEnrichStatus,
) -> Result<Option<BlockHeight>, IndexerError> {
    trace!("next_height_to_enrich: {universal_chain_id} in {block_enrich_status}");

    sqlx::query!(
        "
            SELECT height
            FROM v2_sync.packet_send_sync
            WHERE internal_chain_id = (SELECT id FROM config.chains WHERE family || '.' || chain_id = $1)
            AND height >= $2 AND height < $3
            ORDER BY height
            LIMIT 1
        ",
        universal_chain_id.pg_value()?,
        BlockHeight::from(block_enrich_status.range.start_inclusive).pg_value()?,
        BlockHeight::from(block_enrich_status.range.end_exclusive).pg_value()?,
    )
    .fetch_optional(tx.as_mut())
    .await?
    .map(|record| {
        record.height.ok_or_else(|| IndexerError::InternalCannotMapFromDatabaseDomain("height".to_string(), "expecting non-null height in 'v2_sync.packet_send_sync'".to_string()))?.try_into()
    })
    .transpose()
}

pub async fn block_enrich_delete(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    block_enrich_status: &BlockEnrichStatus,
) -> Result<(), IndexerError> {
    trace!("block_enrich_delete: {block_enrich_status}");
    sqlx::query!(
        "DELETE FROM hubble.block_enrich WHERE id = $1",
        block_enrich_status.id
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}

pub async fn block_enrich_update(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    block_enrich_status: &BlockEnrichStatus,
    new_start_height: &BlockHeight,
) -> Result<(), IndexerError> {
    trace!("block_enrich_delete: {block_enrich_status}");
    sqlx::query!(
        "
        UPDATE hubble.block_enrich 
        SET start_height = $2 
        WHERE id = $1",
        block_enrich_status.id,
        new_start_height.pg_value()?,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}
