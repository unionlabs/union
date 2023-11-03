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
    /// Each line of analytics includes: `<channel_id>,<mean_transfer_duration>,<median_transfer_duration>,<max_transfer_duration>,<min_transfer_duration>,<incomplete_transfers>`
    Analyze {
        input_file: String,
        #[arg(long, short = 'o', env, default_value = "zerg-analyze.csv")]
        output: String,
    },
    /// Process the output produced by `rush` or `observe` to get formatted details about whole transactions.
    /// Each line of processed data includes: `<transaction_uuid>,<is_completed>,<arrival_time?>,<duration?>`
    Process {
        input_file: String,
        #[arg(long, short = 'o', env, default_value = "zerg-process.csv")]
        output: String,
    },
    /// Exports the config to stdout.
    PrintConfig,
    /// Conducts stress tests and benchmarks on the configured network.
    /// Each line of observation includes: `<uuid>,<sender_address>,<timestamp>,<event_type>,<chain_id>`
    Rush {
        #[arg(long, short = 'o', env, default_value = "zerg-rush.csv")]
        output: String,
    },
    /// Observes and benchmarks the configured network.
    /// Each line of observation includes: `<uuid>,<sender_address>,<timestamp>,<event_type>,<chain_id>`
    Observe {
        #[arg(long, short = 'o', env, default_value = "zerg-observe.csv")]
        output: String,
    },
}
