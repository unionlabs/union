use alloy::primitives::Address;
use sqlx::{Postgres, Transaction};

use crate::{
    github_client::GitCommitHash,
    indexer::{
        api::IndexerError,
        ethereum::abi::{Abi, AbiRegistration},
        record::{InternalChainId, PgValue},
    },
};

pub async fn get_abi_registration(
    tx: &mut Transaction<'_, Postgres>,
    internal_chain_id: InternalChainId,
    height: crate::indexer::event::types::BlockHeight,
) -> Result<AbiRegistration, IndexerError> {
    let result = sqlx::query!(
        r#"
        SELECT    internal_chain_id, address, abi, description, commit
        FROM      v2_evm.contracts
        WHERE     internal_chain_id = $1
        AND       $2 between start_height and end_height
        AND       abi IS NOT NULL
        "#,
        internal_chain_id.pg_value()?,
        height.pg_value()?,
    )
    .fetch_all(tx.as_mut())
    .await?
    .into_iter()
    .map(|record| {
        Ok(Abi {
            internal_chain_id: record.internal_chain_id.into(),
            address: record.address.parse::<Address>()?,
            definition: record.abi.expect("abi not null"),
            description: record.description.expect("description not null"),
            commit: GitCommitHash::from_slice(record.commit.as_slice())
                .map_err(IndexerError::InvalidCommitHashForAbi)?,
        })
    })
    .collect::<Result<Vec<Abi>, IndexerError>>()?
    .into_iter()
    .map(|a| (a.address, a))
    .collect();

    Ok(AbiRegistration {
        administration: result,
    })
}

pub async fn ensure_abi_dependency(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    commit: GitCommitHash,
) -> Result<bool, IndexerError> {
    let result = sqlx::query!(
        "
            INSERT INTO abi.dependency(commit)
            VALUES ($1)
            ON CONFLICT DO NOTHING;
            ",
        &commit.0,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn generated_abi(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    commit: GitCommitHash,
    description: String,
) -> Result<Option<String>, IndexerError> {
    Ok(sqlx::query!(
        "
            SELECT data ->> $2 AS abi
            FROM abi.download
            WHERE commit = $1;
            ",
        &commit.0,
        description,
    )
    .fetch_optional(tx.as_mut())
    .await?
    .and_then(|record| record.abi))
}

pub async fn update_contract_abi(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    internal_chain_id: InternalChainId,
    contract: Address,
    abi: String,
) -> Result<bool, IndexerError> {
    // sqlx bug workaround: temporarily disable client_min_messages to suppress NOTICE messages
    sqlx::query!("SET LOCAL client_min_messages = WARNING")
        .execute(tx.as_mut())
        .await?;

    let result = sqlx::query!(
        "
            UPDATE v2_evm.contracts
            SET abi = $1
            WHERE internal_chain_id = $2 
            AND address = $3 
            AND abi <> $1
        ",
        abi,
        internal_chain_id.0,
        format!("{:#x}", contract),
    )
    .execute(tx.as_mut())
    .await?;

    Ok(result.rows_affected() > 0)
}
