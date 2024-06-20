use std::{ffi::OsString, fs};

use clap::Parser;
use config::Config;
use context::Context;
use futures::future::try_join_all;
use tokio::signal;

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
        default_value = "sentinel-config.json"
    )]
    pub config: OsString,

    /// Disable the listen functionality.
    #[arg(long, global = true)]
    pub no_listen: bool,

    /// Disable the interaction functionality.
    #[arg(long, global = true)]
    pub no_interaction: bool,

    /// Perform a single interaction from the provided config.
    #[arg(long, global = true)]
    pub single_interaction: bool,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = AppArgs::parse();
    let config: Config = serde_json::from_str(&fs::read_to_string(args.config).unwrap()).unwrap();

    let context = Context::new(config.clone()).await.unwrap();

    let mut handles = vec![];

    if args.single_interaction {
        if let Some(single_interaction) = config.clone().single_interaction.clone() {
            handles.push(
                context
                    .clone()
                    .do_single_transaction(single_interaction)
                    .await,
            );
        } else {
            tracing::error!(
                "Single transaction flag provided, but no single_transaction configuration found."
            );
            std::process::exit(1);
        }
    } else {
        if !args.no_interaction {
            handles.extend(context.clone().do_transactions().await);
        }

        if !args.no_listen {
            tracing::info!("Starting listen functionality");
            handles.extend(context.clone().listen().await);
            handles.extend(context.clone().check_packet_sequences().await);
        } else {
            tracing::info!("Listen functionality disabled");
        }
    }

    // Await all handles and handle panics
    let result: Result<Vec<_>, _> = try_join_all(handles).await;
    if let Err(e) = result {
        tracing::error!("A task has panicked: {:?}", e);
        std::process::exit(1);
    }

    if !args.single_interaction {
        signal::ctrl_c().await.unwrap();
    }
}
