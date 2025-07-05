use sqlx::Postgres;

use crate::indexer::{
    api::{IndexerError, IndexerId},
    event::types::{BlockHeight, Range},
    record::{ChainContext, PgValue},
};

pub async fn schedule_replication_reset(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    chain_context: &ChainContext,
    height: BlockHeight,
    reason: &str,
) -> Result<(), IndexerError> {
    sqlx::query!(
        "CALL sync.replication_schedule_reset_chain($1, $2, $3);",
        chain_context.internal_chain_id.pg_value_numeric()?, // function should consume i32. leave it because the syncing will be removed
        height.pg_value()?,
        reason
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}

pub async fn schedule_enrich_reset(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    indexer_id: &IndexerId,
    range: &Range,
    reason: &str,
) -> Result<(), IndexerError> {
    let start_inclusive: BlockHeight = range.start_inclusive.into();
    let end_exclusive: BlockHeight = range.end_exclusive.into();

    sqlx::query!(
        "
        INSERT INTO hubble.block_enrich (indexer_id, start_height, end_height, reason)
        VALUES ($1, $2, $3, $4);",
        indexer_id,
        start_inclusive.pg_value()?,
        end_exclusive.pg_value()?,
        reason,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}
