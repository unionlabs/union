use std::{fs, net::SocketAddr, path::Path, str::FromStr};

use clap::{builder::ValueParser, ArgGroup, Parser};
use tracing::{info_span, Instrument};

use crate::{indexer, logging::LogFormat};

fn parse_string_or_file_source(input: &str) -> Result<String, String> {
    if let Some(stripped) = input.strip_prefix('@') {
        let path = Path::new(stripped);
        fs::read_to_string(path)
            .map(|s| s.trim().to_string())
            .map_err(|e| format!("Failed to read {}: {}", stripped, e))
    } else {
        Ok(input.to_string())
    }
}

/// Hubble is state machine observer.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(group(
    ArgGroup::new("nats")
        .required(false)
        .requires_all(["nats-url", "nats-username", "nats-password"])
))]
pub struct Args {
    /// The database url used to connect with timescaledb.
    #[arg(
        group = "datastore",
        required = true,
        short,
        long,
        env = "HUBBLE_DATABASE_URL"
    )]
    pub database_url: Option<String>,

    #[command(flatten)]
    pub nats: Option<Nats>,

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

#[derive(Parser, Debug)]
pub struct Nats {
    /// Nats server URL (without credentials)
    #[arg(
        id = "nats-url", 
        long = "nats-url", 
        value_parser = ValueParser::new(parse_string_or_file_source),
        required = false,
    )]
    pub url: String,

    /// Nats username
    #[arg(
        id = "nats-username",
        long = "nats-username",
        value_parser = ValueParser::new(parse_string_or_file_source),
        required = false,
    )]
    pub username: String,

    /// Nats password
    #[arg(
        id = "nats-password",
        long = "nats-password",
        value_parser = ValueParser::new(parse_string_or_file_source),
        required = false,
    )]
    pub password: String,
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
}

impl IndexerConfig {
    pub fn label(&self) -> &str {
        match &self {
            Self::Dummy(cfg) => &cfg.indexer_id,
            Self::Ethereum(cfg) => &cfg.indexer_id,
            Self::Tendermint(cfg) => &cfg.indexer_id,
        }
    }
}

impl IndexerConfig {
    pub async fn index(
        self,
        db: sqlx::PgPool,
        nats: Option<async_nats::jetstream::context::Context>,
    ) -> Result<(), color_eyre::eyre::Report> {
        let label = self.label();

        let initializer_span = info_span!("initializer", label);
        let indexer_span = info_span!("indexer", label);

        match self {
            Self::Dummy(cfg) => {
                cfg.build(db, nats)
                    .instrument(initializer_span)
                    .await?
                    .index()
                    .instrument(indexer_span)
                    .await
            }
            Self::Ethereum(cfg) => {
                cfg.build(db, nats)
                    .instrument(initializer_span)
                    .await?
                    .index()
                    .instrument(indexer_span)
                    .await
            }
            Self::Tendermint(cfg) => {
                cfg.build(db, nats)
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
