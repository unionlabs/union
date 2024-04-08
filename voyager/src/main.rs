#![feature(trait_alias, min_exhaustive_patterns)]
// #![warn(clippy::pedantic)]
#![allow(
     // required due to return_position_impl_trait_in_trait false positives
    clippy::manual_async_fn,
    clippy::large_enum_variant,
    clippy::module_name_repetitions,
)]

use std::{error::Error, ffi::OsString, fs::read_to_string, iter, process::ExitCode, sync::Arc};

use chain_utils::{
    cosmos::Cosmos, ethereum::Ethereum, scroll::Scroll, union::Union, wasm::Wasm, AnyChain,
    ChainConfigType, EthereumChainConfig,
};
use clap::Parser;
use queue_msg::QueueMsg;
use sqlx::{query_as, PgPool};
use tikv_jemallocator::Jemalloc;
use tracing_subscriber::EnvFilter;
use unionlabs::ethereum::config::{Mainnet, Minimal, PresetBaseKind};
use voyager_message::VoyagerMessageTypes;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use crate::{
    cli::{any_state_proof_to_json, AppArgs, Command, QueryCmd},
    config::{Config, GetChainError},
    queue::{
        chains_from_config, AnyQueueConfig, PgQueueConfig, RunError, Voyager, VoyagerInitError,
    },
};

pub mod cli;
pub mod config;

pub mod queue;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> ExitCode {
    let args = AppArgs::parse();
    match args.log_format {
        cli::LogFormat::Text => {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .init();
        }
        cli::LogFormat::Json => {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .json()
                .init();
        }
    }

    match do_main(args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            // TODO: Clean this up, it sucks I know

            let e = err.to_string().replace('\n', "\n\t");

            eprintln!("Error:\n\t{e}");

            for e in iter::successors(err.source(), |e| (*e).source()) {
                let e = e.to_string().replace('\n', "\n\t");

                eprintln!("Caused by:\n\t{e}");
            }

            ExitCode::FAILURE
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VoyagerError {
    #[error("unable to read the config file at `{}`", path.to_string_lossy())]
    ConfigFileNotFound {
        path: OsString,
        #[source]
        source: std::io::Error,
    },
    #[error("unable to parse the config file at `{}`", path.to_string_lossy())]
    ConfigFileParse {
        path: OsString,
        #[source]
        source: serde_json::Error,
    },
    #[error("error retrieving a chain from the config")]
    GetChain(#[from] GetChainError),
    #[error("error initializing voyager")]
    Init(#[from] VoyagerInitError),
    #[error("error while running migrations")]
    Migrations(#[from] MigrationsError),
    #[error("fatal error encountered")]
    Run(#[from] RunError),
}

#[derive(Debug, thiserror::Error)]
pub enum MigrationsError {
    #[error("running migrations requires the `pg-queue` queue config")]
    IncorrectQueueConfig,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

#[allow(clippy::too_many_lines)]
// NOTE: This function is a mess, will be cleaned up
async fn do_main(args: cli::AppArgs) -> Result<(), VoyagerError> {
    let voyager_config = read_to_string(&args.config_file_path)
        .map_err(|err| VoyagerError::ConfigFileNotFound {
            path: args.config_file_path.clone(),
            source: err,
        })
        .and_then(|s| {
            serde_json::from_str::<Config>(&s).map_err(|err| VoyagerError::ConfigFileParse {
                path: args.config_file_path,
                source: err,
            })
        })?;

    match args.command {
        Command::RunMigrations => {
            let AnyQueueConfig::PgQueue(PgQueueConfig { database_url, .. }) =
                voyager_config.voyager.queue
            else {
                return Err(VoyagerError::Migrations(
                    MigrationsError::IncorrectQueueConfig,
                ));
            };

            let pool = PgPool::connect(&database_url)
                .await
                .map_err(MigrationsError::Sqlx)?;

            pg_queue::MIGRATOR
                .run(&pool)
                .await
                .map_err(MigrationsError::Migrate)?;
        }
        Command::PrintConfig => {
            println!(
                "{}",
                serde_json::to_string_pretty(&voyager_config)
                    .expect("config serialization is infallible; qed;")
            );
        }
        Command::Relay => {
            let queue = Voyager::new(voyager_config.clone()).await?;

            queue.run().await?;
        }
        Command::Setup(cmd) => match cmd {
            cli::SetupCmd::InitialChannel {
                on,
                counterparty_port_id,
                module_address,
                port_id,
                channel_id,
            } => {
                let chain = voyager_config.get_chain(&on).await?;

                match chain {
                    AnyChain::EthereumMinimal(ethereum) => {
                        chain_utils::ethereum::setup_initial_channel(
                            &ethereum,
                            module_address.into(),
                            channel_id,
                            port_id,
                            counterparty_port_id,
                        )
                        .await;
                    }
                    _ => panic!("Not supported."),
                }
            }
            cli::SetupCmd::Transfer { .. } => {}
        },
        Command::Query {
            on,
            at,
            cmd,
            tracking,
        } => {
            let on = voyager_config.get_chain(&on).await?;
            let tracking = voyager_config
                .chain
                .get(&tracking)
                .expect("chain not found in config")
                .clone();

            let chains = Arc::new(chains_from_config(voyager_config.chain).await.unwrap());

            match cmd {
                QueryCmd::IbcPath(path) => {
                    let json = match (on, &tracking.ty) {
                        (AnyChain::Union(union), ChainConfigType::Cosmos(_)) => {
                            // NOTE: ChainSpec is arbitrary
                            any_state_proof_to_json::<Union, Wasm<Cosmos>>(chains, path, union, at)
                                .await
                        }
                        (
                            AnyChain::Union(union),
                            ChainConfigType::Ethereum(EthereumChainConfig {
                                preset_base: PresetBaseKind::Mainnet,
                                ..
                            }),
                        ) => {
                            any_state_proof_to_json::<Wasm<Union>, Ethereum<Mainnet>>(
                                chains,
                                path,
                                Wasm(union),
                                at,
                            )
                            .await
                        }
                        (AnyChain::Union(union), ChainConfigType::Scroll(_)) => {
                            any_state_proof_to_json::<Wasm<Union>, Scroll>(
                                chains,
                                path,
                                Wasm(union),
                                at,
                            )
                            .await
                        }
                        (
                            AnyChain::Union(union),
                            ChainConfigType::Ethereum(EthereumChainConfig {
                                preset_base: PresetBaseKind::Minimal,
                                ..
                            }),
                        ) => {
                            any_state_proof_to_json::<Wasm<Union>, Ethereum<Minimal>>(
                                chains,
                                path,
                                Wasm(union),
                                at,
                            )
                            .await
                        }
                        (AnyChain::Cosmos(cosmos), ChainConfigType::Union(_)) => {
                            // NOTE: ChainSpec is arbitrary
                            any_state_proof_to_json::<Wasm<Cosmos>, Union>(
                                chains,
                                path,
                                Wasm(cosmos),
                                at,
                            )
                            .await
                        }
                        (AnyChain::EthereumMainnet(ethereum), ChainConfigType::Union(_)) => {
                            any_state_proof_to_json::<Ethereum<Mainnet>, Wasm<Union>>(
                                chains, path, ethereum, at,
                            )
                            .await
                        }

                        (AnyChain::EthereumMinimal(ethereum), ChainConfigType::Union(_)) => {
                            any_state_proof_to_json::<Ethereum<Minimal>, Wasm<Union>>(
                                chains, path, ethereum, at,
                            )
                            .await
                        }

                        (AnyChain::Cosmos(cosmos), ChainConfigType::Cosmos(_)) => {
                            any_state_proof_to_json::<Cosmos, Cosmos>(chains, path, cosmos, at)
                                .await
                        }

                        _ => panic!("unsupported"),
                    };

                    println!("{json}");
                }
            }
        }
        Command::Queue(cli_msg) => {
            let db = match voyager_config.voyager.queue {
                AnyQueueConfig::PgQueue(cfg) => cfg.into_pg_pool().await.unwrap(),
                _ => panic!("no database set in config"),
            };

            type Item = sqlx::types::Json<QueueMsg<VoyagerMessageTypes>>;

            match cli_msg {
                cli::QueueCmd::History { id, max_depth } => {
                    #[derive(Debug, serde::Serialize)]
                    struct Record {
                        id: i64,
                        parent: Option<i64>,
                        item: Item,
                    }

                    let results = query_as!(
                        Record,
                        r#"SELECT id as "id!", parent, item as "item!: Item" FROM get_list($1, $2) ORDER BY id ASC"#,
                        id.inner(),
                        max_depth.inner()
                    )
                    .fetch_all(&db)
                    .await
                    .unwrap();

                    println!("{}", serde_json::to_string_pretty(&results).unwrap());
                }
                cli::QueueCmd::Failed { page, per_page } => {
                    #[derive(Debug, serde::Serialize)]
                    struct Record {
                        id: i64,
                        message: String,
                        item: Item,
                    }

                    let results = query_as!(
                        Record,
                        r#"SELECT id, item as "item: Item", message as "message!" FROM queue WHERE status = 'failed' ORDER BY id ASC LIMIT $1 OFFSET $2"#,
                        per_page.inner(),
                        ((page.inner() - 1) * per_page.inner()),
                    )
                    .fetch_all(&db)
                    .await
                    .unwrap();

                    println!("{}", serde_json::to_string_pretty(&results).unwrap());
                }
            }
        }
    }

    Ok(())
}
