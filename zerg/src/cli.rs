use std::ffi::OsString;

use clap::{Parser, Subcommand};

/// Arguments proved to the top level Zerg command.
#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
pub struct AppArgs {
    /// The path to the configuration file used by Zerg.
    #[arg(
        long,
        short = 'c',
        env,
        global = true,
        default_value = "~/.config/zerg/config.json"
    )]
    pub config_file_path: OsString,
    /// The subcommand that Zerg will execute.
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Exports the config to stdout.
    PrintConfig,
    /// Conducts stress tests and benchmarks on the configured network.
    Rush,
    /// Observes and benchmarks the configured network.
    Observe,
}
