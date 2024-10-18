use std::str::FromStr;

use thiserror::Error;
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

pub fn init(log_format: LogFormat) {
    match log_format {
        LogFormat::Plain => {
            Registry::default()
                .with(tracing_subscriber::fmt::layer())
                .with(EnvFilter::from_default_env())
                .with(ErrorLayer::default())
                .init();
        }
        LogFormat::Json => {
            Registry::default()
                .with(tracing_subscriber::fmt::layer().json())
                .with(EnvFilter::from_default_env())
                .with(ErrorLayer::default())
                .init();
        }
    };
}

#[derive(Debug, Copy, Clone)]
pub enum LogFormat {
    Plain,
    Json,
}

#[derive(Debug, Error)]
#[error("unknown log format {0}")]
pub struct UnknownLogFormatError(String);

impl FromStr for LogFormat {
    type Err = UnknownLogFormatError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "plain" => Ok(Self::Plain),
            "json" => Ok(Self::Json),
            s => Err(UnknownLogFormatError(s.to_owned())),
        }
    }
}
