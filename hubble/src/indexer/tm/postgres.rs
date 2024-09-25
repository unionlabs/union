use itertools::Itertools;
use sqlx::{PgPool, Postgres, Transaction};

use crate::{indexer::api::BlockHeight, postgres::schedule_replication_reset};

pub async fn delete_tm_block_transactions_events(
    tx: &mut Transaction<'_, Postgres>,
    chain_id: i32,
    height: BlockHeight,
) -> sqlx::Result<()> {
    sqlx::query!(
        "
        DELETE FROM v0.events WHERE chain_id = $1 AND height = $2
        ",
        chain_id,
        height as i32
    )
    .execute(tx.as_mut())
    .await?;

    sqlx::query!(
        "
        DELETE FROM v0.transactions WHERE chain_id = $1 AND height = $2
        ",
        chain_id,
        height as i32
    )
    .execute(tx.as_mut())
    .await?;

    sqlx::query!(
        "
        DELETE FROM v0.blocks WHERE chain_id = $1 AND height = $2
        ",
        chain_id,
        height as i32
    )
    .execute(tx.as_mut())
    .await?;

    schedule_replication_reset(tx, chain_id, height as i64, "block reorg (delete)").await?;

    Ok(())
}

pub async fn unmapped_client_ids(
    pg_pool: &PgPool,
    internal_chain_id: i32,
) -> sqlx::Result<Vec<String>> {
    let result = sqlx::query!(
        r#"
        SELECT    cc.client_id
        FROM      v1_cosmos.create_client cc
        LEFT JOIN v0.clients cl ON cc.internal_chain_id = cl.chain_id AND cc.client_id = cl.client_id
        WHERE     cc.internal_chain_id = $1
        AND       cl.chain_id IS NULL
        "#,
        internal_chain_id,
    )
    .fetch_all(pg_pool)
    .await?
    .into_iter()
    .map(|record| record.client_id.expect("each record to have a client_id"))
    .collect_vec();

    Ok(result)
}
