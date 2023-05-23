use color_eyre::eyre::bail;
use std::str::FromStr;
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

impl FromStr for LogFormat {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "plain" => Ok(Self::Plain),
            "json" => Ok(Self::Json),
            _ => bail!("unknown log format"),
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
