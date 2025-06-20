use core::fmt::Debug;
use std::fmt;

use sqlx::{types::BigDecimal, Error, Postgres};
use valuable::Valuable;

use crate::indexer::api::UniversalChainId;

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
/// and the UCS04 universal chain id
#[derive(Clone, Debug, Valuable, PartialEq, Eq)]
pub struct ChainIdInner<'a> {
    pub db: i32,
    pub canonical: &'a str,
}

impl<'a> fmt::Display for ChainIdInner<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.canonical, self.db)
    }
}

/// Inside of Hubble, we leak the ChainId.canonical to make ChainIds easily copyable.
impl Copy for ChainIdInner<'static> {}

impl<'a> ChainIdInner<'a> {
    pub fn new(db: i32, canonical: &'a str) -> Self {
        Self { db, canonical }
    }
}

pub async fn fetch_internal_chain_id_for_universal_chain_id(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    universal_chain_id: &UniversalChainId,
) -> sqlx::Result<i32> {
    match sqlx::query!(
        "
        SELECT id as internal_chain_id 
        FROM config.chains c WHERE family || '.' || chain_id = $1 
        LIMIT 1
        ",
        universal_chain_id
    )
    .fetch_optional(tx.as_mut())
    .await?
    {
        Some(record) => Ok(record.internal_chain_id),
        None => Err(Error::Protocol("No chain found with universal_chain_id {universal_chain_id}. Add it to the config.chains table before using it in hubble".into()))
    }
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
