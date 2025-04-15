use std::fmt::Display;

use crate::github_fetcher::client::FileContents;

mod client;
mod fetcher;
mod postgres;

#[derive(Clone, Debug)]
pub struct Subscription {
    id: String,
    repo: String,
    path: String,
    branch: String,
    data: Option<Vec<u8>>,
}

impl Display for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} {}:{} ({}) {} bytes",
            self.id,
            self.repo,
            self.path,
            self.branch,
            self.data.as_ref().map_or(0, |data| data.len())
        ))
    }
}

#[derive(Clone, Debug)]
pub struct Attempt {
    subscription_id: String,
    success: bool,
    details: serde_json::Value,
}

impl Display for Attempt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} => success: {} ({})",
            self.subscription_id, self.success, self.details
        ))
    }
}

#[derive(Clone, Debug)]
pub struct Download<'a> {
    subscription_id: String,
    data: &'a FileContents,
    meta: serde_json::Value,
}

impl Display for Download<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} => {} bytes ({})",
            self.subscription_id,
            self.data.0.len(),
            self.meta
        ))
    }
}

pub async fn update_subscriptions(db: &sqlx::PgPool) -> color_eyre::Result<()> {
    crate::github_fetcher::fetcher::update_subscriptions(db).await
}
