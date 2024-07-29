use std::{ffi::OsString, fs, net::SocketAddr};

use axum::{extract::Extension, routing::get, Json, Router};
use clap::Parser;
use config::Config;
use context::Context;
use futures::future::try_join_all;
use tokio::signal;
use tracing_subscriber::EnvFilter;
pub mod chains;
pub mod config;
pub mod context;
use std::{collections::HashMap, sync::Arc};

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Default, clap::ValueEnum, derive_more::Display)]
pub enum LogFormat {
    #[default]
    #[display(fmt = "text")]
    Text,
    #[display(fmt = "json")]
    Json,
}

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

    #[arg(long, short = 'l', env, global = true, default_value_t = LogFormat::default())]
    pub log_format: LogFormat,

    // Perform token distribution before any other operations
    #[arg(long, global = true)]
    pub token_distribution: bool,

    // Perform native token distribution before any other operations
    #[arg(long, global = true)]
    pub native_token_distribution: bool,

    #[arg(
        long,
        global = true,
        default_value = "0.0.0.0:65432",
        value_parser = parse_socket_addr
    )]
    pub laddr: SocketAddr,
}
/// Custom parser function to ensure correct parsing of `SocketAddr`
fn parse_socket_addr(addr: &str) -> Result<SocketAddr, std::net::AddrParseError> {
    addr.parse()
}

async fn balances_handler(Extension(context): Extension<Context>) -> Json<HashMap<String, Value>> {
    // Call the balances method on the context and return the result as JSON.
    let balances = context.balances().await;
    Json(balances)
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args = AppArgs::parse();
    let config: Config = serde_json::from_str(&fs::read_to_string(args.config).unwrap()).unwrap();

    match args.log_format {
        LogFormat::Text => {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .init();
        }
        LogFormat::Json => {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .json()
                .init();
        }
    }

    let context = Context::new(config.clone()).await.unwrap();

    let router = Router::new()
        .route("/balances", get(balances_handler))
        .layer(Extension(context.clone())); // Pass the context as an extension

    // Ensure this server starts running on the given address
    tokio::spawn(async move {
        axum::Server::bind(&args.laddr)
            .serve(router.into_make_service())
            .await
            .unwrap();
    });

    if args.token_distribution || args.native_token_distribution {
        if args.native_token_distribution {
            if let Err(e) = context.perform_native_token_distribution().await.await {
                tracing::error!("Native token distribution task has panicked: {:?}", e);
                std::process::exit(1);
            }
        }

        if args.token_distribution {
            if let Err(e) = context.perform_token_distribution().await.await {
                tracing::error!("Token distribution task has panicked: {:?}", e);
                std::process::exit(1);
            }
        }
    }

    tracing::info!("All token distribution over.");

    let mut handles = vec![];

    if args.single_interaction {
        if let Some(single_interaction) = config.single_interaction.clone() {
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
