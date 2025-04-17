use std::fmt::Display;

use serde_json::Value;

mod client;
mod fetcher;
mod postgres;

#[derive(Clone, Debug)]
pub struct AbiDependency {
    commit: Vec<u8>,
}

impl Display for AbiDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", hex::encode(&self.commit)))
    }
}

#[derive(Clone, Debug)]
pub struct Attempt {
    commit: Vec<u8>,
    success: bool,
    details: serde_json::Value,
}

impl Display for Attempt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} => success: {} ({})",
            hex::encode(&self.commit),
            self.success,
            self.details
        ))
    }
}

#[derive(Clone, Debug)]
pub struct Download<'a> {
    commit: &'a Vec<u8>,
    data: Value,
    meta: serde_json::Value,
}

impl Display for Download<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} => {} ({})",
            hex::encode(self.commit),
            self.data,
            self.meta
        ))
    }
}

pub async fn fetch_abis(db: &sqlx::PgPool) -> color_eyre::Result<()> {
    crate::abi_fetcher::fetcher::fetch_abis(db).await
}
