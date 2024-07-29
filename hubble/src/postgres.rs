use core::fmt::Debug;
use std::fmt;

use futures::{Stream, TryStreamExt};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, PgPool, Postgres};
use time::OffsetDateTime;
use tracing::info;
use valuable::Valuable;

/// A trait to describe the different parameters of a chain, used to instantiate types for insertion.
pub trait ChainType {
    type BlockHeight;
    type BlockHash;
    type TransactionHash;
}

/// DTO corresponding to the v0.blocks table.
pub struct Block<Chain: ChainType> {
    pub chain_id: ChainId,
    pub hash: Chain::BlockHash,
    pub height: Chain::BlockHeight,
    pub time: OffsetDateTime,
    pub data: serde_json::Value,
}

/// DTO corresponding to the v0.transactions table.
pub struct Transaction<Chain: ChainType> {
    pub chain_id: ChainId,
    pub block_hash: Chain::BlockHash,
    pub block_height: Chain::BlockHeight,
    #[allow(dead_code)]
    pub time: OffsetDateTime,
    pub data: serde_json::Value,
    pub hash: Chain::TransactionHash,
    pub index: i32,
}

/// DTO corresponding to the v0.events table.
pub struct Event<Chain: ChainType> {
    pub chain_id: ChainId,
    pub block_hash: Chain::BlockHash,
    pub block_height: Chain::BlockHeight,
    pub time: OffsetDateTime,
    pub data: serde_json::Value,
    pub transaction_hash: Option<Chain::TransactionHash>,
    pub transaction_index: Option<i32>,
    pub block_index: i32,
}

/// DTO corresponding to the v0.logs table. Note that `logs` are considered opaque, unprocessed
/// chunks of data depending on the chain type. For example, for Ethereum, a log is a header + transaction receipts.
pub struct Log<Chain: ChainType, T> {
    pub chain_id: ChainId,
    pub block_hash: Chain::BlockHash,
    pub height: Chain::BlockHeight,
    pub time: OffsetDateTime,
    pub data: T,
}

/// ChainIds track both the database ID of a chain, as well as some canonical representation for
/// debug logging.
///
/// # Implementation Detail
/// ChainIds contain leaked values, hence care should be taken when creating them.
///
/// We do not track too many chains in hubble, hence leaking the canonical
/// chain-id makes the code more efficient and easier to pass IDs around as `Copy`.
pub type ChainId = ChainIdInner<'static>;

/// The internal representation of a chain-id, assigned by the database, combined
/// with the canonical chain-id (from the genesis).
#[derive(Clone, Debug, Valuable)]
pub struct ChainIdInner<'a> {
    pub db: i32,
    pub canonical: &'a str,
}

impl<'a> fmt::Display for ChainIdInner<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.canonical)
    }
}

/// Inside of Hubble, we leak the ChainId.canonical to make ChainIds easily copyable.
impl Copy for ChainIdInner<'static> {}

impl<'a> ChainIdInner<'a> {
    pub fn new(db: i32, canonical: &'a str) -> Self {
        Self { db, canonical }
    }
}

pub fn get_last_n_logs<'a>(
    db: &'a PgPool,
    chain_id: ChainId,
    n: i64,
) -> sqlx::Result<impl Stream<Item = sqlx::Result<(String, i32)>> + 'a> {
    Ok(sqlx::query!(
        "SELECT block_hash, height from v0.logs where chain_id = $1 ORDER BY height DESC LIMIT $2",
        chain_id.db,
        n
    )
    .fetch(db)
    .map_ok(|r| (r.block_hash, r.height)))
}

pub fn get_n_logs_from<'a>(
    db: &'a PgPool,
    chain_id: ChainId,
    height: i32,
    n: i64,
) -> sqlx::Result<impl Stream<Item = sqlx::Result<(String, i32)>> + 'a> {
    Ok(sqlx::query!(
        "SELECT block_hash, height from v0.logs where chain_id = $1 and height >= $2 ORDER BY height ASC LIMIT $3",
        chain_id.db,
        height,
        n
    )
    .fetch(db)
    .map_ok(|r| (r.block_hash, r.height)))
}

pub async fn update_batch_logs<C: ChainType, T: Serialize>(
    db: &PgPool,
    logs: impl IntoIterator<Item = Log<C, T>>,
) -> sqlx::Result<()>
where
    <C as ChainType>::BlockHeight: Into<i32>,
    <C as ChainType>::BlockHash: Into<String> + Debug,
{
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
                l.block_hash.into(),
                serde_json::to_value(&l.data).expect("data should be json serializable"),
                l.height.into(),
                l.time,
            )
        })
        .multiunzip();

    sqlx::query!("
        UPDATE v0.logs
        SET chain_id = batch.chain_id,
            block_hash = batch.block_hash,
            data = batch.data,
            height = batch.height,
            time = batch.time
        FROM (
            SELECT unnest($1::int[]) as chain_id, unnest($2::text[]) as block_hash, unnest($3::jsonb[]) as data, unnest($4::int[]) as height, unnest($5::timestamptz[]) as time
        ) as batch
        WHERE batch.height = v0.logs.height AND batch.chain_id = v0.logs.chain_id
        ", &chain_ids, &hashes, &data, &height, &time)
    .execute(db).await?;
    Ok(())
}

pub async fn insert_batch_logs<C: ChainType, T: Serialize>(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    logs: impl IntoIterator<Item = Log<C, T>>,
    mode: InsertMode,
) -> sqlx::Result<()>
where
    <C as ChainType>::BlockHeight: Into<i32>,
    <C as ChainType>::BlockHash: Into<String> + Debug,
{
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
                l.block_hash.into(),
                serde_json::to_value(&l.data).expect("data should be json serializable"),
                l.height.into(),
                l.time,
            )
        })
        .multiunzip();

    if mode.is_insert() {
        sqlx::query!("
            INSERT INTO v0.logs (chain_id, block_hash, data, height, time)
            SELECT unnest($1::int[]), unnest($2::text[]), unnest($3::jsonb[]), unnest($4::int[]), unnest($5::timestamptz[])
            ", &chain_ids, &hashes, &data, &height, &time)
        .execute(tx.as_mut()).await?;
    } else {
        sqlx::query!("
            INSERT INTO v0.logs (chain_id, block_hash, data, height, time)
            SELECT unnest($1::int[]), unnest($2::text[]), unnest($3::jsonb[]), unnest($4::int[]), unnest($5::timestamptz[])
            ON CONFLICT (block_hash) DO 
            UPDATE SET
                chain_id = excluded.chain_id,
                block_hash = excluded.block_hash,
                data = excluded.data,
                height = excluded.height,
                time = excluded.time
            ", &chain_ids, &hashes, &data, &height, &time)
        .execute(tx.as_mut()).await?;
    }
    Ok(())
}

#[derive(Copy, Clone, Debug, Default, Deserialize)]
pub enum InsertMode {
    #[default]
    Insert,
    Upsert,
}

impl InsertMode {
    fn is_insert(&self) -> bool {
        matches!(self, InsertMode::Insert)
    }
}

pub async fn insert_batch_blocks<C: ChainType>(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    blocks: impl IntoIterator<Item = Block<C>>,
    mode: InsertMode,
) -> sqlx::Result<()>
where
    <C as ChainType>::BlockHeight: Into<i32>,
    <C as ChainType>::BlockHash: Into<String> + Debug,
{
    let (chain_ids, hashes, data, height, time): (
        Vec<i32>,
        Vec<String>,
        Vec<_>,
        Vec<i32>,
        Vec<OffsetDateTime>,
    ) = blocks
        .into_iter()
        .map(|b| {
            (
                b.chain_id.db,
                b.hash.into(),
                b.data,
                b.height.into(),
                b.time,
            )
        })
        .multiunzip();

    if mode.is_insert() {
        sqlx::query!("
            INSERT INTO v0.blocks (chain_id, hash, data, height, time)
            SELECT unnest($1::int[]), unnest($2::text[]), unnest($3::jsonb[]), unnest($4::int[]), unnest($5::timestamptz[])
            ", &chain_ids, &hashes, &data, &height, &time)
        .execute(tx.as_mut()).await?;
    } else {
        sqlx::query!("
            INSERT INTO v0.blocks (chain_id, hash, data, height, time)
            SELECT unnest($1::int[]), unnest($2::text[]), unnest($3::jsonb[]), unnest($4::int[]), unnest($5::timestamptz[])
            ON CONFLICT (hash) DO 
            UPDATE SET
                chain_id = excluded.chain_id,
                hash = excluded.hash,
                data = excluded.data,
                height = excluded.height,
                time = excluded.time
        ", &chain_ids, &hashes, &data, &height, &time)
        .execute(tx.as_mut()).await?;
    }
    Ok(())
}

pub async fn insert_batch_transactions<C: ChainType>(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    transactions: impl IntoIterator<Item = Transaction<C>>,
    mode: InsertMode,
) -> sqlx::Result<()>
where
    <C as ChainType>::BlockHeight: Into<i32> + Debug,
    <C as ChainType>::BlockHash: Into<String> + Debug,
    <C as ChainType>::TransactionHash: Into<String> + Debug,
{
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
                t.block_hash.into(),
                t.block_height.into(),
                t.hash.into(),
                t.data,
                t.index,
            )
        })
        .multiunzip();

    if mode.is_insert() {
        sqlx::query!("
            INSERT INTO v0.transactions (chain_id, block_hash, height, hash, data, index) 
            SELECT unnest($1::int[]), unnest($2::text[]), unnest($3::int[]), unnest($4::text[]), unnest($5::jsonb[]), unnest($6::int[])
            ", 
            &chain_ids, &block_hashes, &heights, &hashes, &data, &indexes)
        .execute(tx.as_mut()).await?;
    } else {
        sqlx::query!("
            INSERT INTO v0.transactions (chain_id, block_hash, height, hash, data, index)
            SELECT unnest($1::int[]), unnest($2::text[]), unnest($3::int[]), unnest($4::text[]), unnest($5::jsonb[]), unnest($6::int[])
            ON CONFLICT (hash) DO
            UPDATE SET
                chain_id = excluded.chain_id,
                block_hash = excluded.block_hash,
                height = excluded.height,
                hash = excluded.hash,
                data = excluded.data,
                index = excluded.index
        ", 
        &chain_ids, &block_hashes, &heights, &hashes, &data, &indexes)
        .execute(tx.as_mut()).await?;
    }
    Ok(())
}

pub async fn insert_batch_events<C: ChainType>(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    events: impl IntoIterator<Item = Event<C>>,
    mode: InsertMode,
) -> sqlx::Result<()>
where
    <C as ChainType>::BlockHeight: Into<i32> + Debug,
    <C as ChainType>::BlockHash: Into<String> + Debug,
    <C as ChainType>::TransactionHash: Into<String> + Debug,
{
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
                e.block_hash.into(),
                e.block_height.into(),
                e.transaction_hash.map(Into::into),
                e.block_index,
                e.transaction_index,
                e.data,
                e.time,
            )
        })
        .multiunzip();

    if mode.is_insert() {
        sqlx::query!("
            INSERT INTO v0.events (chain_id, block_hash, height, transaction_hash, index, transaction_index, data, time)
            SELECT unnest($1::int[]), unnest($2::text[]), unnest($3::int[]), unnest($4::text[]), unnest($5::int[]), unnest($6::int[]), unnest($7::jsonb[]), unnest($8::timestamptz[])
            ", 
            &chain_ids, &block_hashes, &heights, &transaction_hashes as _, &indexes, &transaction_indexes as _, &data, &times)
        .execute(tx.as_mut()).await?;
    } else {
        sqlx::query!("
        INSERT INTO v0.events (chain_id, block_hash, height, transaction_hash, index, transaction_index, data, time)
        SELECT unnest($1::int[]), unnest($2::text[]), unnest($3::int[]), unnest($4::text[]), unnest($5::int[]), unnest($6::int[]), unnest($7::jsonb[]), unnest($8::timestamptz[])
        ON CONFLICT (transaction_hash, index) DO
        UPDATE SET
            chain_id = excluded.chain_id,
            block_hash = excluded.block_hash,
            height = excluded.height,
            transaction_hash = excluded.transaction_hash,
            index = excluded.index,
            transaction_index = excluded.transaction_index,
            data = excluded.data,
            time = excluded.time
    ", 
    &chain_ids, &block_hashes, &heights, &transaction_hashes as _, &indexes, &transaction_indexes as _, &data, &times)
    .execute(tx.as_mut()).await?;
    }
    Ok(())
}

pub enum FetchOrCreated<T> {
    Fetched(T),
    Created(T),
}

impl<T> FetchOrCreated<T> {
    pub fn get_inner_logged(self) -> T {
        match self {
            FetchOrCreated::Fetched(id) => id,
            FetchOrCreated::Created(id) => {
                info!("no existing chain-id found in db, created");
                id
            }
        }
    }
}

pub async fn fetch_or_insert_chain_id<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    canonical: String,
) -> sqlx::Result<FetchOrCreated<ChainId>> {
    use FetchOrCreated::*;
    let mut conn = db.acquire().await?;
    let db_chain_id = if let Some(chain_id) = sqlx::query!(
        "SELECT id FROM \"v0\".chains WHERE chain_id = $1 LIMIT 1",
        canonical.to_string()
    )
    .fetch_optional(&mut *conn)
    .await?
    {
        Fetched(ChainId::new(chain_id.id, canonical.leak()))
    } else {
        let id = sqlx::query!(
            "INSERT INTO \"v0\".chains (chain_id) VALUES ($1) RETURNING id",
            canonical.to_string()
        )
        .fetch_one(&mut *conn)
        .await?
        .id;
        Created(ChainId::new(id, canonical.leak()))
    };
    Ok(db_chain_id)
}

pub async fn get_chain_id<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    canonical: String,
) -> sqlx::Result<Option<ChainId>> {
    let mut conn = db.acquire().await?;
    let id = sqlx::query!(
        "SELECT id FROM \"v0\".chains WHERE chain_id = $1 LIMIT 1",
        canonical.to_string()
    )
    .fetch_optional(&mut *conn)
    .await?
    .map(|r| ChainId::new(r.id, canonical.leak()));
    Ok(id)
}

pub async fn insert_mapped_execution_heights<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    execution_heights: Vec<i64>,
    consensus_heights: Vec<i64>,
    chain_id: ChainId,
) -> sqlx::Result<()> {
    let mut conn = db.acquire().await?;
    sqlx::query!(
        "
        INSERT INTO v0.consensus_heights (chain_id, consensus_height, execution_height)
        SELECT $1, unnest($2::bigint[]), unnest($3::bigint[])
        ",
        chain_id.db,
        &consensus_heights,
        &execution_heights,
    )
    .execute(&mut *conn)
    .await?;
    Ok(())
}

pub async fn update_contracts_indexed_heights<'a>(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    contracts: Vec<String>,
    heights: Vec<i64>,
    chain_id: ChainId,
) -> sqlx::Result<usize> {
    let rows_updated = sqlx::query!(
        "
        UPDATE v0.contracts 
        SET indexed_height = data.height
        FROM (
            SELECT unnest($1::bigint[]) as height, unnest($2::text[]) as address
        ) as data
        WHERE v0.contracts.address = data.address AND chain_id = $3
        RETURNING v0.contracts.address
        ",
        &heights,
        &contracts,
        &chain_id.db,
    )
    .fetch_all(tx.as_mut())
    .await?
    .iter()
    .count();
    Ok(rows_updated)
}

pub async fn get_max_consensus_height<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    chain_id: ChainId,
) -> sqlx::Result<i64> {
    let mut conn = db.acquire().await?;
    let height = sqlx::query!(
        "
        SELECT MAX(consensus_height) as height from v0.consensus_heights
        WHERE chain_id = $1
        ",
        chain_id.db
    )
    .fetch_optional(&mut *conn)
    .await?
    .map(|r| r.height.unwrap_or_default())
    .unwrap_or(0);

    Ok(height)
}
