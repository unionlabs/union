use std::ffi::OsString;

use clap::{Parser, Subcommand};

/// Arguments proved to the top level Zerg command.
#[derive(Debug, Parser, Clone)]
#[command(arg_required_else_help = true)]
pub struct AppArgs {
    /// The path to the configuration file used by Zerg.
    #[arg(
        long,
        short = 'c',
        global = true,
        default_value = "~/.config/zerg/config.json"
    )]
    pub config: OsString,
    /// The subcommand that Zerg will execute.
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand, Clone)]
pub enum Command {
    /// Gather analytics based off of a `process` output.
    Analyze {
        input_file: String,
        #[arg(long, short = 'o', env, default_value = "zerg-analyze.csv")]
        output: String,
    },
    /// Process the output produced by `rush` or `observe` to get formatted details about whole transactions.
    Process {
        input_file: String,
        #[arg(long, short = 'o', env, default_value = "zerg-process.csv")]
        output: String,
    },
    /// Exports the config to stdout.
    PrintConfig,
    /// Conducts stress tests and benchmarks on the configured network.
    Rush {
        #[arg(long, short = 'o', env, default_value = "zerg-rush.csv")]
        output: String,
    },
    /// Observes and benchmarks the configured network.
    Observe {
        #[arg(long, short = 'o', env, default_value = "zerg-observe.csv")]
        output: String,
    },
}
