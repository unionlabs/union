use core::fmt::Debug;
use std::fmt;

use itertools::Itertools;
use serde::Deserialize;
use sqlx::{types::BigDecimal, Acquire, Postgres};
use tracing::info;
use valuable::Valuable;

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
#[derive(Clone, Debug, Valuable, PartialEq, Eq)]
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

#[derive(Copy, Clone, Debug, Default, Deserialize)]
pub enum InsertMode {
    #[default]
    Insert,
    Upsert,
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

pub async fn fetch_or_insert_chain_id_tx(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    canonical: String,
) -> sqlx::Result<FetchOrCreated<ChainId>> {
    use FetchOrCreated::*;
    let db_chain_id = if let Some(chain_id) = sqlx::query!(
        "SELECT id FROM \"v0\".chains WHERE chain_id = $1 LIMIT 1",
        canonical.to_string()
    )
    .fetch_optional(tx.as_mut())
    .await?
    {
        Fetched(ChainId::new(chain_id.id, canonical.leak()))
    } else {
        let id = sqlx::query!(
            "INSERT INTO \"v0\".chains (chain_id) VALUES ($1) RETURNING id",
            canonical.to_string()
        )
        .fetch_one(tx.as_mut())
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

pub async fn schedule_replication_reset(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    chain_id: i32,
    height: i64,
    reason: &str,
) -> sqlx::Result<()> {
    sqlx::query!(
        "CALL public.replication_schedule_reset_chain($1, $2, $3);",
        BigDecimal::from(chain_id),
        &height,
        reason
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
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

pub async fn insert_client_mapping<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    chain_id: i32,
    client_id: String,
    counterparty_chain_id: String,
) -> sqlx::Result<()> {
    let mut conn = db.acquire().await?;
    sqlx::query!(
        r#"
        INSERT INTO
            v0.clients (chain_id, client_id, counterparty_chain_id)
        VALUES
            ($1, $2, $3)
        ON CONFLICT DO NOTHING
        "#,
        chain_id,
        client_id,
        counterparty_chain_id,
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}

pub async fn get_chain_ids_and_ids<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
) -> sqlx::Result<std::collections::HashMap<String, i32>> {
    let mut conn = db.acquire().await?;

    let rows = sqlx::query!("SELECT chain_id, id FROM v0.chains")
        .fetch_all(&mut *conn)
        .await?;

    let chain_ids_and_ids: std::collections::HashMap<String, i32> =
        rows.into_iter().map(|row| (row.chain_id, row.id)).collect();

    Ok(chain_ids_and_ids)
}

#[allow(clippy::type_complexity)] // it's just kind of a mess
pub async fn insert_or_update_tokens<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    tokens: &[(i64, String, String, i64, Option<String>, String)],
) -> sqlx::Result<()> {
    let mut conn = db.acquire().await?;

    let (chain_ids, denoms, display_symbols, decimals, logo_uris, display_names): (
        Vec<i64>,
        Vec<String>,
        Vec<String>,
        Vec<i64>,
        Vec<Option<String>>,
        Vec<String>,
    ) = tokens
        .iter()
        .map(
            |(chain_id, denom, display_symbol, decimals, logo_uri, display_name)| {
                (
                    *chain_id,
                    denom.clone(),
                    display_symbol.clone(),
                    *decimals,
                    logo_uri.clone(),
                    display_name.clone(),
                )
            },
        )
        .multiunzip();

    sqlx::query!(
        r#"
        INSERT INTO v0.assets (chain_id, denom, display_symbol, decimals, logo_uri, display_name, gas_token)
        SELECT 
            unnest($1::bigint[]), 
            unnest($2::text[]), 
            unnest($3::text[]), 
            unnest($4::bigint[]), 
            unnest($5::text[]), 
            unnest($6::text[]), 
            false
        ON CONFLICT (chain_id, denom) DO UPDATE SET
            display_symbol = EXCLUDED.display_symbol,
            decimals = EXCLUDED.decimals,
            logo_uri = EXCLUDED.logo_uri,
            display_name = EXCLUDED.display_name
        "#,
        &chain_ids,
        &denoms,
        &display_symbols,
        &decimals,
        &logo_uris as _,
        &display_names
    )
    .execute(&mut *conn)
    .await?;

    info!("Successfully inserted or updated {} tokens.", tokens.len());

    Ok(())
}
