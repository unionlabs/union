use sqlx::{Postgres, Transaction};

use crate::indexer::{api::BlockHeight, tendermint::block_handle::ActiveContracts};

pub async fn active_contracts(
    tx: &mut Transaction<'_, Postgres>,
    internal_chain_id: i32,
    height: BlockHeight,
) -> sqlx::Result<ActiveContracts> {
    let height: i64 = height.try_into().unwrap();

    let mut result = ActiveContracts::new();

    sqlx::query!(
        r#"
        SELECT    address, flow
        FROM      v2_cosmos.contracts
        WHERE     internal_chain_id = $1
        AND       $2 between start_height and end_height
        "#,
        internal_chain_id,
        height,
    )
    .fetch_all(tx.as_mut())
    .await?
    .into_iter()
    .for_each(|record| {
        result.register(record.address, record.flow);
    });

    Ok(result)
}
