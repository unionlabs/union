use core::fmt::Debug;
use std::fmt;

use serde::Deserialize;
use sqlx::{types::BigDecimal, Error, Postgres};
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

pub async fn fetch_chain_id_tx(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    canonical: String,
) -> sqlx::Result<ChainId> {
    match sqlx::query!(
        "SELECT id FROM config.chains WHERE chain_id = $1 LIMIT 1",
        canonical.to_string()
    )
    .fetch_optional(tx.as_mut())
    .await?
    {
        Some(chain_id) => Ok(ChainId::new(chain_id.id, canonical.leak())),
        None => Err(Error::Protocol("No chain found with chain_id {canonical}. Add it to the config.chains table before using it in hubble".into()))
    }
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
