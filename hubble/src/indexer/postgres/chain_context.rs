use sqlx::Postgres;

use crate::indexer::{
    api::IndexerError,
    event::types::UniversalChainId,
    record::{ChainContext, ChainNetwork, PgValue},
};

pub async fn fetch_chain_context_for_universal_chain_id(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    universal_chain_id: &UniversalChainId,
) -> Result<ChainContext, IndexerError> {
    match sqlx::query!(
        "
        SELECT id as internal_chain_id, testnet
        FROM config.chains c WHERE family || '.' || chain_id = $1 
        LIMIT 1
        ",
        universal_chain_id.pg_value()?,
    )
    .fetch_optional(tx.as_mut())
    .await?
    {
        Some(record) => Ok(ChainContext {
            internal_chain_id: record.internal_chain_id.into(),
            network: match record.testnet {
                Some(true) => ChainNetwork::Testnet,
                Some(false) => ChainNetwork::Mainnet,
                None => ChainNetwork::Testnet,
            },
        }),
        None => Err(IndexerError::MissingChainConfiguration(
            universal_chain_id.clone(),
        )),
    }
}
