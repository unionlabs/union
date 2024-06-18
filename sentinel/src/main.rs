use std::{ffi::OsString, fs};

use clap::Parser;
use config::Config;
use context::Context;
use futures::future::try_join_all;
use tokio::{
    signal,
    task::JoinHandle,
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

    let mut handles = vec![];

    handles.extend(context.clone().do_transactions().await);
    handles.extend(context.clone().listen().await);
    handles.extend(context.clone().check_packet_sequences().await);

    // Await all handles and handle panics
    let result: Result<Vec<_>, _> = try_join_all(handles).await;
    if let Err(e) = result {
        tracing::error!("A task has panicked: {:?}", e);
        std::process::exit(1);
    }

    signal::ctrl_c().await.unwrap();
}
