use std::collections::HashMap;

use sqlx::Postgres;

use crate::token_fetcher::{TokenRepresentation, TokenSource};

pub async fn get_token_sources(
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> sqlx::Result<Vec<TokenSource>> {
    Ok(sqlx::query!(
        r#"
        SELECT id, source_url, name, logo_url
        FROM hubble.token_sources
        WHERE enabled = true
        ORDER BY id
        "#,
    )
    .fetch_all(tx.as_mut())
    .await?
    .into_iter()
    .map(|record| TokenSource {
        id: record.id,
        source_url: record.source_url,
        name: record.name,
        logo_url: record.logo_url,
    })
    .collect())
}

pub async fn get_token_representations(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    token_source: &TokenSource,
) -> sqlx::Result<Vec<TokenRepresentation>> {
    Ok(sqlx::query!(
        r#"
        SELECT token_source_id, internal_chain_id, address, symbol, name, decimals, logo_url
        FROM hubble.token_source_representations
        WHERE token_source_id = $1
        "#,
        token_source.id,
    )
    .fetch_all(tx.as_mut())
    .await?
    .into_iter()
    .map(|record| TokenRepresentation {
        token_source_id: record.token_source_id,
        internal_chain_id: record.internal_chain_id,
        address: record.address,
        symbol: record.symbol,
        name: record.name,
        decimals: record.decimals,
        logo_url: record.logo_url,
    })
    .collect())
}

pub async fn delete_token_representation(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    token_representation: &TokenRepresentation,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
        DELETE
        FROM hubble.token_source_representations
        WHERE token_source_id = $1 and address = $2
        "#,
        token_representation.token_source_id,
        token_representation.address,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}

pub async fn upsert_token_representation(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    token_representation: &TokenRepresentation,
) -> sqlx::Result<()> {
    sqlx::query!(
        "
        INSERT INTO hubble.token_source_representations (token_source_id, internal_chain_id, address, symbol, name, decimals, logo_url)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (token_source_id, internal_chain_id, address) DO 
        UPDATE SET
            symbol = excluded.symbol,
            name = excluded.name,
            decimals = excluded.decimals,
            logo_url = excluded.logo_url
        ",
        token_representation.token_source_id, 
        token_representation.internal_chain_id, 
        token_representation.address, 
        token_representation.symbol, 
        token_representation.name, 
        token_representation.decimals, 
        token_representation.logo_url,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}

pub async fn update_token_source(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    token_source: &TokenSource,
) -> sqlx::Result<()> {
    sqlx::query!(
        "
        UPDATE hubble.token_sources
        SET 
            source_url = $2, 
            name = $3, 
            logo_url = $4
        WHERE id = $1
        ",
        token_source.id, 
        token_source.source_url, 
        token_source.name, 
        token_source.logo_url, 
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}

pub async fn get_internal_chain_id_by_chain_id(
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> sqlx::Result<HashMap<String, i32>> {
    let rows = sqlx::query!("SELECT chain_id, id FROM hubble.chains")
        .fetch_all(tx.as_mut())
        .await?;

    let result: HashMap<String, i32> =
        rows.into_iter().map(|row| (row.chain_id, row.id)).collect();

    Ok(result)
}
