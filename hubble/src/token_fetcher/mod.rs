use std::fmt::Display;

mod client;
mod fetcher;
mod postgres;

#[derive(Clone, Debug)]
pub struct TokenSource {
    id: i32,
    source_uri: String,
    name: String,
    logo_uri: Option<String>,
}

impl Display for TokenSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}: {}", self.id, self.source_uri))
    }
}

#[derive(Clone, Debug)]
pub struct TokenRepresentation {
    token_source_id: i32,
    internal_chain_id: i32,
    address: Vec<u8>,
    symbol: String,
    name: String,
    decimals: i32,
    logo_uri: Option<String>,
}

impl Display for TokenRepresentation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}.{}:{}, {} ({}) [{}] - {}",
            self.token_source_id,
            self.internal_chain_id,
            hex::encode(&self.address),
            self.symbol,
            self.name,
            self.decimals,
            self.logo_uri.as_deref().unwrap_or("None")
        ))
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TokenKey {
    internal_chain_id: i32,
    address: Vec<u8>,
}

impl Display for TokenKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}:{}",
            self.internal_chain_id,
            hex::encode(&self.address),
        ))
    }
}

pub async fn update_tokens(db: &sqlx::PgPool) -> color_eyre::Result<()> {
    crate::token_fetcher::fetcher::update_tokens(db).await
}
