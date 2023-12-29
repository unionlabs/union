#![recursion_limit = "256"]
#![feature(trait_alias, extract_if)]
// #![warn(clippy::pedantic)]
#![allow(
     // required due to return_position_impl_trait_in_trait false positives
    clippy::manual_async_fn,
    clippy::module_name_repetitions
)]
// #![deny(clippy::unwrap_used)]

use std::{error::Error, ffi::OsString, fs::read_to_string, iter, process::ExitCode};

use chain_utils::{cosmos::Cosmos, evm::Evm, union::Union};
use clap::Parser;
use sqlx::PgPool;
use tikv_jemallocator::Jemalloc;
use unionlabs::ethereum::config::{Mainnet, Minimal};
use voyager_message::Wasm;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use crate::{
    chain::AnyChain,
    cli::{any_state_proof_to_json, AppArgs, Command, QueryCmd},
    config::{Config, GetChainError, VoyagerConfig},
    queue::{AnyQueue, AnyQueueConfig, PgQueueConfig, RunError, Voyager, VoyagerInitError},
};

pub mod cli;
pub mod config;

pub mod queue;

pub mod chain;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> ExitCode {
    tracing_subscriber::fmt::init();

    let args = AppArgs::parse();

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
    #[error("unable to read the config file at {}", path.to_string_lossy())]
    ConfigFileNotFound {
        path: OsString,
        #[source]
        source: std::io::Error,
    },
    #[error("unable to parse the config file at {}", path.to_string_lossy())]
    ConfigFileParse {
        path: OsString,
        #[source]
        source: serde_json::Error,
    },
    #[error("error retrieving a chain from the config")]
    GetChain(#[from] GetChainError),
    #[error("error initializing voyager")]
    Init(#[from] VoyagerInitError<AnyQueue>),
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
            serde_json::from_str::<Config<AnyQueue>>(&s).map_err(|err| {
                VoyagerError::ConfigFileParse {
                    path: args.config_file_path,
                    source: err,
                }
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
            // TODO(aeryz): this might go into channel as well, since it's highly coupled with it
            cli::SetupCmd::BindPort {
                on,
                module_address,
                port_id,
            } => {
                let chain = voyager_config.get_chain(&on).await?;

                match chain {
                    AnyChain::EvmMinimal(evm) => {
                        chain_utils::evm::bind_port(&evm, module_address.into(), port_id).await
                    }
                    AnyChain::EvmMainnet(evm) => {
                        chain_utils::evm::bind_port(&evm, module_address.into(), port_id).await
                    }
                    _ => panic!("Not supported"),
                };
            }
            cli::SetupCmd::InitialChannel {
                on,
                counterparty_port_id,
                module_address,
                port_id,
                channel_id,
            } => {
                let chain = voyager_config.get_chain(&on).await?;

                match chain {
                    AnyChain::EvmMinimal(evm) => {
                        chain_utils::evm::setup_initial_channel(
                            &evm,
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
            _ => panic!("not supported"),
        },
        Command::Query {
            on,
            at,
            cmd,
            tracking,
        } => {
            let on = voyager_config.get_chain(&on).await?;
            let tracking = voyager_config.get_chain(&tracking).await?;

            let voyager = Voyager::new(crate::config::Config {
                chain: voyager_config.chain,
                voyager: VoyagerConfig {
                    num_workers: 1,
                    queue: (),
                },
            })
            .await
            .unwrap();

            match cmd {
                QueryCmd::IbcPath(path) => {
                    let json = match (on, tracking) {
                        (AnyChain::Union(union), AnyChain::Cosmos(_)) => {
                            // NOTE: ChainSpec is arbitrary
                            any_state_proof_to_json::<Union, Wasm<Cosmos>>(voyager, path, union, at)
                                .await
                        }
                        (AnyChain::Union(union), AnyChain::EvmMainnet(_)) => {
                            any_state_proof_to_json::<Wasm<Union>, Evm<Mainnet>>(
                                voyager,
                                path,
                                Wasm(union),
                                at,
                            )
                            .await
                        }
                        (AnyChain::Union(union), AnyChain::EvmMinimal(_)) => {
                            any_state_proof_to_json::<Wasm<Union>, Evm<Minimal>>(
                                voyager,
                                path,
                                Wasm(union),
                                at,
                            )
                            .await
                        }
                        (AnyChain::Cosmos(cosmos), AnyChain::Union(_)) => {
                            // NOTE: ChainSpec is arbitrary
                            any_state_proof_to_json::<Wasm<Cosmos>, Union>(
                                voyager,
                                path,
                                Wasm(cosmos),
                                at,
                            )
                            .await
                        }
                        (AnyChain::EvmMainnet(evm), AnyChain::Union(_)) => {
                            any_state_proof_to_json::<Evm<Mainnet>, Wasm<Union>>(
                                voyager, path, evm, at,
                            )
                            .await
                        }

                        (AnyChain::EvmMinimal(evm), AnyChain::Union(_)) => {
                            any_state_proof_to_json::<Evm<Minimal>, Wasm<Union>>(
                                voyager, path, evm, at,
                            )
                            .await
                        }

                        _ => panic!("unsupported"),
                    };

                    println!("{json}");
                }
            }
        }
    }

    Ok(())
}

// commented out for now as this is useful in debugging but not to be run in CI
// #[cfg(test)]
// mod tests {
//     use serde::{Deserialize, Serialize};

//     use crate::{chain::union::EthereumMinimal, msg::msg::MsgUpdateClientData};

//     #[test]
//     fn update_csv() {
//         #[derive(Debug, Serialize, Deserialize)]
//         struct Record {
//             data: String,
//             id: u64,
//         }

//         for record in csv::ReaderBuilder::new()
//             .from_path("/tmp/out.csv")
//             .unwrap()
//             .into_deserialize::<Record>()
//         {
//             let record = record.unwrap();
//             let json =
//                 serde_json::from_str::<MsgUpdateClientData<EthereumMinimal>>(&record.data).unwrap();

//             let update_from = json.update_from;

//             let msg = json.msg.client_message.data;

//             println!(
//                 "id: {}\nupdate_from: {}\nattested beacon slot: {}\nattested execution block number: {}\nfinalized beacon slot: {}\nfinalized execution block number: {}\nnext_sync_committee: {}\n",
//                 record.id,
//                 update_from,
//                 msg.consensus_update.attested_header.beacon.slot,
//                 msg.consensus_update.attested_header.execution.block_number,
//                 msg.consensus_update.finalized_header.beacon.slot,
//                 msg.consensus_update.finalized_header.execution.block_number,
//                 msg.consensus_update.next_sync_committee.is_some(),
//             );
//         }
//     }
// }
