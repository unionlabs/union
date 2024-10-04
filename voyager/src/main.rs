#![feature(trait_alias, try_find)]
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
use pg_queue::PgQueueConfig;
use schemars::gen::{SchemaGenerator, SchemaSettings};
use serde::Serialize;
use serde_json::json;
use serde_utils::Hex;
use tikv_jemallocator::Jemalloc;
use tracing::info;
use tracing_subscriber::EnvFilter;
use unionlabs::{ethereum::ibc_commitment_key, ics24};
use voyager_message::{
    call::FetchBlocks,
    context::{get_plugin_info, ModulesConfig},
    core::ChainId,
    pass::{make_filter, run_filter, FilterResult},
    VoyagerMessage,
};
use voyager_vm::{call, Queue};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use crate::{
    cli::{AppArgs, Command, ConfigCmd, ModuleCmd, PluginCmd, QueueCmd, UtilCmd},
    config::{default_rest_laddr, default_rpc_laddr, Config, VoyagerConfig},
    queue::{AnyQueueConfig, Voyager, VoyagerInitError},
};

#[cfg(not(target_os = "linux"))]
compile_error!(
    "voyager interacts directly with subprocesses and has \
    not been tested on non-linux operating systems."
);

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
    let get_voyager_config = || match &args.config_file_path {
        Some(config_file_path) => read_to_string(config_file_path)
            .map_err(|err| VoyagerError::ConfigFileNotFound {
                path: config_file_path.clone(),
                source: err,
            })
            .and_then(|s| {
                serde_json::from_str::<Config>(&s).map_err(|err| VoyagerError::ConfigFileParse {
                    path: config_file_path.clone(),
                    source: err,
                })
            })
            .map_err(Into::into),
        None => Err::<_, BoxDynError>("config file must be specified".to_owned().into()),
    };

    match args.command {
        Command::Config(cmd) => match cmd {
            ConfigCmd::Print => {
                print_json(&get_voyager_config()?);
            }
            ConfigCmd::Default => print_json(&Config {
                schema: None,
                plugins: vec![],
                modules: ModulesConfig {
                    chain: vec![],
                    consensus: vec![],
                    client: vec![],
                },
                voyager: VoyagerConfig {
                    num_workers: 1,
                    rest_laddr: default_rest_laddr(),
                    rpc_laddr: default_rpc_laddr(),
                    queue: AnyQueueConfig::PgQueue(PgQueueConfig {
                        database_url: "".to_owned(),
                        max_connections: None,
                        min_connections: None,
                        idle_timeout: None,
                        max_lifetime: None,
                    }),
                    optimizer_delay_milliseconds: 100,
                },
            }),
            ConfigCmd::Schema => print_json(
                &SchemaGenerator::new(SchemaSettings::draft2019_09().with(|s| {
                    s.option_nullable = true;
                    s.option_add_null_type = false;
                }))
                .into_root_schema_for::<Config>(),
            ),
        },
        Command::Start => {
            let voyager = Voyager::new(get_voyager_config()?).await?;

            info!("starting relay service");

            voyager.run().await?;
        }
        Command::Plugin(cmd) => match cmd {
            PluginCmd::Interest {
                plugin_name,
                message,
            } => {
                let plugin_config = get_voyager_config()?
                    .plugins
                    .into_iter()
                    .try_find(|plugin_config| {
                        Ok::<_, BoxDynError>(plugin_name == get_plugin_info(plugin_config)?.name)
                    })?
                    .ok_or("plugin not found".to_owned())?;

                let (filter, plugin_name) = make_filter(get_plugin_info(&plugin_config)?)?;

                let result = run_filter(
                    &filter,
                    &plugin_name,
                    serde_json::from_str::<serde_json::Value>(&message)?.into(),
                );

                match result {
                    Ok(FilterResult::Interest) => println!("interest"),
                    Ok(FilterResult::NoInterest) => println!("no interest"),
                    Err(()) => println!("failed"),
                }
            }
            PluginCmd::Info { plugin_name } => {
                let plugin_config = get_voyager_config()?
                    .plugins
                    .into_iter()
                    .try_find(|plugin_config| {
                        Ok::<_, BoxDynError>(plugin_name == get_plugin_info(plugin_config)?.name)
                    })?
                    .ok_or("plugin not found".to_owned())?;

                print_json(&get_plugin_info(&plugin_config)?);
            }
            PluginCmd::Call { plugin_name, args } => match plugin_name {
                Some(module_name) => {
                    let plugin_config = get_voyager_config()?
                        .plugins
                        .into_iter()
                        .try_find(|plugin_config| {
                            Ok::<_, BoxDynError>(
                                module_name == get_plugin_info(plugin_config)?.name,
                            )
                        })?
                        .ok_or("plugin not found".to_owned())?;

                    tokio::process::Command::new(&plugin_config.path)
                        .arg("cmd")
                        .arg("--config")
                        .arg(plugin_config.config.to_string())
                        .args(args)
                        .spawn()?
                        .wait()
                        .await?;
                }
                None => {
                    println!("available plugins and modules");
                    for module_config in get_voyager_config()?.plugins {
                        println!("  {}", get_plugin_info(&module_config)?.name);
                    }
                }
            },
        },
        Command::Module(cmd) => match cmd {
            ModuleCmd::Chain(_) => todo!(),
            ModuleCmd::Consensus(_) => todo!(),
            ModuleCmd::Client(_) => todo!(),
        },
        Command::Query { on, height, path } => {
            let voyager = Voyager::new(get_voyager_config()?).await?;

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
        Command::Queue(cli_msg) => {
            let db = match get_voyager_config()?.voyager.queue {
                AnyQueueConfig::PgQueue(cfg) => {
                    pg_queue::PgQueue::<VoyagerMessage>::new(cfg).await?
                }
                _ => {
                    return Err("no database set in config, queue commands \
                        require the `pg-queue` database backend"
                        .to_string()
                        .into())
                }
            };

            match cli_msg {
                // NOTE: Temporarily disabled until i figure out a better way to implement this with the new queue design
                // cli::QueueCmd::History { id, max_depth } => {
                //     // let results = query_as!(
                //     //     Record,
                //     //     r#"SELECT id as "id!", parent, item as "item!: Item" FROM get_list($1, $2) ORDER BY id ASC"#,
                //     //     id.inner(),
                //     //     max_depth.inner()
                //     // )
                //     // .fetch_all(&db)
                //     // .await
                //     // .unwrap();

                //     // println!("{}", serde_json::to_string_pretty(&results).unwrap());

                //     todo!();
                // }
                QueueCmd::QueryFailed {
                    page,
                    per_page,
                    item_filters,
                    message_filters,
                } => {
                    let record = db
                        .query_failed(page.into(), per_page.into(), item_filters, message_filters)
                        .await?;

                    print_json(&record);
                }
                QueueCmd::QueryFailedById { id } => {
                    let record = db.query_failed_by_id(id.inner()).await?;

                    print_json(&record);
                }
            }
        }
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
            let mut voyager_config = get_voyager_config()?;

            voyager_config.voyager.queue = AnyQueueConfig::InMemory;

            let voyager = Voyager::new(voyager_config).await?;

            let height = voyager
                .context
                .rpc_server
                .query_latest_height(&ChainId::new(chain_id.clone()))
                .await?;

            voyager.shutdown().await;

            print_json(&call::<VoyagerMessage>(FetchBlocks {
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
