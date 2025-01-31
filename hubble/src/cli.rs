use std::{net::SocketAddr, str::FromStr};

use clap::Parser;
use tracing::{info_span, Instrument};
use url::Url;

use crate::{indexer, logging::LogFormat};

/// Hubble is state machine observer.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The url to the hasura graphql endpoint.
    #[arg(short, long, env = "HUBBLE_HASURA_URL")]
    pub url: Option<Url>,

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

    /// The log format for Hubble.
    #[arg(
        global = true,
        short = 'f',
        long,
        env = "HUBBLE_LOG_FORMAT",
        default_value = "json"
    )]
    pub log_format: LogFormat,
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
#[allow(clippy::large_enum_variant)]
pub enum IndexerConfig {
    #[serde(rename = "dummy")]
    Dummy(indexer::dummy::config::Config),
    #[serde(rename = "ethereum")]
    Ethereum(indexer::ethereum::config::Config),
    #[serde(rename = "tendermint")]
    Tendermint(indexer::tendermint::config::Config),
    #[serde(rename = "aptos")]
    Aptos(indexer::aptos::config::Config),
}

impl IndexerConfig {
    pub fn label(&self) -> &str {
        match &self {
            Self::Dummy(cfg) => &cfg.indexer_id,
            Self::Ethereum(cfg) => &cfg.indexer_id,
            Self::Tendermint(cfg) => &cfg.indexer_id,
            Self::Aptos(cfg) => &cfg.indexer_id,
        }
    }
}

impl IndexerConfig {
    pub async fn index(self, db: sqlx::PgPool) -> Result<(), color_eyre::eyre::Report> {
        let label = self.label();

        let initializer_span = info_span!("initializer", label);
        let indexer_span = info_span!("indexer", label);

        match self {
            Self::Dummy(cfg) => {
                cfg.build(db)
                    .instrument(initializer_span)
                    .await?
                    .index()
                    .instrument(indexer_span)
                    .await
            }
            Self::Ethereum(cfg) => {
                cfg.build(db)
                    .instrument(initializer_span)
                    .await?
                    .index()
                    .instrument(indexer_span)
                    .await
            }
            Self::Tendermint(cfg) => {
                cfg.build(db)
                    .instrument(initializer_span)
                    .await?
                    .index()
                    .instrument(indexer_span)
                    .await
            }
            Self::Aptos(cfg) => {
                cfg.build(db)
                    .instrument(initializer_span)
                    .await?
                    .index()
                    .instrument(indexer_span)
                    .await
            }
        }
    }
}

impl FromStr for Indexers {
    type Err = color_eyre::eyre::Error;

    fn from_str(item: &str) -> Result<Self, <Self as FromStr>::Err> {
        serde_json::from_str(item).map_err(Into::into)
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct TokensUrls(Vec<String>);

impl IntoIterator for TokensUrls {
    type Item = String;

    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromStr for TokensUrls {
    type Err = color_eyre::eyre::Error;

    fn from_str(item: &str) -> Result<Self, <Self as FromStr>::Err> {
        serde_json::from_str(item).map_err(Into::into)
    }
}
