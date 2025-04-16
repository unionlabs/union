use std::collections::HashSet;

use itertools::Itertools;
use sqlx::{Postgres, Transaction};
use time::OffsetDateTime;

use crate::{
    indexer::api::{BlockHash, BlockHeight},
    postgres::{schedule_replication_reset, ChainId},
};

type TransactionHash = String;

/// DTO corresponding to the v2_cosmos.blocks table.
#[derive(Clone)]
pub struct PgBlock {
    pub chain_id: ChainId,
    pub hash: BlockHash,
    pub height: BlockHeight,
    pub time: OffsetDateTime,
    pub data: serde_json::Value,
}

/// DTO corresponding to the v2_cosmos.transactions table.
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

/// DTO corresponding to the v2_cosmos.events table.
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
        Vec<i64>,
        Vec<OffsetDateTime>,
    ) = blocks
        .into_iter()
        .map(|b| {
            let height: i64 = b.height.try_into().unwrap();
            (b.chain_id.db, b.hash, b.data, height, b.time)
        })
        .multiunzip();

    sqlx::query!("
        INSERT INTO v2_cosmos.blocks (internal_chain_id, hash, data, height, time)
        SELECT unnest($1::int[]), unnest($2::text[]), unnest($3::jsonb[]), unnest($4::bigint[]), unnest($5::timestamptz[])
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
        Vec<i64>,
        Vec<String>,
        Vec<_>,
        Vec<i32>,
    ) = transactions
        .into_iter()
        .map(|t| {
            let block_height: i64 = t.block_height.try_into().unwrap();

            (
                t.chain_id.db,
                t.block_hash,
                block_height,
                t.hash,
                t.data,
                t.index,
            )
        })
        .multiunzip();

    sqlx::query!("
        INSERT INTO v2_cosmos.transactions (internal_chain_id, block_hash, height, hash, data, index) 
        SELECT unnest($1::int[]), unnest($2::text[]), unnest($3::bigint[]), unnest($4::text[]), unnest($5::jsonb[]), unnest($6::int[])
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
        Vec<i64>,
        Vec<Option<String>>,
        Vec<i32>,
        Vec<Option<i32>>,
        Vec<_>,
        Vec<OffsetDateTime>,
    ) = events
        .into_iter()
        .map(|e| {
            let block_height: i64 = e.block_height.try_into().unwrap();

            (
                e.chain_id.db,
                e.block_hash,
                block_height,
                e.transaction_hash,
                e.block_index,
                e.transaction_index,
                e.data,
                e.time,
            )
        })
        .multiunzip();

    sqlx::query!("
        INSERT INTO v2_cosmos.events (internal_chain_id, block_hash, height, transaction_hash, index, transaction_index, data, time)
        SELECT unnest($1::int[]), unnest($2::text[]), unnest($3::bigint[]), unnest($4::text[]), unnest($5::int[]), unnest($6::int[]), unnest($7::jsonb[]), unnest($8::timestamptz[])
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
    let height: i64 = height.try_into().unwrap();

    sqlx::query!(
        "
        DELETE FROM v2_cosmos.events WHERE internal_chain_id = $1 AND height = $2
        ",
        chain_id,
        height,
    )
    .execute(tx.as_mut())
    .await?;

    sqlx::query!(
        "
        DELETE FROM v2_cosmos.transactions WHERE internal_chain_id = $1 AND height = $2
        ",
        chain_id,
        height,
    )
    .execute(tx.as_mut())
    .await?;

    sqlx::query!(
        "
        DELETE FROM v2_cosmos.blocks WHERE internal_chain_id = $1 AND height = $2
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
) -> sqlx::Result<HashSet<String>> {
    let height: i64 = height.try_into().unwrap();

    let result = sqlx::query!(
        r#"
        SELECT    address
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
    .map(|record| record.address)
    .collect();

    Ok(result)
}
