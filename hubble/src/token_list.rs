use color_eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::cli::TokensUrls;

pub async fn update_tokens(db: sqlx::PgPool, urls: TokensUrls) -> Result<()> {
    info!("Starting token update process.");
    if let Some(token_list) = get_tokens(urls).await? {
        let mut token_inserts = Vec::new();

        for token_value in token_list {
            for token in token_value.tokens {
                let token_insert: TokenInsert = token.into();
                token_inserts.push((
                    token_insert.chain_id,
                    token_insert.denom.clone(),
                    token_insert.display_symbol.clone(),
                    token_insert.decimals,
                    token_insert.logo_uri.clone(),
                    token_insert.display_name.clone(),
                ));
            }
        }

        // Check for duplicate tokens
        let with_duplicates = token_inserts.len();
        let mut seen = std::collections::HashSet::new();
        token_inserts.retain(|token| seen.insert((token.0, token.1.clone())));
        info!(
            "Total number of tokens removed during filtering: {}",
            with_duplicates - token_inserts.len()
        );

        if !token_inserts.is_empty() {
            let chain_ids_and_ids = crate::postgres::get_chain_ids_and_ids(&db).await?;
            // Filter tokens based on valid chain_id
            let filtered_tokens: Vec<(i64, String, String, i64, Option<String>, String)> =
                token_inserts
                    .iter()
                    .filter_map(|token| {
                        chain_ids_and_ids.get(&token.0.to_string()).map(|id| {
                            (
                                *id as i64,
                                token.1.clone(),
                                token.2.clone(),
                                token.3,
                                token.4.clone(),
                                token.5.clone(),
                            )
                        })
                    })
                    .collect();

            info!("Number of filtered tokens: {}", filtered_tokens.len());
            if filtered_tokens.is_empty() {
                return Ok(());
            }

            // Insert or update the filtered tokens
            crate::postgres::insert_or_update_tokens(&db, &filtered_tokens).await?;
        }
    }

    Ok(())
}

pub async fn get_tokens(urls: TokensUrls) -> Result<Option<Vec<TokenList>>> {
    let client = reqwest::Client::new();

    let requests = urls.into_iter().map(|url| {
        let client = client.clone();
        info!("Requesting token list from: {}", url);
        async move {
            let val: serde_json::Value = client.get(url.clone()).send().await?.json().await?;

            if val.get("statusCode").is_none() {
                debug!("Token list successfully retrieved from: {}", url);
                Ok(Some(serde_json::from_value(val).unwrap())) as Result<Option<TokenList>>
            } else {
                debug!("No valid token list found at: {}", url);
                return Ok(None);
            }
        }
    });

    // Execute all requests simultaneously and collect the results
    let results: Vec<Result<Option<TokenList>>> = futures::future::join_all(requests).await;

    let mut tokens = Vec::new();
    for result in results {
        if let Ok(Some(token_list)) = result {
            tokens.push(token_list);
        }
    }
    if tokens.is_empty() {
        info!("No valid token lists found");
        Ok(None)
    } else {
        Ok(Some(tokens))
    }
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

#[derive(Debug, Serialize, Deserialize)]
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
