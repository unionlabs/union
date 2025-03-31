use core::fmt::Debug;
use std::fmt;

use serde::Deserialize;
use sqlx::{types::BigDecimal, Postgres};
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
        "SELECT id FROM \"hubble\".chains WHERE chain_id = $1 LIMIT 1",
        canonical.to_string()
    )
    .fetch_optional(tx.as_mut())
    .await?
    {
        Fetched(ChainId::new(chain_id.id, canonical.leak()))
    } else {
        let id = sqlx::query!(
            "INSERT INTO \"hubble\".chains (chain_id) VALUES ($1) RETURNING id",
            canonical.to_string()
        )
        .fetch_one(tx.as_mut())
        .await?
        .id;
        Created(ChainId::new(id, canonical.leak()))
    };
    Ok(db_chain_id)
}

pub async fn schedule_replication_reset(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    chain_id: i32,
    height: i64,
    reason: &str,
) -> sqlx::Result<()> {
    sqlx::query!(
        "CALL sync.replication_schedule_reset_chain($1, $2, $3);",
        BigDecimal::from(chain_id),
        &height,
        reason
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}
