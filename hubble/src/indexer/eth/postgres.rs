use alloy::rpc::types::Block;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use time::OffsetDateTime;

use crate::{
    indexer::{
        api::{BlockHash, BlockHeight, BlockRange},
        eth::{
            block_handle::{BlockInsert, TransactionInsert},
            fetcher_client::{AddressFilter, TransactionFilter},
        },
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
    pub header: Block,
}

impl From<BlockInsert> for PgLog {
    fn from(block: BlockInsert) -> Self {
        PgLog {
            chain_id: block.chain_id,
            block_hash: block.hash.clone(),
            height: block.height as u64,
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
        Vec<i32>,
        Vec<OffsetDateTime>,
    ) = logs
        .into_iter()
        .map(|l| {
            (
                l.chain_id.db,
                l.block_hash,
                serde_json::to_value(&l.data).expect("data should be json serializable"),
                l.height as i32,
                l.time,
            )
        })
        .multiunzip();

    match mode {
        InsertMode::Insert => {
            sqlx::query!("
                INSERT INTO v0.logs (chain_id, block_hash, data, height, time)
                SELECT unnest($1::int[]), unnest($2::text[]), unnest($3::jsonb[]), unnest($4::int[]), unnest($5::timestamptz[])
                ", &chain_ids, &hashes, &data, &height, &time)
            .execute(tx.as_mut()).await?;
        }
        InsertMode::Upsert => {
            sqlx::query!("
                INSERT INTO v0.logs (chain_id, block_hash, data, height, time)
                SELECT unnest($1::int[]), unnest($2::text[]), unnest($3::jsonb[]), unnest($4::int[]), unnest($5::timestamptz[])
                ON CONFLICT (chain_id, height) DO 
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

                schedule_replication_reset(
                    tx,
                    *chain_id,
                    (*min_height).into(),
                    "block reorg (upsert)",
                )
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

pub struct UnmappedClient {
    pub transaction_hash: String,
    pub height: BlockHeight,
    pub client_id: Option<String>,
}

pub async fn unmapped_clients(
    pg_pool: &PgPool,
    internal_chain_id: i32,
) -> sqlx::Result<Vec<UnmappedClient>> {
    let result = sqlx::query!(
        r#"
        SELECT cc.transaction_hash, cc.height, cc.client_id
        FROM   v1_evm.client_created cc
        LEFT JOIN v0.clients cl ON cc.internal_chain_id = cl.chain_id AND cc.client_id = cl.client_id
        WHERE  cc.internal_chain_id = $1
        AND    cl.chain_id IS NULL
        "#,
        internal_chain_id
    )
    .fetch_all(pg_pool)
    .await?
    .into_iter()
    .map(|record| UnmappedClient {
        transaction_hash: record.transaction_hash.expect("client-created event to have transaction hash"),
        height: record.height.expect("client-created event to have a height") as u64,
        client_id: record.client_id,
    })
    .collect_vec();

    Ok(result)
}

pub async fn transaction_filter(
    pg_pool: &PgPool,
    internal_chain_id: i32,
) -> sqlx::Result<TransactionFilter> {
    let address_filters = sqlx::query!(
        r#"
        SELECT start_height, end_height, address
        FROM   v1_evm.contracts
        WHERE  internal_chain_id = $1
        "#,
        internal_chain_id
    )
    .fetch_all(pg_pool)
    .await?
    .into_iter()
    .map(|record| AddressFilter {
        block_range: BlockRange {
            start_inclusive: record.start_height as u64,
            end_exclusive: record.end_height as u64,
        },
        address: record.address.parse().expect("address to be valid"),
    })
    .collect_vec();

    Ok(TransactionFilter { address_filters })
}
