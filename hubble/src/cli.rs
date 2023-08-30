use std::str::FromStr;

use clap::Parser;
use url::Url;

/// Hubble is state machine observer.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The url to the hasura graphql endpoint.
    #[arg(short, long)]
    pub hasura: Url,
    /// The admin secret used to authenticate with hasura.
    #[arg(short, long)]
    pub secret: String,
    /// Indexer configurations to start.
    pub indexers: Vec<IndexerConfig>,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(tag = "type")]
pub enum IndexerConfig {
    Tm(crate::tm::Config),
}

impl IndexerConfig {
    pub async fn index(&self, hasura: &Url, secret: &str) -> Result<(), color_eyre::eyre::Report> {
        match self {
            Self::Tm(cfg) => cfg.index(hasura, secret).await,
        }
    }
}

impl FromStr for IndexerConfig {
    type Err = color_eyre::eyre::Error;

    fn from_str(item: &str) -> Result<Self, <Self as FromStr>::Err> {
        serde_json::from_str(item).map_err(Into::into)
    }
}
