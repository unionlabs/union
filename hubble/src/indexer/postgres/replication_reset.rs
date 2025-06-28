use sqlx::Postgres;

use crate::indexer::{
    api::IndexerError,
    event::types::BlockHeight,
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
