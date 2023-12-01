use std::ffi::OsString;

use clap::{Parser, Subcommand};

/// Arguments proved to the top level Zerg command.
#[derive(Debug, Parser, Clone)]
#[command(arg_required_else_help = true)]
pub struct AppArgs {
    /// The path to the configuration file used by Zerg.
    #[arg(long, short = 'c', global = true, default_value = "zerg-config.json")]
    pub config: OsString,
    /// The subcommand that Zerg will execute.
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand, Clone)]
pub enum Command {
    /// Gather analytics based off of a `process` output.
    /// Each line of analytics includes: `<channel_id>,<mean_execution_duration>,<median_execution_duration>,<max_execution_duration>,<min_execution_duration>,<mean_finalization_duration>,<median_finalization_duration>,<max_finalization_duration>,<min_finalization_duration>,<incomplete_transfers>,<complete_transfers>,<from>`
    Analyze {
        input_file: String,
        #[arg(long, short = 'o', env, default_value = "zerg-analyze.csv")]
        output: String,
    },
    /// Process the output produced by `rush` or `observe` to get formatted details about whole transactions.
    /// Each line of processed data includes: `<uuid>,<execution_time?>,<execution_duration?><finalization_time?>,<finalization_duration?>`
    Process {
        input_file: String,
        #[arg(long, short = 'o', env, default_value = "zerg-process.csv")]
        output: String,
    },
    /// Exports the config to stdout.
    PrintConfig,
    /// Conducts stress tests and benchmarks on the configured network.
    /// Each line of observation includes: `<uuid>,<address>,<execution_timestamp>,<finalization_timestamp>,<event_type>,<chain_id>`
    Rush {
        #[arg(long, short = 'o', env, default_value = "zerg-rush.csv")]
        output: String,
    },
}
