use alloy::{network::AnyRpcBlock, primitives::Address};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Transaction};
use time::OffsetDateTime;

use crate::{
    indexer::{
        api::{BlockHash, BlockHeight},
        ethereum::block_handle::{BlockInsert, TransactionInsert},
    },
    postgres::{schedule_replication_reset, ChainId, InsertMode},
};

pub struct PgLog {
    pub chain_id: ChainId,
    pub block_hash: BlockHash,
    pub height: BlockHeight,
    pub time: OffsetDateTime,
    pub data: PgLogData,
}

#[derive(Serialize, Deserialize)]
pub struct PgLogData {
    pub transactions: Vec<TransactionInsert>,
    pub header: AnyRpcBlock,
}

impl From<BlockInsert> for PgLog {
    fn from(block: BlockInsert) -> Self {
        PgLog {
            chain_id: block.chain_id,
            block_hash: block.hash.clone(),
            height: block.height.try_into().unwrap(),
            time: block.time,
            data: PgLogData {
                header: block.header,
                transactions: block.transactions,
            },
        }
    }
}

pub async fn insert_batch_logs(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    logs: impl IntoIterator<Item = PgLog>,
    mode: InsertMode,
) -> sqlx::Result<()> {
    let (chain_ids, hashes, data, height, time): (
        Vec<i32>,
        Vec<String>,
        Vec<_>,
        Vec<i64>,
        Vec<OffsetDateTime>,
    ) = logs
        .into_iter()
        .map(|l| {
            let height: i64 = l.height.try_into().unwrap();

            (
                l.chain_id.db,
                l.block_hash,
                serde_json::to_value(&l.data).expect("data should be json serializable"),
                height,
                l.time,
            )
        })
        .multiunzip();

    match mode {
        InsertMode::Insert => {
            sqlx::query!("
                INSERT INTO v2_evm.logs (internal_chain_id, block_hash, data, height, time)
                SELECT unnest($1::int[]), unnest($2::text[]), unnest($3::jsonb[]), unnest($4::bigint[]), unnest($5::timestamptz[])
                ", &chain_ids, &hashes, &data, &height, &time)
            .execute(tx.as_mut()).await?;
        }
        InsertMode::Upsert => {
            sqlx::query!("
                INSERT INTO v2_evm.logs (internal_chain_id, block_hash, data, height, time)
                SELECT unnest($1::int[]), unnest($2::text[]), unnest($3::jsonb[]), unnest($4::bigint[]), unnest($5::timestamptz[])
                ON CONFLICT (internal_chain_id, height) DO 
                UPDATE SET
                    data = excluded.data,
                    block_hash = excluded.block_hash,
                    time = excluded.time
                ", &chain_ids, &hashes, &data, &height, &time)
            .execute(tx.as_mut()).await?;

            if let Some(chain_id) = chain_ids.first() {
                assert!(
                    chain_ids.iter().all(|&x| x == *chain_id),
                    "expecting all logs to originate from the same chain_id: {:?}",
                    chain_ids
                );
                let min_height = height.iter().min().expect("at least one height");

                schedule_replication_reset(tx, *chain_id, *min_height, "block reorg (upsert)")
                    .await?;
            }
        }
    };
    Ok(())
}

pub async fn delete_eth_log(
    tx: &mut Transaction<'_, Postgres>,
    chain_id: i32,
    height: BlockHeight,
) -> sqlx::Result<()> {
    let height: i64 = height.try_into().unwrap();
    sqlx::query!(
        "
        DELETE FROM v2_evm.logs WHERE internal_chain_id = $1 AND height = $2
        ",
        chain_id,
        height,
    )
    .execute(tx.as_mut())
    .await?;

    schedule_replication_reset(tx, chain_id, height, "block reorg (delete)").await?;

    Ok(())
}

pub async fn active_contracts(
    tx: &mut Transaction<'_, Postgres>,
    internal_chain_id: i32,
    height: BlockHeight,
) -> sqlx::Result<Vec<Address>> {
    let height: i64 = height.try_into().unwrap();

    let result = sqlx::query!(
        r#"
        SELECT    address
        FROM      v2_evm.contracts
        WHERE     internal_chain_id = $1
        AND       $2 between start_height and end_height
        "#,
        internal_chain_id,
        height,
    )
    .fetch_all(tx.as_mut())
    .await?
    .into_iter()
    .map(|record| record.address.parse().expect("address to be valid"))
    .collect();

    Ok(result)
}
