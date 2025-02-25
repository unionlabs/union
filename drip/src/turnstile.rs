use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Deserialize)]
struct TokenValidateResponse {
    #[serde(rename = "error-codes")]
    error_codes: Option<Vec<String>>,
    success: bool,
    #[allow(dead_code)]
    action: Option<String>,
    #[allow(dead_code)]
    cdata: Option<String>,
}

#[derive(Serialize)]
struct TokenValidateRequest {
    response: String,
    secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub success: bool,
    pub error: Option<String>,
}

pub async fn verify(token: &str, secret: &str) -> Result<ValidationResult> {
    let client = Client::new();

    let request_body = TokenValidateRequest {
        response: token.to_string(),
        secret: secret.to_string(),
    };

    let response = client
        .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    let data: TokenValidateResponse = response.json().await?;

    if !data.success {
        warn!("error verifying turnstile: {:?}", data)
    }

    Ok(ValidationResult {
        // Return the status
        success: data.success,
        // Return the first error if it exists
        error: data.error_codes.map(|codes| codes.join(", ")),
    })
}
