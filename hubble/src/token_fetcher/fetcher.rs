#![allow(clippy::type_complexity)]
use std::collections::{HashMap, HashSet};

use sqlx::Acquire;
use tracing::{debug, error, info};

use crate::token_fetcher::{
    client::{get_tokens, FetcherClientError, Token},
    postgres::{
        delete_token_representation, get_internal_chain_id_by_chain_id, get_token_representations,
        get_token_sources, update_token_source, upsert_token_representation,
    },
    TokenKey, TokenRepresentation, TokenSource,
};

#[derive(Debug, thiserror::Error)]
pub enum UpdateTokensError {
    #[error("client error fetching tokens of {0}: {1}")]
    FetchTokens(TokenSource, FetcherClientError),

    #[error("database error fetching tokens of {0}: {1}")]
    CreateTransaction(TokenSource, sqlx::Error),

    #[error("database error fetching chain ids of {0}: {1}")]
    GetChainIds(TokenSource, sqlx::Error),

    #[error("database error fetching token representation of {0}: {1}")]
    GetTokenRepresentations(TokenSource, sqlx::Error),

    #[error("database error updating token source of {0}: {1}")]
    UpdateTokenSource(TokenSource, sqlx::Error),

    #[error("database error updating token representation of {0}: {1}")]
    UpdateTokenRepresentation(TokenSource, TokenRepresentation, sqlx::Error),

    #[error("database error inserting token representation of {0}: {1}")]
    InsertTokenRepresentation(TokenSource, TokenRepresentation, sqlx::Error),

    #[error("database error deleting token representation of {0}: {1}")]
    DeleteTokenRepresentation(TokenSource, TokenRepresentation, sqlx::Error),

    #[error("database error fetching tokens of {0}: {1}")]
    CommitTransaction(TokenSource, sqlx::Error),
}

pub async fn update_tokens(db: &sqlx::PgPool) -> color_eyre::Result<()> {
    info!("Starting token update process.");

    let token_sources = get_token_sources(&mut db.acquire().await?.begin().await?).await?;

    for token_source in token_sources {
        debug!("process: {}", token_source);

        match update_tokens_for_source(db, &token_source).await {
            Ok(_) => debug!("process: {token_source} => success"),
            Err(error) => error!("process: {token_source} => error: {error:?}"),
        }
    }

    Ok(())
}

pub async fn update_tokens_for_source(
    db: &sqlx::PgPool,
    token_source: &TokenSource,
) -> Result<(), UpdateTokensError> {
    debug!("fetching tokens: {token_source}");

    let mut tx = db
        .begin()
        .await
        .map_err(|error| UpdateTokensError::CreateTransaction(token_source.clone(), error))?;

    let internal_chain_id_by_chain_id = get_internal_chain_id_by_chain_id(&mut tx)
        .await
        .map_err(|error| UpdateTokensError::GetChainIds(token_source.clone(), error))?;

    let tokens_from_source = get_tokens(&token_source.source_uri)
        .await
        .map_err(|error| UpdateTokensError::FetchTokens(token_source.clone(), error))?;

    if token_source.logo_uri != tokens_from_source.logo_uri
        || token_source.name != tokens_from_source.name
    {
        debug!("update source details: {token_source}");

        update_token_source(
            &mut tx,
            &TokenSource {
                id: token_source.id,
                source_uri: token_source.source_uri.clone(),
                name: tokens_from_source.name,
                logo_uri: tokens_from_source.logo_uri,
            },
        )
        .await
        .map_err(|error| UpdateTokensError::UpdateTokenSource(token_source.clone(), error))?;
    }

    let tokens_from_db = get_token_representations(&mut tx, token_source)
        .await
        .map_err(|error| UpdateTokensError::GetTokenRepresentations(token_source.clone(), error))?;

    let tokens_from_source_by_key: HashMap<TokenKey, Token> = tokens_from_source
        .tokens
        .iter()
        .filter_map(|token| {
            internal_chain_id_by_chain_id
                .get(&token.chain_id)
                .map(|internal_chain_id| {
                    (
                        TokenKey {
                            internal_chain_id: *internal_chain_id,
                            address: token.address.clone(),
                        },
                        token.clone(),
                    )
                })
        })
        .collect();

    let tokens_from_db_by_key: HashMap<TokenKey, TokenRepresentation> = tokens_from_db
        .iter()
        .map(|token| {
            (
                TokenKey {
                    internal_chain_id: token.internal_chain_id,
                    address: token.address.clone(),
                },
                token.clone(),
            )
        })
        .collect();

    let tokens_keys_from_source = tokens_from_source_by_key.keys().collect::<HashSet<_>>();
    let tokens_keys_from_db = tokens_from_db_by_key.keys().collect::<HashSet<_>>();

    let common_token_keys = tokens_keys_from_source.intersection(&tokens_keys_from_db);
    for common_token_key in common_token_keys {
        let token_from_source = tokens_from_source_by_key
            .get(common_token_key)
            .expect("token to exist in source (common)");

        let token_from_db = tokens_from_db_by_key
            .get(common_token_key)
            .expect("token to exist in db (common)");

        if token_from_source.decimals != token_from_db.decimals
            || token_from_source.logo_uri != token_from_db.logo_uri
            || token_from_source.name != token_from_db.name
            || token_from_source.symbol != token_from_db.symbol
        {
            debug!("update token: {token_from_db} <> {token_from_source}");

            let token_representation = TokenRepresentation {
                token_source_id: token_from_db.token_source_id,
                internal_chain_id: token_from_db.internal_chain_id,
                address: token_from_db.address.clone(),
                symbol: token_from_source.symbol.clone(),
                name: token_from_source.name.clone(),
                decimals: token_from_source.decimals,
                logo_uri: token_from_source.logo_uri.clone(),
            };

            upsert_token_representation(&mut tx, &token_representation)
                .await
                .map_err(|error| {
                    UpdateTokensError::UpdateTokenRepresentation(
                        token_source.clone(),
                        token_representation,
                        error,
                    )
                })?
        }
    }

    let new_token_keys = tokens_keys_from_source.difference(&tokens_keys_from_db);
    for new_token_key in new_token_keys {
        let token_from_source = tokens_from_source_by_key
            .get(new_token_key)
            .expect("token to exist in source (new)");

        debug!("new token: {token_from_source}");

        let token_representation = TokenRepresentation {
            token_source_id: token_source.id,
            internal_chain_id: new_token_key.internal_chain_id,
            address: new_token_key.address.clone(),
            symbol: token_from_source.symbol.clone(),
            name: token_from_source.name.clone(),
            decimals: token_from_source.decimals,
            logo_uri: token_from_source.logo_uri.clone(),
        };

        upsert_token_representation(&mut tx, &token_representation)
            .await
            .map_err(|error| {
                UpdateTokensError::InsertTokenRepresentation(
                    token_source.clone(),
                    token_representation,
                    error,
                )
            })?
    }

    let deleted_tokens = tokens_keys_from_db.difference(&tokens_keys_from_source);
    for deleted_token in deleted_tokens {
        let token_from_db = tokens_from_db_by_key
            .get(deleted_token)
            .expect("token to exist in db (delete)");

        debug!("delete token: {token_from_db}");
        delete_token_representation(&mut tx, token_from_db)
            .await
            .map_err(|error| {
                UpdateTokensError::DeleteTokenRepresentation(
                    token_source.clone(),
                    token_from_db.clone(),
                    error,
                )
            })?
    }

    tx.commit()
        .await
        .map_err(|error| UpdateTokensError::CommitTransaction(token_source.clone(), error))?;

    Ok(())
}
