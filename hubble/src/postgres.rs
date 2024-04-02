use futures::{Stream, StreamExt, TryStreamExt};
use serde::Serialize;
use sqlx::{Acquire, Postgres, QueryBuilder};
use time::OffsetDateTime;
use tracing::{debug, info};
use valuable::Valuable;

pub const BIND_LIMIT: usize = 65535;

pub struct Block {
    pub chain_id: ChainId,
    pub hash: String,
    pub height: i32,
    pub time: OffsetDateTime,
    pub data: serde_json::Value,
}

pub struct Transaction {
    pub chain_id: ChainId,
    pub block_hash: String,
    pub block_height: i32,
    pub time: OffsetDateTime,
    pub data: serde_json::Value,
    pub hash: String,
    pub index: i32,
}

pub struct Event {
    pub chain_id: ChainId,
    pub block_hash: String,
    pub block_height: i32,
    pub time: OffsetDateTime,
    pub data: serde_json::Value,
    pub transaction_hash: Option<String>,
    pub transaction_index: Option<i32>,
    pub block_index: i32,
}

/// The internal representation of a chain-id, assigned by the database, combined
/// with the canonical chain-id (from the genesis).
#[derive(Copy, Clone, Debug, Valuable)]
pub struct ChainId {
    pub db: i32,
    // We do not track too many chains in hubble, hence leaking the canonical
    // chain-id makes the code more efficient and easier to pass IDs around as `Copy`.
    pub canonical: &'static str,
}

impl ChainId {
    pub fn new_leaked(db: i32, canonical: String) -> Self {
        let canonical = canonical.leak();
        Self { db, canonical }
    }
}

pub struct Log<T> {
    pub chain_id: ChainId,
    pub block_hash: String,
    pub height: i32,
    pub time: OffsetDateTime,
    pub data: T,
}

pub async fn insert_batch_logs<T: Serialize, B: Stream<Item = Log<T>>>(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    logs: B,
) -> sqlx::Result<()> {
    logs.chunks(BIND_LIMIT / 5)
        .map(Ok::<_, sqlx::Error>)
        .try_fold(tx.as_mut(), |tx, chunk| async {
            let mut logs_query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
                // Note the trailing space; most calls to `QueryBuilder` don't automatically insert
                // spaces as that might interfere with identifiers or quoted strings where exact
                // values may matter.
                "INSERT INTO v0.logs (chain_id, block_hash, data, height, time) ",
            );
            logs_query_builder.push_values(chunk.into_iter(), |mut b, log| {
                debug!(
                    chain_id = log.chain_id.canonical,
                    height = log.height,
                    block_hash = log.block_hash,
                    "batch inserting log"
                );
                b.push_bind(log.chain_id.db)
                    .push_bind(log.block_hash.clone())
                    .push_bind(serde_json::to_value(&log.data).unwrap())
                    .push_bind(log.height)
                    .push_bind(log.time);
            });
            logs_query_builder
                .build()
                .persistent(true)
                .execute(tx.as_mut())
                .await?;
            Ok(tx)
        })
        .await?;

    Ok(())
}

pub async fn upsert_log<T: Serialize>(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    log: Log<T>,
) -> sqlx::Result<()> {
    sqlx::query!(
        "
        INSERT INTO v0.logs (chain_id, block_hash, data, height, time) 
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (chain_id, block_hash, height)
        DO UPDATE
        SET chain_id = $1, block_hash = $2, data = $3, height = $4, time = $5
        ",
        log.chain_id.db,
        log.block_hash,
        serde_json::to_value(&log.data).unwrap(),
        log.height,
        log.time
    )
    .execute(tx.as_mut())
    .await?;
    Ok(())
}

pub async fn insert_batch_blocks<B: Stream<Item = Block>>(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    blocks: B,
) -> sqlx::Result<()> {
    blocks
        .chunks(BIND_LIMIT / 5)
        .map(Ok::<_, sqlx::Error>)
        .try_fold(tx.as_mut(), |tx, chunk| async {
            let mut blocks_query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
                // Note the trailing space; most calls to `QueryBuilder` don't automatically insert
                // spaces as that might interfere with identifiers or quoted strings where exact
                // values may matter.
                "INSERT INTO v0.blocks (chain_id, hash, data, height, time) ",
            );
            blocks_query_builder.push_values(chunk.into_iter(), |mut b, block| {
                debug!(
                    chain_id = block.chain_id.canonical,
                    height = block.height,
                    block_hash = block.hash,
                    "batch inserting block"
                );
                b.push_bind(block.chain_id.db)
                    .push_bind(block.hash.clone())
                    .push_bind(block.data.clone())
                    .push_bind(block.height)
                    .push_bind(block.time);
            });
            blocks_query_builder
                .build()
                .persistent(true)
                .execute(tx.as_mut())
                .await?;
            Ok(tx)
        })
        .await?;

    Ok(())
}

pub async fn insert_batch_transactions<B: Stream<Item = Transaction>>(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    transactions: B,
) -> sqlx::Result<()> {
    // We insert all transactions in batched statements without their logs first.
    transactions
        .chunks(BIND_LIMIT / 6)
        .map(Ok::<_, sqlx::Error>)
        .try_fold(tx.as_mut(), |tx, chunk| async {
            let mut tx_query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
                // Note the trailing space; most calls to `QueryBuilder` don't automatically insert
                // spaces as that might interfere with identifiers or quoted strings where exact
                // values may matter.
                "INSERT INTO v0.transactions (chain_id, block_hash, height, hash, data, index) ",
            );
            tx_query_builder.push_values(chunk.into_iter(), |mut b, transaction| {
                debug!(
                    chain_id = transaction.chain_id.canonical,
                    height = transaction.block_height,
                    block_hash = transaction.block_hash,
                    transaction_hash = &transaction.hash,
                    transaction_index = transaction.index,
                    "batch inserting transaction"
                );
                b.push_bind(transaction.chain_id.db)
                    .push_bind(transaction.block_hash)
                    .push_bind(transaction.block_height)
                    .push_bind(transaction.hash)
                    .push_bind(transaction.data)
                    .push_bind(transaction.index);
            });
            tx_query_builder
                .build()
                // Since there can be different amount of transactions in each block; we omit prepared statements,
                // as that would fill up the query cache.
                .persistent(false)
                .execute(tx.as_mut())
                .await?;
            Ok(tx)
        })
        .await?;
    Ok(())
}

pub async fn insert_batch_events<B: Stream<Item = Event>>(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    events: B,
) -> sqlx::Result<()> {
    events
    .chunks(BIND_LIMIT / 8)
    .map(Ok::<_, sqlx::Error>)
    .try_fold(tx.as_mut(), |tx, chunk| async {
        let mut event_query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        // Note the trailing space; most calls to `QueryBuilder` don't automatically insert
        // spaces as that might interfere with identifiers or quoted strings where exact
        // values may matter.
        "INSERT INTO v0.events (chain_id, block_hash, height, transaction_hash, index, transaction_index, data, time) ",
    );
    event_query_builder.push_values(chunk.into_iter(), |mut b, event| {
            debug!(
                chain_id = event.chain_id.canonical,
                height = event.block_height,
                block_hash = event.block_hash,
                transaction_hash = &event.transaction_hash,
                transaction_index = event.transaction_index,
                index = event.block_index,
                "batch inserting event"
            );
            b.push_bind(event.chain_id.db)
                .push_bind(event.block_hash)
                .push_bind(event.block_height)
                .push_bind(event.transaction_hash)
                .push_bind(event.block_index)
                .push_bind(event.transaction_index)
                .push_bind(event.data)
                .push_bind(event.time);
        });
        event_query_builder
            .build()
            // Since there can be different amount of events in each block; we omit prepared statements,
            // as that would fill up the query cache.
            .persistent(false)
            .execute(tx.as_mut())
            .await?;
        Ok(tx)
    })
    .await?;
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
        Fetched(ChainId::new_leaked(chain_id.id, canonical))
    } else {
        let id = sqlx::query!(
            "INSERT INTO \"v0\".chains (chain_id) VALUES ($1) RETURNING id",
            canonical.to_string()
        )
        .fetch_one(&mut *conn)
        .await?
        .id;
        Created(ChainId::new_leaked(id, canonical))
    };
    Ok(db_chain_id)
}
