use std::collections::{HashSet, HashMap};

use color_eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::cli::TokensUrls;

pub async fn update_tokens(db: sqlx::PgPool, urls: TokensUrls) -> Result<()> {
    info!("Starting token update process.");

    if let Some(token_lists) = fetch_token_lists(urls).await? {
        let mut token_inserts: HashSet<TokenInsert> = HashSet::new();

        for token_list in token_lists {
            for token in token_list.tokens {
                token_inserts.insert(token.into());
            }
        }

        info!("Fetched {} tokens before deduplication.", token_inserts.len());

        // Retrieve valid chain IDs and filter tokens
        let chain_ids_and_ids = crate::postgres::get_chain_ids_and_ids(&db).await?;
        let filtered_tokens: Vec<_> = token_inserts
            .into_iter()
            .filter_map(|token| map_valid_token(&chain_ids_and_ids, token))
            .collect();

        info!("Number of filtered tokens: {}", filtered_tokens.len());

        if filtered_tokens.is_empty() {
            info!("No tokens to update. Process completed.");
            return Ok(());
        }

        // Insert or update tokens in the database
        crate::postgres::insert_or_update_tokens(&db, &filtered_tokens).await?;
        info!("Successfully updated tokens in the database.");
    }

    Ok(())
}

/// Fetch and parse token lists from URLs concurrently.
async fn fetch_token_lists(urls: TokensUrls) -> Result<Option<Vec<TokenList>>> {
    let client = reqwest::Client::new();

    let tokens: Vec<_> = futures::stream::iter(urls.into_iter().map(|url| {
        let client = client.clone();
        async move {
            match client.get(&url).send().await {
                Ok(response) => match response.json::<serde_json::Value>().await {
                    Ok(val) if val.get("statusCode").is_none() => {
                        debug!("Successfully retrieved token list from: {}", url);
                        serde_json::from_value::<TokenList>(val).map(Some).map_err(Into::into)
                    }
                    Ok(_) => {
                        debug!("Invalid token list format at: {}", url);
                        Ok(None)
                    }
                    Err(err) => {
                        error!("Error parsing token list from {}: {}", url, err);
                        Err(err.into())
                    }
                },
                Err(err) => {
                    error!("Failed to fetch token list from {}: {}", url, err);
                    Err(err.into())
                }
            }
        }
    }))
    .buffer_unordered(10) // Adjust concurrency level as needed
    .filter_map(|res| async { res.transpose() })
    .collect()
    .await;

    if tokens.is_empty() {
        info!("No valid token lists found.");
        Ok(None)
    } else {
        Ok(Some(tokens.into_iter().flatten().collect()))
    }
}

/// Maps a token to the required database schema if it has a valid chain ID.
fn map_valid_token(
    chain_ids_and_ids: &HashMap<String, i64>,
    token: TokenInsert,
) -> Option<(i64, String, String, i64, Option<String>, String)> {
    chain_ids_and_ids.get(&token.chain_id.to_string()).map(|id| {
        (
            *id,
            token.denom,
            token.display_symbol,
            token.decimals,
            token.logo_uri,
            token.display_name,
        )
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenList {
    pub name: String,
    pub timestamp: String,
    pub version: Version,
    pub tokens: Vec<Token>,
    #[serde(rename = "logoURI")]
    pub logo_uri: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub major: i64,
    pub minor: i64,
    pub patch: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub chain_id: i64,
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: i64,
    #[serde(rename = "logoURI")]
    pub logo_uri: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TokenInsert {
    pub chain_id: i64,
    pub denom: String,
    pub display_symbol: String,
    pub decimals: i64,
    pub logo_uri: Option<String>,
    pub display_name: String,
}

impl From<Token> for TokenInsert {
    fn from(token: Token) -> Self {
        TokenInsert {
            chain_id: token.chain_id,
            denom: token.address,
            display_symbol: token.symbol,
            decimals: token.decimals,
            logo_uri: token.logo_uri,
            display_name: token.name,
        }
    }
}
