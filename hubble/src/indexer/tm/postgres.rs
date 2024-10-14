use itertools::Itertools;
use sqlx::{PgPool, Postgres, Transaction};
use time::OffsetDateTime;

use crate::{
    indexer::api::{BlockHash, BlockHeight},
    postgres::{schedule_replication_reset, ChainId},
};

type TransactionHash = String;

/// DTO corresponding to the v0.blocks table.
#[derive(Clone)]
pub struct PgBlock {
    pub chain_id: ChainId,
    pub hash: BlockHash,
    pub height: BlockHeight,
    pub time: OffsetDateTime,
    pub data: serde_json::Value,
}

/// DTO corresponding to the v0.transactions table.
#[derive(Clone)]
pub struct PgTransaction {
    pub chain_id: ChainId,
    pub block_hash: BlockHash,
    pub block_height: BlockHeight,
    #[allow(dead_code)]
    pub time: OffsetDateTime,
    pub data: serde_json::Value,
    pub hash: TransactionHash,
    pub index: i32,
}

/// DTO corresponding to the v0.events table.
#[derive(Clone)]
pub struct PgEvent {
    pub chain_id: ChainId,
    pub block_hash: BlockHash,
    pub block_height: BlockHeight,
    pub time: OffsetDateTime,
    pub data: serde_json::Value,
    pub transaction_hash: Option<TransactionHash>,
    pub transaction_index: Option<i32>,
    pub block_index: i32,
}

pub async fn insert_batch_blocks(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    blocks: impl IntoIterator<Item = PgBlock>,
) -> sqlx::Result<()> {
    let (chain_ids, hashes, data, height, time): (
        Vec<i32>,
        Vec<String>,
        Vec<_>,
        Vec<i32>,
        Vec<OffsetDateTime>,
    ) = blocks
        .into_iter()
        .map(|b| (b.chain_id.db, b.hash, b.data, b.height as i32, b.time))
        .multiunzip();

    sqlx::query!("
        INSERT INTO v0.blocks (chain_id, hash, data, height, time)
        SELECT unnest($1::int[]), unnest($2::text[]), unnest($3::jsonb[]), unnest($4::int[]), unnest($5::timestamptz[])
        ", &chain_ids, &hashes, &data, &height, &time)
    .execute(tx.as_mut()).await?;

    Ok(())
}

pub async fn insert_batch_transactions(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    transactions: impl IntoIterator<Item = PgTransaction>,
) -> sqlx::Result<()> {
    #![allow(clippy::type_complexity)]
    let (chain_ids, block_hashes, heights, hashes, data, indexes): (
        Vec<i32>,
        Vec<String>,
        Vec<i32>,
        Vec<String>,
        Vec<_>,
        Vec<i32>,
    ) = transactions
        .into_iter()
        .map(|t| {
            (
                t.chain_id.db,
                t.block_hash,
                t.block_height as i32,
                t.hash,
                t.data,
                t.index,
            )
        })
        .multiunzip();

    sqlx::query!("
        INSERT INTO v0.transactions (chain_id, block_hash, height, hash, data, index) 
        SELECT unnest($1::int[]), unnest($2::text[]), unnest($3::int[]), unnest($4::text[]), unnest($5::jsonb[]), unnest($6::int[])
        ", 
        &chain_ids, &block_hashes, &heights, &hashes, &data, &indexes)
    .execute(tx.as_mut()).await?;

    Ok(())
}

pub async fn insert_batch_events(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    events: impl IntoIterator<Item = PgEvent>,
) -> sqlx::Result<()> {
    #![allow(clippy::type_complexity)]
    let (
        chain_ids,
        block_hashes,
        heights,
        transaction_hashes,
        indexes,
        transaction_indexes,
        data,
        times,
    ): (
        Vec<i32>,
        Vec<String>,
        Vec<i32>,
        Vec<Option<String>>,
        Vec<i32>,
        Vec<Option<i32>>,
        Vec<_>,
        Vec<OffsetDateTime>,
    ) = events
        .into_iter()
        .map(|e| {
            (
                e.chain_id.db,
                e.block_hash,
                e.block_height as i32,
                e.transaction_hash.map(Into::into),
                e.block_index,
                e.transaction_index,
                e.data,
                e.time,
            )
        })
        .multiunzip();

    sqlx::query!("
        INSERT INTO v0.events (chain_id, block_hash, height, transaction_hash, index, transaction_index, data, time)
        SELECT unnest($1::int[]), unnest($2::text[]), unnest($3::int[]), unnest($4::text[]), unnest($5::int[]), unnest($6::int[]), unnest($7::jsonb[]), unnest($8::timestamptz[])
        ", 
        &chain_ids, &block_hashes, &heights, &transaction_hashes as _, &indexes, &transaction_indexes as _, &data, &times)
    .execute(tx.as_mut()).await?;

    Ok(())
}

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
