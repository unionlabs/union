#![feature(trait_alias)]
// #![warn(clippy::pedantic)]
#![allow(
     // required due to return_position_impl_trait_in_trait false positives
    clippy::manual_async_fn,
    clippy::large_enum_variant,
    clippy::module_name_repetitions,
)]

use std::{
    ffi::OsString,
    fmt::{Debug, Write},
    fs::read_to_string,
    iter,
    process::ExitCode,
};

use chain_utils::BoxDynError;
use clap::Parser;
use serde::Serialize;
use serde_json::{json, Value};
use serde_utils::Hex;
use tikv_jemallocator::Jemalloc;
use tracing::info;
use tracing_subscriber::EnvFilter;
use unionlabs::{ethereum::ibc_commitment_key, ics24};
use voyager_message::{
    call::FetchBlocks, core::ChainId, module::ChainModuleClient, VoyagerMessage,
};
use voyager_vm::call;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use crate::{
    cli::{AppArgs, Command, UtilCmd},
    config::Config,
    queue::{AnyQueueConfig, Voyager, VoyagerInitError},
};

pub mod cli;
pub mod config;

pub mod queue;

fn main() -> ExitCode {
    let args = AppArgs::parse();

    match args.log_format {
        cli::LogFormat::Text => {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                // .with_span_events(FmtSpan::CLOSE)
                .init();
        }
        cli::LogFormat::Json => {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                // .with_span_events(FmtSpan::CLOSE)
                .json()
                .init();
        }
    }

    let res = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(args.stack_size)
        .build()
        .unwrap()
        .block_on(do_main(args));

    match res {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            let errs = iter::successors(err.source(), |e| (*e).source())
                .enumerate()
                .fold(format!("0: {err}\n"), |mut acc, (i, e)| {
                    writeln!(acc, "{}: {e}", i + 1).unwrap();
                    acc
                });

            eprintln!("{errs}");

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
    #[error("error initializing voyager")]
    Init(#[from] VoyagerInitError),
    #[error("fatal error encountered")]
    Run(#[from] BoxDynError),
    #[error("unable to run command")]
    Command(#[source] BoxDynError),
}

#[allow(clippy::too_many_lines)]
// NOTE: This function is a mess, will be cleaned up
async fn do_main(args: cli::AppArgs) -> Result<(), BoxDynError> {
    let mut voyager_config = read_to_string(&args.config_file_path)
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
        Command::PrintConfig => {
            print_json(&voyager_config);
        }
        Command::Relay => {
            let voyager = Voyager::new(voyager_config.clone()).await?;

            info!("starting relay service");

            voyager.run().await?;
        }
        Command::Module {
            plugin_name: _,
            args: _,
        } => {
            // match plugin_name {
            //     Some(module_name) => {
            //         let module_config = voyager_config
            //             .plugins
            //             .into_iter()
            //             .find(|module_config| {
            //                 module_name == get_module_info(module_config).unwrap().name
            //             })
            //             .expect("module not found");

            //         tokio::process::Command::new(&module_config.path)
            //             .arg("cmd")
            //             .arg("--config")
            //             .arg(module_config.config.to_string())
            //             .args(args)
            //             .spawn()?
            //             .wait()
            //             .await?;
            //     }
            //     None => {
            //         for module_config in voyager_config.plugins {
            //             println!("{}", get_module_info(&module_config).unwrap().name);
            //         }
            //     }
            // }
        }
        Command::Query { on, height, path } => {
            let voyager = Voyager::new(voyager_config.clone()).await?;

            let height = voyager.context.rpc_server.query_height(&on, height).await?;

            let state = voyager
                .context
                .rpc_server
                .query_ibc_state(&on, height, path.clone())
                .await?
                .state;

            let state = match &path {
                ics24::Path::ClientState(path) => {
                    let client_info = voyager
                        .context
                        .rpc_server
                        .client_info(&on, path.client_id.clone())
                        .await?;

                    voyager
                        .context
                        .rpc_server
                        .decode_client_state(
                            &client_info.client_type,
                            &client_info.ibc_interface,
                            serde_json::from_value::<Hex<Vec<u8>>>(state).unwrap().0,
                        )
                        .await?
                }
                ics24::Path::ClientConsensusState(path) => {
                    let client_info = voyager
                        .context
                        .rpc_server
                        .client_info(&on, path.client_id.clone())
                        .await?;

                    voyager
                        .context
                        .rpc_server
                        .decode_consensus_state(
                            &client_info.client_type,
                            &client_info.ibc_interface,
                            serde_json::from_value::<Hex<Vec<u8>>>(state).unwrap().0,
                        )
                        .await?
                }
                _ => state,
            };

            voyager.shutdown().await;

            print_json(&json!({
               "path": path.to_string(),
               "state": state,
            }));
        }
        Command::Queue(_cli_msg) => {
            todo!()
        }
        // Command::Queue(cli_msg) => {
        //     let db = match voyager_config.voyager.queue {
        //         AnyQueueConfig::PgQueue(cfg) => cfg.into_pg_pool().await.unwrap(),
        //         _ => panic!("no database set in config"),
        //     };

        //     type Item = sqlx::types::Json<Op<VoyagerMessage>>;

        //     match cli_msg {
        //         // NOTE: Temporarily disabled until i figure out a better way to implement this with the new queue design
        //         // cli::QueueCmd::History { id, max_depth } => {
        //         //     // let results = query_as!(
        //         //     //     Record,
        //         //     //     r#"SELECT id as "id!", parent, item as "item!: Item" FROM get_list($1, $2) ORDER BY id ASC"#,
        //         //     //     id.inner(),
        //         //     //     max_depth.inner()
        //         //     // )
        //         //     // .fetch_all(&db)
        //         //     // .await
        //         //     // .unwrap();

        //         //     // println!("{}", serde_json::to_string_pretty(&results).unwrap());

        //         //     todo!();
        //         // }
        //         cli::QueueCmd::Failed { page, per_page } => {
        //             #[derive(Debug, serde::Serialize)]
        //             struct Record {
        //                 id: i64,
        //                 message: String,
        //                 item: Item,
        //             }

        //             let results = query_as!(
        //                 Record,
        //                 r#"SELECT id, item as "item: Item", message as "message!" FROM queue WHERE status = 'failed' ORDER BY id ASC LIMIT $1 OFFSET $2"#,
        //                 per_page.inner(),
        //                 ((page.inner() - 1) * per_page.inner()),
        //             )
        //             .fetch_all(&db)
        //             .await
        //             .unwrap();

        //             print_json(&results);
        //         }
        //     }
        // }
        Command::Handshake(_) => todo!(),
        // Command::Handshake(HandshakeCmd {
        //     chain_a,
        //     chain_b,
        //     ty,
        // }) => {
        //     let chain_a = voyager_config.get_chain(&chain_a).await?;
        //     let chain_b = voyager_config.get_chain(&chain_b).await?;

        //     let chains = Arc::new(chains_from_config(voyager_config.chain).await.unwrap());

        //     let all_msgs = match (chain_a, chain_b) {
        //         (AnyChain::Union(union), AnyChain::Cosmos(cosmos)) => {
        //             mk_handshake::<Union, Wasm<Cosmos>>(&union, &Wasm(cosmos), ty, chains).await
        //         }
        //         (AnyChain::Union(union), AnyChain::EthereumMainnet(ethereum)) => {
        //             mk_handshake::<Wasm<Union>, Ethereum<Mainnet>>(
        //                 &Wasm(union),
        //                 &ethereum,
        //                 ty,
        //                 chains,
        //             )
        //             .await
        //         }
        //         (AnyChain::Union(union), AnyChain::EthereumMinimal(ethereum)) => {
        //             mk_handshake::<Wasm<Union>, Ethereum<Minimal>>(
        //                 &Wasm(union),
        //                 &ethereum,
        //                 ty,
        //                 chains,
        //             )
        //             .await
        //         }
        //         (AnyChain::Union(union), AnyChain::Scroll(scroll)) => {
        //             mk_handshake::<Wasm<Union>, Scroll>(&Wasm(union), &scroll, ty, chains).await
        //         }
        //         (AnyChain::Union(union), AnyChain::Arbitrum(scroll)) => {
        //             mk_handshake::<Wasm<Union>, Arbitrum>(&Wasm(union), &scroll, ty, chains).await
        //         }
        //         (AnyChain::Union(union), AnyChain::Berachain(berachain)) => {
        //             mk_handshake::<Wasm<Union>, Berachain>(&Wasm(union), &berachain, ty, chains)
        //                 .await
        //         }
        //         (AnyChain::Cosmos(cosmos), AnyChain::Union(union)) => {
        //             mk_handshake::<Wasm<Cosmos>, Union>(&Wasm(cosmos), &union, ty, chains).await
        //         }
        //         (AnyChain::Cosmos(cosmos_a), AnyChain::Cosmos(cosmos_b)) => {
        //             mk_handshake::<Cosmos, Cosmos>(&cosmos_a, &cosmos_b, ty, chains).await
        //         }
        //         (AnyChain::EthereumMainnet(ethereum), AnyChain::Union(union)) => {
        //             mk_handshake::<Ethereum<Mainnet>, Wasm<Union>>(
        //                 &ethereum,
        //                 &Wasm(union),
        //                 ty,
        //                 chains,
        //             )
        //             .await
        //         }
        //         (AnyChain::EthereumMinimal(ethereum), AnyChain::Union(union)) => {
        //             mk_handshake::<Ethereum<Minimal>, Wasm<Union>>(
        //                 &ethereum,
        //                 &Wasm(union),
        //                 ty,
        //                 chains,
        //             )
        //             .await
        //         }
        //         (AnyChain::Scroll(scroll), AnyChain::Union(union)) => {
        //             mk_handshake::<Scroll, Wasm<Union>>(&scroll, &Wasm(union), ty, chains).await
        //         }
        //         (AnyChain::Arbitrum(scroll), AnyChain::Union(union)) => {
        //             mk_handshake::<Arbitrum, Wasm<Union>>(&scroll, &Wasm(union), ty, chains).await
        //         }
        //         (AnyChain::Berachain(berachain), AnyChain::Union(union)) => {
        //             mk_handshake::<Berachain, Wasm<Union>>(&berachain, &Wasm(union), ty, chains)
        //                 .await
        //         }
        //         _ => panic!("invalid"),
        //     };

        //     print_json(&all_msgs);
        // }
        Command::InitFetch { chain_id } => {
            voyager_config.voyager.queue = AnyQueueConfig::InMemory;

            let voyager = Voyager::new(voyager_config).await?;

            let chain = voyager
                .context
                .rpc_server
                .modules()?
                .chain_module(&ChainId::new(&chain_id))?;

            let height = chain.query_latest_height().await.unwrap();

            voyager.shutdown().await;

            print_json(&call::<VoyagerMessage<Value, Value, Value>>(FetchBlocks {
                chain_id: ChainId::new(chain_id),
                start_height: height,
            }));
        }
        Command::Util(util) => match util {
            UtilCmd::IbcCommitmentKey {
                path,
                commitment_slot,
            } => print_json(&ibc_commitment_key(&path.to_string(), commitment_slot).to_be_hex()),
        },
    }

    Ok(())
}

fn print_json<T: Serialize>(t: &T) {
    println!("{}", serde_json::to_string(&t).unwrap())
}
