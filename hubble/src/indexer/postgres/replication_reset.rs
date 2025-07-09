use sqlx::Postgres;

use crate::indexer::{
    api::IndexerError,
    event::types::{BlockHeight, Range, UniversalChainId},
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
    universal_chain_id: &UniversalChainId,
    range: &Range,
    reason: &str,
) -> Result<(), IndexerError> {
    let start_inclusive: BlockHeight = range.start_inclusive.into();
    let end_exclusive: BlockHeight = range.end_exclusive.into();

    sqlx::query!(
        "
        INSERT INTO hubble.block_enrich (universal_chain_id, start_height, end_height, reason)
        VALUES ($1, $2, $3, $4);",
        universal_chain_id.pg_value()?,
        start_inclusive.pg_value()?,
        end_exclusive.pg_value()?,
        reason,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}
