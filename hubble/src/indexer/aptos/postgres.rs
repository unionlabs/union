use std::collections::HashSet;

use itertools::Itertools;
use serde_json::Value;
use sqlx::{PgPool, Postgres, Transaction};
use time::OffsetDateTime;
use tracing::trace;

use crate::{indexer::api::BlockHeight, postgres::schedule_replication_reset};

pub struct PgBlock {
    pub internal_chain_id: i32,
    pub height: i64,
    pub block_hash: String,
    pub timestamp: OffsetDateTime,
    pub first_version: i64,
    pub last_version: i64,
    pub transactions: Vec<PgTransaction>,
}

pub struct PgTransaction {
    pub internal_chain_id: i32,
    pub height: i64,
    pub version: i64,
    pub transaction_hash: String,
    pub transaction_index: i64,
    pub events: Vec<PgEvent>,
}

pub struct PgEvent {
    pub internal_chain_id: i32,
    pub height: i64,
    pub version: i64,
    pub index: i64,
    pub transaction_event_index: i64,
    pub sequence_number: i64,
    pub creation_number: i64,
    pub account_address: String,
    pub typ: String,
    pub data: Value,
}

pub async fn insert_aptos_block(
    tx: &mut Transaction<'_, Postgres>,
    block: PgBlock,
) -> sqlx::Result<()> {
    trace!("insert: {}", block.height);
    sqlx::query!(
        "
        INSERT INTO v1_aptos.blocks (
            internal_chain_id, 
            block_hash, 
            height, 
            timestamp, 
            first_version, 
            last_version
        ) VALUES ($1, $2, $3, $4, $5, $6);
        ",
        block.internal_chain_id,
        block.block_hash,
        block.height,
        block.timestamp,
        block.first_version,
        block.last_version,
    )
    .execute(tx.as_mut())
    .await?;

    for transaction in block.transactions {
        trace!("insert: {}/{}", block.height, transaction.version);
        sqlx::query!(
            "
            INSERT INTO v1_aptos.transactions (
                internal_chain_id, 
                height,
                version,
                transaction_hash,
                transaction_index
            ) VALUES ($1, $2, $3, $4, $5);
            ",
            transaction.internal_chain_id,
            transaction.height,
            transaction.version,
            transaction.transaction_hash,
            transaction.transaction_index,
        )
        .execute(tx.as_mut())
        .await?;

        for event in transaction.events {
            trace!(
                "insert: {}/{}/{} ({}) {}#{}",
                block.height,
                transaction.version,
                event.creation_number,
                event.account_address,
                event.typ,
                event.sequence_number
            );
            sqlx::query!(
                "
                INSERT INTO v1_aptos.events (
                    internal_chain_id,
                    height,
                    version,
                    sequence_number,
                    creation_number,
                    index,
                    transaction_event_index,
                    account_address,
                    type,
                    data
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10);
                ",
                event.internal_chain_id,
                event.height,
                event.version,
                event.sequence_number,
                event.creation_number,
                event.index,
                event.transaction_event_index,
                event.account_address,
                event.typ,
                event.data
            )
            .execute(tx.as_mut())
            .await?;
        }
    }

    Ok(())
}

pub async fn delete_aptos_block_transactions_events(
    tx: &mut Transaction<'_, Postgres>,
    internal_chain_id: i32,
    height: BlockHeight,
) -> sqlx::Result<()> {
    sqlx::query!(
        "
        DELETE FROM v1_aptos.events WHERE internal_chain_id = $1 AND height = $2
        ",
        internal_chain_id,
        height as i64
    )
    .execute(tx.as_mut())
    .await?;

    sqlx::query!(
        "
        DELETE FROM v1_aptos.transactions WHERE internal_chain_id = $1 AND height = $2
        ",
        internal_chain_id,
        height as i64
    )
    .execute(tx.as_mut())
    .await?;

    sqlx::query!(
        "
        DELETE FROM v1_aptos.blocks WHERE internal_chain_id = $1 AND height = $2
        ",
        internal_chain_id,
        height as i64,
    )
    .execute(tx.as_mut())
    .await?;

    schedule_replication_reset(tx, internal_chain_id, height as i64, "block reorg (delete)")
        .await?;

    Ok(())
}

pub async fn active_contracts(
    tx: &mut Transaction<'_, Postgres>,
    internal_chain_id: i32,
    height: BlockHeight,
) -> sqlx::Result<HashSet<String>> {
    let result = sqlx::query!(
        r#"
        SELECT    address
        FROM      v1_aptos.contracts
        WHERE     internal_chain_id = $1
        AND       $2 between start_height and end_height
        "#,
        internal_chain_id,
        height as i64,
    )
    .fetch_all(tx.as_mut())
    .await?
    .into_iter()
    .map(|record| record.address)
    .collect();

    Ok(result)
}
pub struct UnmappedClient {
    pub version: u64,
    pub height: BlockHeight,
    pub client_id: Option<String>,
}

pub async fn unmapped_clients(
    pg_pool: &PgPool,
    internal_chain_id: i32,
) -> sqlx::Result<Vec<UnmappedClient>> {
    let result = sqlx::query!(
        r#"
        SELECT cc.transaction_version, cc.height, cc.client_id
        FROM   v1_aptos.client_created_event cc
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
        version: record.transaction_version.expect("client-created-event to have transaction version") as u64,
        height: record.height.expect("client-created-event to have a height") as u64,
        client_id: record.client_id,
    })
    .collect_vec();

    Ok(result)
}
