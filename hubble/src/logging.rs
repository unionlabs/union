use std::str::FromStr;

use thiserror::Error;
use tracing_subscriber::EnvFilter;

pub fn init(log_format: LogFormat) {
    match log_format {
        LogFormat::Plain => {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .init();
        }
        LogFormat::Json => {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .json()
                .init();
        }
    }
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
