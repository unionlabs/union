use std::{net::SocketAddr, str::FromStr};

use clap::Parser;
use url::Url;

/// Hubble is state machine observer.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The url to the hasura graphql endpoint.
    #[arg(short, long, env = "HUBBLE_HASURA_URL")]
    pub url: Option<Url>,

    /// The admin secret used to authenticate with hasura.
    #[arg(
        requires("url"),
        group = "datastore",
        required = true,
        short = 's',
        long,
        env = "HUBBLE_HASURA_ADMIN_SECRET"
    )]
    pub hasura_admin_secret: Option<String>,

    /// The database url used to connect with timescaledb.
    #[arg(
        group = "datastore",
        required = true,
        short,
        long,
        env = "HUBBLE_DATABASE_URL"
    )]
    pub database_url: Option<String>,

    /// Indexer configurations to start.
    #[arg(short, long, env = "HUBBLE_INDEXERS")]
    pub indexers: Indexers,

    /// Indexer configurations to start.
    #[arg(short, long, env = "HUBBLE_METRICS_PORT")]
    pub metrics_addr: Option<SocketAddr>,

    /// Fetch the counterparty chain ids for all clients known to hubble.
    #[arg(long)]
    pub fetch_client_chain_ids: bool,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Indexers(Vec<IndexerConfig>);

impl IntoIterator for Indexers {
    type Item = IndexerConfig;

    type IntoIter = std::vec::IntoIter<IndexerConfig>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(tag = "type")]
pub enum IndexerConfig {
    #[serde(rename = "tendermint")]
    Tm(crate::tm::Config),
    #[serde(rename = "ethereum")]
    Eth(crate::eth::Config),
}

impl IndexerConfig {
    pub async fn index(self, db: sqlx::PgPool) -> Result<(), color_eyre::eyre::Report> {
        match self {
            Self::Tm(cfg) => cfg.index(db).await,
            Self::Eth(cfg) => cfg.indexer(db).await?.index().await,
        }
    }
}

impl FromStr for Indexers {
    type Err = color_eyre::eyre::Error;

    fn from_str(item: &str) -> Result<Self, <Self as FromStr>::Err> {
        serde_json::from_str(item).map_err(Into::into)
    }
}
