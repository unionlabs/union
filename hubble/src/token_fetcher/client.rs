use std::fmt::Display;

use reqwest::StatusCode;
use serde::Deserialize;
use tracing::{debug, info};

#[derive(Debug, thiserror::Error)]
pub enum FetcherClientError {
    #[error("error fetching tokens from {0}: {1}")]
    FetchTokensError(String, reqwest::Error),
    #[error("error parsing tokens from {0}: {1}")]
    ParseTokensError(String, reqwest::Error),
    #[error("error response fetching tokens from {0}: {1}")]
    ErrorResponse(String, StatusCode),
}

pub async fn get_tokens(url: &str) -> Result<TokenList, FetcherClientError> {
    let client = reqwest::Client::new();

    info!("reading token list from: {url}");

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| FetcherClientError::FetchTokensError(url.to_string(), error))?;

    if response.status().is_success() {
        debug!("read token list from: {}", url);
        Ok(response
            .json()
            .await
            .map_err(|error| FetcherClientError::ParseTokensError(url.to_string(), error))?)
    } else {
        debug!("no valid token list at: {url} ({})", response.status());
        Err(FetcherClientError::ErrorResponse(
            url.to_string(),
            response.status(),
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenList {
    pub name: String,
    pub timestamp: String,
    pub version: Version,
    pub tokens: Vec<Token>,
    #[serde(rename = "logoURI")]
    pub logo_uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub major: i64,
    pub minor: i64,
    pub patch: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub chain_id: i64,
    #[serde(with = "hex_as_vec")]
    pub address: Vec<u8>,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    #[serde(rename = "logoURI")]
    pub logo_uri: Option<String>,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}, {} ({}) [{}] - {}",
            hex::encode(&self.address),
            self.symbol,
            self.name,
            self.decimals,
            self.logo_uri.as_deref().unwrap_or("None")
        ))
    }
}

mod hex_as_vec {

    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_str: &str = Deserialize::deserialize(deserializer)?;
        if let Some(stripped) = hex_str.strip_prefix("0x") {
            hex::decode(stripped).map_err(serde::de::Error::custom)
        } else {
            Err(serde::de::Error::custom("Hex string must start with '0x'"))
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::token_fetcher::client::{get_tokens, FetcherClientError};

//     #[tokio::test]
//     async fn fetch_url_valid() {
//         let result =
//             get_tokens("https://unionlabs.github.io/token-lists/holesky.17000/tokenlist.json")
//                 .await;
//         assert!(result.is_ok());

//         let tokens = result.unwrap();

//         assert!(!tokens.tokens.is_empty());
//     }

//     #[tokio::test]
//     async fn fetch_url_no_tokenlist() {
//         let result = get_tokens("https://google.com").await;
//         assert!(result.is_err());

//         match result.err().unwrap() {
//             FetcherClientError::ParseTokensError(url, _) => {
//                 assert_eq!(url, "https://google.com");
//             }
//             _ => panic!("expected ParseTokensError"),
//         };
//     }
// }
