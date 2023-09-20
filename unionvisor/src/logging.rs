use std::str::FromStr;

use thiserror::Error;
use tracing_subscriber::filter::LevelFilter;

pub fn init(log_format: LogFormat, level: LevelFilter) {
    match log_format {
        LogFormat::Plain => {
            tracing_subscriber::fmt()
                .with_max_level(level)
                .with_level(true)
                .init();
        }
        LogFormat::Json => {
            tracing_subscriber::fmt()
                .json()
                .with_max_level(level)
                .with_level(true)
                .init();
        }
    }
}

#[derive(Copy, Clone)]
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

impl LogFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            LogFormat::Json => "json",
            LogFormat::Plain => "plain",
        }
    }
}
