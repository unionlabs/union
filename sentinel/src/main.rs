use std::{ffi::OsString, fs};

use clap::Parser;
use config::Config;
use context::Context;
use tokio::{
    signal,
    time::{interval, Duration},
};

pub mod chains;
pub mod config;
pub mod context;

/// Arguments provided to the top-level command.
#[derive(Debug, Parser, Clone)]
pub struct AppArgs {
    /// The path to the configuration file.
    #[arg(
        long,
        short = 'c',
        global = true,
        default_value = "sentinel-config-testnet.json"
    )]
    pub config: OsString,

    /// The database URL
    #[arg(
        long,
        short = 'd',
        global = true,
        default_value = "postgres://postgres:postgrespassword@127.0.0.1:5432/default" //"postgres://user:password@localhost/dbname"
    )]
    pub database_url: String,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = AppArgs::parse();
    let config: Config = serde_json::from_str(&fs::read_to_string(args.config).unwrap()).unwrap();

    let context = Context::new(config).await.unwrap();

    context.clone().do_transactions().await;
    context.clone().listen().await;
    context.clone().check_packet_sequences().await;

    signal::ctrl_c().await.unwrap();
}
