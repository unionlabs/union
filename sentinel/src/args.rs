use core::{fmt::Display, str::FromStr};

use clap::{Args, Parser, Subcommand};
use regex::Regex;
use serde::Deserialize;

#[derive(Clone, Parser)]
#[command(version, about, long_about = None)]
/// End-to-end tests and production monitoring scenarios to ensure Union is fully functional.
pub struct Cli {
    /// Command to run.
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Clone, Subcommand)]
pub enum Command {
    Info(InfoCmd),
    Run(RunCmd),
}

/// Display info about varios sentinels.
#[derive(Clone, Args)]
pub struct InfoCmd {}

/// Run the sentinel testsuite.
#[derive(Clone, Args)]
pub struct RunCmd {
    /// Configuration overrides for each individual Sentinel. Pass as an object of key value pairs, where each key is
    /// the name of the sentinel.
    #[arg(short, long, default_value_t)]
    pub overrides: Overrides,

    /// Regex which matches sentinel names. All successful matches will be run.
    #[arg(short, long)]
    pub filter: Option<Regex>,
}

#[derive(Clone, Default, Deserialize)]
#[serde(transparent)]
pub struct Overrides {
    inner: serde_json::Map<String, serde_json::Value>,
}

impl Display for Overrides {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(&self.inner).unwrap())
    }
}

impl Overrides {
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.inner.get(key)
    }
}

impl FromStr for Overrides {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}
