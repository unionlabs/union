use sqlx::{Postgres, Transaction};

use crate::{indexer::api::BlockHeight, postgres::schedule_replication_reset};

pub async fn delete_eth_log(
    tx: &mut Transaction<'_, Postgres>,
    chain_id: i32,
    height: BlockHeight,
) -> sqlx::Result<()> {
    sqlx::query!(
        "
        DELETE FROM v0.logs WHERE chain_id = $1 AND height = $2
        ",
        chain_id,
        height as i32
    )
    .execute(tx.as_mut())
    .await?;

    schedule_replication_reset(tx, chain_id, height as i64, "block reorg (delete)").await?;

    Ok(())
}
