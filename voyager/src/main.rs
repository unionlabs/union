#![feature(trait_alias, min_exhaustive_patterns)]
// #![warn(clippy::pedantic)]
#![allow(
     // required due to return_position_impl_trait_in_trait false positives
    clippy::manual_async_fn,
    clippy::large_enum_variant,
    clippy::module_name_repetitions,
)]

use std::{
    collections::HashMap,
    error::Error,
    ffi::OsString,
    fmt::{Debug, Write},
    fs::read_to_string,
    iter,
    process::ExitCode,
};

use chain_utils::{any_chain, keyring::ChainKeyring, AnyChain, Chains, IncorrectChainTypeError};
use clap::Parser;
use queue_msg::{call, BoxDynError};
use serde::Serialize;
use serde_json::Value;
use tikv_jemallocator::Jemalloc;
use tracing_subscriber::EnvFilter;
use unionlabs::ethereum::ibc_commitment_key;
use voyager_message::{call::FetchBlock, plugin::ChainModuleClient, ChainId, VoyagerMessage};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use crate::{
    cli::{AppArgs, Command, UtilCmd},
    config::{Config, GetChainError},
    queue::{Voyager, VoyagerInitError},
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
                .init();
        }
        cli::LogFormat::Json => {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
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
    #[error("error retrieving a chain from the config")]
    GetChain(#[from] GetChainError),
    #[error("error initializing voyager")]
    Init(#[from] VoyagerInitError),
    #[error("error while running migrations")]
    Migrations(#[from] MigrationsError),
    #[error("fatal error encountered")]
    Run(#[from] BoxDynError),
    #[error("unable to run command")]
    Command(#[source] Box<dyn Error>),
    #[error("chain was not of expected type")]
    IncorrectChainType(#[from] IncorrectChainTypeError),
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
            run_migrations(voyager_config).await?;
        }
        Command::PrintConfig => {
            print_json(&voyager_config);
        }
        Command::Relay => {
            let queue = Voyager::new(voyager_config.clone()).await?;

            queue.run().await?;
        }
        Command::Query {
            on: _on_name,
            at: _,
            cmd: _,
            tracking: _,
        } => {
            // query(&mut voyager_config, on_name, tracking, cmd, at).await?;
            todo!()
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
            let voyager = Voyager::new(voyager_config).await?;

            let chain = voyager
                .context
                .chain_module::<Value, Value, Value>(&ChainId::new(&chain_id))
                .unwrap();

            let height = chain.query_latest_height().await.unwrap();

            voyager.shutdown().await;

            print_json(&call::<VoyagerMessage<Value, Value, Value>>(FetchBlock {
                chain_id: ChainId::new(chain_id),
                height,
            }));
        }
        Command::Util(util) => match util {
            UtilCmd::IbcCommitmentKey {
                path,
                commitment_slot,
            } => print_json(&ibc_commitment_key(&path.to_string(), commitment_slot).to_be_hex()),
            UtilCmd::QueryLatestHeight { on: _ } => {
                // let on = voyager_config.get_chain(&on).await?;

                // // TODO: Figure out how to use `any_chain!` here
                // let height = match on {
                //     AnyChain::Union(on) => on
                //         .query_latest_height()
                //         .await
                //         .map_err(|e| VoyagerError::Command(Box::new(e)))?,
                //     AnyChain::Cosmos(on) => on
                //         .query_latest_height()
                //         .await
                //         .map_err(|e| VoyagerError::Command(Box::new(e)))?,
                //     AnyChain::EthereumMainnet(on) => on
                //         .query_latest_height()
                //         .await
                //         .map_err(|e| VoyagerError::Command(Box::new(e)))?,
                //     AnyChain::EthereumMinimal(on) => on
                //         .query_latest_height()
                //         .await
                //         .map_err(|e| VoyagerError::Command(Box::new(e)))?,
                //     AnyChain::Scroll(on) => on
                //         .query_latest_height()
                //         .await
                //         .map_err(|e| VoyagerError::Command(e))?,
                //     AnyChain::Arbitrum(on) => on
                //         .query_latest_height()
                //         .await
                //         .map_err(|e| VoyagerError::Command(e))?,
                //     AnyChain::Berachain(on) => on
                //         .query_latest_height()
                //         .await
                //         .map_err(|e| VoyagerError::Command(Box::new(e)))?,
                // };

                // print_json(&height);

                todo!()
            }
            UtilCmd::QuerySelfConsensusState { on: _, height: _ } => {
                // let on = voyager_config.get_chain(&on).await?;

                // any_chain!(|on| {
                //     let height = match height {
                //         QueryHeight::Latest => on.query_latest_height().await.unwrap(),
                //         QueryHeight::Specific(height) => height,
                //     };

                //     print_json(&on.self_consensus_state(height).await)
                // });

                todo!()
            }
            UtilCmd::QuerySelfClientState { on: _, height: _ } => {
                // let on = voyager_config.get_chain(&on).await?;

                // any_chain!(|on| {
                //     let height = match height {
                //         QueryHeight::Latest => on.query_latest_height().await.unwrap(),
                //         QueryHeight::Specific(height) => height,
                //     };

                //     print_json(&on.self_client_state(height).await)
                // });

                todo!()
            }
            // UtilCmd::Arbitrum(cmd) => match cmd {
            //     ArbitrumCmd::NextNodeNumAtBeaconSlot { on, slot } => print_json(
            //         &voyager_config
            //             .get_chain(&on.to_string())
            //             .await?
            //             .downcast::<Arbitrum>()?
            //             .next_node_num_at_beacon_slot(slot)
            //             .await,
            //     ),
            //     ArbitrumCmd::ExecutionHeightOfBeaconSlot { on, slot } => print_json(
            //         &voyager_config
            //             .get_chain(&on.to_string())
            //             .await?
            //             .downcast::<Arbitrum>()?
            //             .execution_height_of_beacon_slot(slot)
            //             .await,
            //     ),
            // },
            // UtilCmd::Berachain(cmd) => match cmd {
            //     BerachainCmd::ExecutionHeightOfBeaconSlot { on, slot } => print_json(
            //         &voyager_config
            //             .get_chain(&on.to_string())
            //             .await?
            //             .downcast::<Berachain>()?
            //             .execution_height_of_beacon_slot(slot)
            //             .await,
            //     ),
            //     BerachainCmd::ExecutionHeaderAtBeaconSlot { on, slot } => print_json(
            //         &voyager_config
            //             .get_chain(&on.to_string())
            //             .await?
            //             .downcast::<Berachain>()?
            //             .execution_header_at_beacon_slot(slot)
            //             .await,
            //     ),
            //     BerachainCmd::BeaconHeaderAtBeaconSlot { on, slot } => print_json(
            //         &voyager_config
            //             .get_chain(&on.to_string())
            //             .await?
            //             .downcast::<Berachain>()?
            //             .beacon_block_header_at_beacon_slot(slot)
            //             .await,
            //     ),
            // },
            // UtilCmd::Ethereum(cmd) => match cmd {
            //     EthereumCmd::ExecutionHeightOfBeaconSlot { on, slot } => {
            //         print_json(&match voyager_config.get_chain(&on.to_string()).await? {
            //             AnyChain::EthereumMainnet(on) => {
            //                 on.execution_height_of_beacon_slot(slot).await
            //             }
            //             AnyChain::EthereumMinimal(on) => {
            //                 on.execution_height_of_beacon_slot(slot).await
            //             }
            //             chain => panic!(
            //                 "chain type for `{}` not supported for this method",
            //                 chain.chain_id()
            //             ),
            //         })
            //     }
            // },
            _ => todo!(),
        },
        // Command::Signer(cmd) => match cmd {
        //     SignerCmd::Balances { on } => match on {
        //         Some(on) => {
        //             let on = voyager_config.get_chain(&on).await.unwrap();

        //             any_chain!(|on| {
        //                 print_json(&on.balances().await);
        //             });
        //         }
        //         None => {
        //             let chains = chains_from_config(voyager_config.chain).await.unwrap();

        //             let balances = signer_balances(&chains).await;

        //             print_json(&balances);
        //         }
        //     },
        // },
        _ => todo!(),
    }

    Ok(())
}

pub async fn signer_balances(chains: &Chains) -> HashMap<String, serde_json::Value> {
    let mut balances = HashMap::new();

    for (chain_id, chain) in &chains.chains {
        any_chain!(|chain| {
            balances.insert(
                chain_id.to_string(),
                serde_json::to_value(chain.balances().await).unwrap(),
            );
        });
    }

    balances
}

// async fn query(
//     voyager_config: &mut Config,
//     on_name: String,
//     tracking: String,
//     cmd: QueryCmd,
//     at: QueryHeight<unionlabs::ibc::core::client::height::Height>,
// ) -> Result<(), VoyagerError> {
//     let on = voyager_config.get_chain(&on_name).await?;
//     let tracking = voyager_config
//         .chain
//         .get(&tracking)
//         .expect("chain not found in config")
//         .clone();

//     let chains = Arc::new(
//         chains_from_config(
//             [voyager_config
//                 .chain
//                 .remove_entry(&on_name)
//                 .expect("chain is present as it was retrieved previously")]
//             .into_iter()
//             .collect(),
//         )
//         .await
//         .unwrap(),
//     );
//     match cmd {
//         QueryCmd::IbcPath(path) => {
//             let json = match (on, &tracking.ty) {
//                 (AnyChain::Union(union), ChainConfigType::Cosmos(_)) => {
//                     any_state_proof_to_json::<Union, Wasm<Cosmos>>(chains, path, union, at).await
//                 }
//                 (
//                     AnyChain::Union(union),
//                     ChainConfigType::Ethereum(EthereumChainConfig {
//                         preset_base: PresetBaseKind::Mainnet,
//                         ..
//                     }),
//                 ) => {
//                     // NOTE: ChainSpec is arbitrary
//                     any_state_proof_to_json::<Wasm<Union>, Ethereum<Mainnet>>(
//                         chains,
//                         path,
//                         Wasm(union),
//                         at,
//                     )
//                     .await
//                 }
//                 (AnyChain::Union(union), ChainConfigType::Scroll(_)) => {
//                     any_state_proof_to_json::<Wasm<Union>, Scroll>(chains, path, Wasm(union), at)
//                         .await
//                 }
//                 (AnyChain::Union(union), ChainConfigType::Arbitrum(_)) => {
//                     any_state_proof_to_json::<Wasm<Union>, Arbitrum>(chains, path, Wasm(union), at)
//                         .await
//                 }
//                 (
//                     AnyChain::Union(union),
//                     ChainConfigType::Ethereum(EthereumChainConfig {
//                         preset_base: PresetBaseKind::Minimal,
//                         ..
//                     }),
//                 ) => {
//                     any_state_proof_to_json::<Wasm<Union>, Ethereum<Minimal>>(
//                         chains,
//                         path,
//                         Wasm(union),
//                         at,
//                     )
//                     .await
//                 }
//                 (AnyChain::Union(union), ChainConfigType::Berachain(_)) => {
//                     any_state_proof_to_json::<Wasm<Union>, Berachain>(chains, path, Wasm(union), at)
//                         .await
//                 }
//                 (AnyChain::Cosmos(cosmos), ChainConfigType::Union(_)) => {
//                     // NOTE: ChainSpec is arbitrary
//                     any_state_proof_to_json::<Wasm<Cosmos>, Union>(chains, path, Wasm(cosmos), at)
//                         .await
//                 }
//                 (AnyChain::EthereumMainnet(ethereum), ChainConfigType::Union(_)) => {
//                     any_state_proof_to_json::<Ethereum<Mainnet>, Wasm<Union>>(
//                         chains, path, ethereum, at,
//                     )
//                     .await
//                 }

//                 (AnyChain::EthereumMinimal(ethereum), ChainConfigType::Union(_)) => {
//                     any_state_proof_to_json::<Ethereum<Minimal>, Wasm<Union>>(
//                         chains, path, ethereum, at,
//                     )
//                     .await
//                 }

//                 (AnyChain::Scroll(scroll), ChainConfigType::Union(_)) => {
//                     any_state_proof_to_json::<Scroll, Wasm<Union>>(chains, path, scroll, at).await
//                 }

//                 (AnyChain::Arbitrum(arbitrum), ChainConfigType::Union(_)) => {
//                     any_state_proof_to_json::<Arbitrum, Wasm<Union>>(chains, path, arbitrum, at)
//                         .await
//                 }

//                 (AnyChain::Berachain(berachain), ChainConfigType::Union(_)) => {
//                     any_state_proof_to_json::<Berachain, Wasm<Union>>(chains, path, berachain, at)
//                         .await
//                 }
//                 (AnyChain::Cosmos(cosmos), ChainConfigType::Cosmos(_)) => {
//                     any_state_proof_to_json::<Cosmos, Cosmos>(chains, path, cosmos, at).await
//                 }

//                 _ => panic!("unsupported"),
//             };

//             print_json(&json);
//         }
//     };

//     Ok(())
// }

async fn run_migrations(_voyager_config: Config) -> Result<(), VoyagerError> {
    // let AnyQueueConfig::PgQueue(PgQueueConfig { database_url, .. }) = voyager_config.voyager.queue
    // else {
    //     return Err(VoyagerError::Migrations(
    //         MigrationsError::IncorrectQueueConfig,
    //     ));
    // };

    // let pool = PgPool::connect(&database_url)
    //     .await
    //     .map_err(MigrationsError::Sqlx)?;

    // pg_queue::MIGRATOR
    //     .run(&pool)
    //     .await
    //     .map_err(MigrationsError::Migrate)?;

    // Ok(())

    todo!()
}

fn print_json<T: Serialize>(t: &T) {
    println!("{}", serde_json::to_string(&t).unwrap())
}

// async fn mk_handshake<A, B>(
//     a: &A,
//     b: &B,
//     ty: HandshakeType,
//     chains: Arc<Chains>,
// ) -> Op<VoyagerMessage>
// where
//     A: relay_message::ChainExt<ClientId: TryFrom<ClientId, Error: Debug>> + LightClientType<B>,
//     B: relay_message::ChainExt<ClientId: TryFrom<ClientId, Error: Debug>> + LightClientType<A>,

//     relay_message::AnyLightClientIdentified<relay_message::fetch::AnyFetch>:
//         From<relay_message::Identified<A, B, relay_message::fetch::Fetch<A, B>>>,
//     relay_message::AnyLightClientIdentified<relay_message::fetch::AnyFetch>:
//         From<relay_message::Identified<B, A, relay_message::fetch::Fetch<B, A>>>,

//     relay_message::AnyLightClientIdentified<relay_message::data::AnyData>:
//         From<relay_message::Identified<A, B, relay_message::data::Data<A, B>>>,
//     relay_message::AnyLightClientIdentified<relay_message::data::AnyData>:
//         From<relay_message::Identified<B, A, relay_message::data::Data<B, A>>>,

//     relay_message::AnyLightClientIdentified<relay_message::aggregate::AnyAggregate>:
//         From<relay_message::Identified<A, B, relay_message::aggregate::Aggregate<A, B>>>,
//     relay_message::AnyLightClientIdentified<relay_message::aggregate::AnyAggregate>:
//         From<relay_message::Identified<B, A, relay_message::aggregate::Aggregate<B, A>>>,

//     relay_message::AnyLightClientIdentified<relay_message::event::AnyEvent>:
//         From<relay_message::Identified<A, B, relay_message::event::Event<A, B>>>,
//     relay_message::AnyLightClientIdentified<relay_message::event::AnyEvent>:
//         From<relay_message::Identified<B, A, relay_message::event::Event<B, A>>>,

//     relay_message::AnyLightClientIdentified<relay_message::effect::AnyEffect>:
//         From<relay_message::Identified<A, B, relay_message::effect::Effect<A, B>>>,
//     relay_message::AnyLightClientIdentified<relay_message::effect::AnyEffect>:
//         From<relay_message::Identified<B, A, relay_message::effect::Effect<B, A>>>,

//     relay_message::Identified<A, B, relay_message::data::IbcState<NextClientSequencePath, A, B>>:
//         relay_message::use_aggregate::IsAggregateData,
//     relay_message::Identified<B, A, relay_message::data::IbcState<NextClientSequencePath, B, A>>:
//         relay_message::use_aggregate::IsAggregateData,

//     relay_message::Identified<
//         A,
//         B,
//         relay_message::data::IbcState<NextConnectionSequencePath, A, B>,
//     >: relay_message::use_aggregate::IsAggregateData,
//     relay_message::Identified<
//         B,
//         A,
//         relay_message::data::IbcState<NextConnectionSequencePath, B, A>,
//     >: relay_message::use_aggregate::IsAggregateData,
// {
//     let get_next_client_sequences = || async {
//         run_to_completion::<
//             TupleAggregator,
//             RelayMessage,
//             (
//                 relay_message::Identified<A, B, IbcState<NextClientSequencePath, A, B>>,
//                 (
//                     relay_message::Identified<B, A, IbcState<NextClientSequencePath, B, A>>,
//                     (),
//                 ),
//             ),
//             InMemoryQueue<RelayMessage>,
//             _,
//             _,
//         >(
//             TupleAggregator,
//             chains.clone(),
//             (),
//             [
//                 fetch(relay_message::id::<A, B, _>(
//                     a.chain_id(),
//                     relay_message::fetch::FetchState {
//                         at: QueryHeight::Latest,
//                         path: NextClientSequencePath {}.into(),
//                     },
//                 )),
//                 fetch(relay_message::id::<B, A, _>(
//                     b.chain_id(),
//                     relay_message::fetch::FetchState {
//                         at: QueryHeight::Latest,
//                         path: NextClientSequencePath {}.into(),
//                     },
//                 )),
//             ],
//             NormalizeFinal::default(),
//             Pure(NormalizeFinal::default()),
//         )
//         .await
//     };

//     let get_next_connection_sequences = || async {
//         run_to_completion::<
//             TupleAggregator,
//             RelayMessage,
//             (
//                 relay_message::Identified<A, B, IbcState<NextConnectionSequencePath, A, B>>,
//                 (
//                     relay_message::Identified<B, A, IbcState<NextConnectionSequencePath, B, A>>,
//                     (),
//                 ),
//             ),
//             InMemoryQueue<RelayMessage>,
//             _,
//             _,
//         >(
//             TupleAggregator,
//             chains.clone(),
//             (),
//             [
//                 fetch(relay_message::id::<A, B, _>(
//                     a.chain_id(),
//                     relay_message::fetch::FetchState {
//                         at: QueryHeight::Latest,
//                         path: NextConnectionSequencePath {}.into(),
//                     },
//                 )),
//                 fetch(relay_message::id::<B, A, _>(
//                     b.chain_id(),
//                     relay_message::fetch::FetchState {
//                         at: QueryHeight::Latest,
//                         path: NextConnectionSequencePath {}.into(),
//                     },
//                 )),
//             ],
//             NormalizeFinal::default(),
//             Pure(NormalizeFinal::default()),
//         )
//         .await
//     };

//     let mk_create_client_msgs = |client_a_config: serde_json::Value,
//                                  client_b_config: serde_json::Value,
//                                  next_client_sequence_a,
//                                  next_client_sequence_b,
//                                  msgs: Op<RelayMessage>| {
//         let client_config_a =
//             serde_json::from_value::<<A as relay_message::ChainExt>::Config>(client_a_config)
//                 .unwrap();
//         let client_config_b =
//             serde_json::from_value::<<B as relay_message::ChainExt>::Config>(client_b_config)
//                 .unwrap();

//         seq([
//             // create both clients, in parallel
//             conc::<RelayMessage>([
//                 aggregate(
//                     [
//                         fetch(relay_message::id::<B, A, _>(
//                             b.chain_id(),
//                             relay_message::fetch::FetchSelfClientState {
//                                 at: QueryHeight::Latest,
//                                 __marker: PhantomData,
//                             },
//                         )),
//                         fetch(relay_message::id::<B, A, _>(
//                             b.chain_id(),
//                             relay_message::fetch::FetchSelfConsensusState {
//                                 at: QueryHeight::Latest,
//                                 __marker: PhantomData,
//                             },
//                         )),
//                     ],
//                     [],
//                     relay_message::id::<A, B, _>(
//                         a.chain_id(),
//                         relay_message::aggregate::AggregateMsgCreateClient {
//                             config: client_config_a,
//                             __marker: PhantomData,
//                         },
//                     ),
//                 ),
//                 aggregate(
//                     [
//                         fetch(relay_message::id::<A, B, _>(
//                             a.chain_id(),
//                             relay_message::fetch::FetchSelfClientState {
//                                 at: QueryHeight::Latest,
//                                 __marker: PhantomData,
//                             },
//                         )),
//                         fetch(relay_message::id::<A, B, _>(
//                             a.chain_id(),
//                             relay_message::fetch::FetchSelfConsensusState {
//                                 at: QueryHeight::Latest,
//                                 __marker: PhantomData,
//                             },
//                         )),
//                     ],
//                     [],
//                     relay_message::id::<B, A, _>(
//                         b.chain_id(),
//                         relay_message::aggregate::AggregateMsgCreateClient {
//                             config: client_config_b,
//                             __marker: PhantomData,
//                         },
//                     ),
//                 ),
//             ]),
//             // wait for the next client sequence to increase
//             conc([
//                 aggregate(
//                     [fetch(relay_message::id::<A, B, _>(
//                         a.chain_id(),
//                         relay_message::fetch::FetchState {
//                             at: QueryHeight::Latest,
//                             path: NextClientSequencePath {}.into(),
//                         },
//                     ))],
//                     [],
//                     relay_message::id::<A, B, _>(
//                         a.chain_id(),
//                         AggregateWaitForNextClientSequence {
//                             // increment because we wait for the current next sequence to increase
//                             sequence: next_client_sequence_a + 1,
//                             __marker: PhantomData,
//                         },
//                     ),
//                 ),
//                 aggregate(
//                     [fetch(relay_message::id::<B, A, _>(
//                         b.chain_id(),
//                         relay_message::fetch::FetchState {
//                             at: QueryHeight::Latest,
//                             path: NextClientSequencePath {}.into(),
//                         },
//                     ))],
//                     [],
//                     relay_message::id::<B, A, _>(
//                         b.chain_id(),
//                         AggregateWaitForNextClientSequence {
//                             // increment because we wait for the current next sequence to increase
//                             sequence: next_client_sequence_b + 1,
//                             __marker: PhantomData,
//                         },
//                     ),
//                 ),
//             ]),
//             // queue update messages, along with any additional messages to be handled after the clients are created (i.e. connection and channel handshakes)
//             conc(
//                 [
//                     repeat(
//                         None,
//                         seq([
//                             event(relay_message::id::<A, B, _>(
//                                 a.chain_id(),
//                                 relay_message::event::Command::UpdateClient {
//                                     client_id: mk_client_id::<A, B>(next_client_sequence_a),
//                                     __marker: PhantomData,
//                                 },
//                             )),
//                             defer_relative(10),
//                         ]),
//                     ),
//                     repeat(
//                         None,
//                         seq([
//                             event(relay_message::id::<B, A, _>(
//                                 b.chain_id(),
//                                 relay_message::event::Command::UpdateClient {
//                                     client_id: mk_client_id::<B, A>(next_client_sequence_b),
//                                     __marker: PhantomData,
//                                 },
//                             )),
//                             defer_relative(10),
//                         ]),
//                     ),
//                 ]
//                 .into_iter()
//                 .chain([msgs]),
//             ),
//         ])
//     };

//     let mk_connection_msgs = |client_a_id, client_b_id, connection_ordering| {
//         effect::<RelayMessage>(relay_message::id::<A, B, _>(
//             a.chain_id(),
//             relay_message::effect::MsgConnectionOpenInitData(MsgConnectionOpenInit {
//                 client_id: client_a_id,
//                 counterparty: connection::counterparty::Counterparty {
//                     client_id: client_b_id,
//                     connection_id: "".to_string().parse().unwrap(),
//                     prefix: MerklePrefix {
//                         // TODO: Make configurable
//                         key_prefix: b"ibc".to_vec(),
//                     },
//                 },
//                 version: Version {
//                     identifier: "1".into(),
//                     features: connection_ordering,
//                 },
//                 delay_period: unionlabs::DELAY_PERIOD,
//             }),
//         ))
//     };

//     let mk_wait_for_connection_open = |sequence_a: u64, sequence_b: u64| {
//         seq([
//             aggregate(
//                 [fetch(relay_message::id::<A, B, _>(
//                     a.chain_id(),
//                     relay_message::fetch::FetchState {
//                         at: QueryHeight::Latest,
//                         path: NextConnectionSequencePath {}.into(),
//                     },
//                 ))],
//                 [],
//                 relay_message::id::<A, B, _>(
//                     a.chain_id(),
//                     AggregateWaitForNextConnectionSequence {
//                         sequence: sequence_a + 1,
//                         __marker: PhantomData,
//                     },
//                 ),
//             ),
//             aggregate(
//                 [fetch(relay_message::id::<B, A, _>(
//                     b.chain_id(),
//                     relay_message::fetch::FetchState {
//                         at: QueryHeight::Latest,
//                         path: NextConnectionSequencePath {}.into(),
//                     },
//                 ))],
//                 [],
//                 relay_message::id::<B, A, _>(
//                     b.chain_id(),
//                     AggregateWaitForNextConnectionSequence {
//                         sequence: sequence_b + 1,
//                         __marker: PhantomData,
//                     },
//                 ),
//             ),
//             // wait for the connection on chain B to be open, since if B is open then A will also be open
//             aggregate(
//                 [fetch(relay_message::id::<B, A, _>(
//                     b.chain_id(),
//                     relay_message::fetch::FetchState {
//                         at: QueryHeight::Latest,
//                         path: ConnectionPath {
//                             connection_id: format!("connection-{}", sequence_b).parse().unwrap(),
//                         }
//                         .into(),
//                     },
//                 ))],
//                 [],
//                 relay_message::id::<B, A, _>(
//                     b.chain_id(),
//                     AggregateWaitForConnectionOpen {
//                         connection_id: format!("connection-{}", sequence_b).parse().unwrap(),
//                         __marker: PhantomData,
//                     },
//                 ),
//             ),
//         ])
//     };

//     let mk_channel_msgs = |connection_a_id, port_a, port_b, channel_ordering, channel_version| {
//         effect::<RelayMessage>(relay_message::id::<A, B, _>(
//             a.chain_id(),
//             relay_message::effect::MsgChannelOpenInitData {
//                 msg: MsgChannelOpenInit {
//                     port_id: port_a,
//                     channel: Channel {
//                         state: channel::state::State::Init,
//                         ordering: channel_ordering,
//                         counterparty: channel::counterparty::Counterparty {
//                             port_id: port_b,
//                             channel_id: "".to_string(),
//                         },
//                         connection_hops: vec![connection_a_id],
//                         version: channel_version,
//                     },
//                 },
//                 __marker: PhantomData,
//             },
//         ))
//     };

//     let msgs = match ty {
//         HandshakeType::Client {
//             client_a_config,
//             client_b_config,
//         } => {
//             let (sequence_a, (sequence_b, ())) = get_next_client_sequences().await;

//             mk_create_client_msgs(
//                 client_a_config,
//                 client_b_config,
//                 sequence_a.t.state,
//                 sequence_b.t.state,
//                 Op::Noop,
//             )
//         }
//         HandshakeType::ClientConnection {
//             client_a_config,
//             client_b_config,
//             connection_ordering,
//         } => {
//             let (client_sequence_a, (client_sequence_b, ())) = get_next_client_sequences().await;

//             mk_create_client_msgs(
//                 client_a_config,
//                 client_b_config,
//                 client_sequence_a.t.state,
//                 client_sequence_b.t.state,
//                 mk_connection_msgs(
//                     mk_client_id::<A, B>(client_sequence_a.t.state),
//                     mk_client_id::<B, A>(client_sequence_b.t.state),
//                     connection_ordering,
//                 ),
//             )
//         }
//         HandshakeType::ClientConnectionChannel {
//             client_a_config,
//             client_b_config,
//             port_a,
//             port_b,
//             channel_version,
//             connection_ordering,
//             channel_ordering,
//         } => {
//             assert!(connection_ordering.contains(&channel_ordering));

//             let (client_sequence_a, (client_sequence_b, ())) = get_next_client_sequences().await;
//             let (connection_sequence_a, (connection_sequence_b, ())) =
//                 get_next_connection_sequences().await;

//             mk_create_client_msgs(
//                 client_a_config,
//                 client_b_config,
//                 client_sequence_a.t.state,
//                 client_sequence_b.t.state,
//                 seq([
//                     mk_connection_msgs(
//                         mk_client_id::<A, B>(client_sequence_a.t.state),
//                         mk_client_id::<B, A>(client_sequence_b.t.state),
//                         connection_ordering,
//                     ),
//                     mk_wait_for_connection_open(
//                         connection_sequence_a.t.state,
//                         connection_sequence_b.t.state,
//                     ),
//                     mk_channel_msgs(
//                         format!("connection-{}", connection_sequence_a.t.state)
//                             .parse()
//                             .unwrap(),
//                         port_a,
//                         port_b,
//                         channel_ordering,
//                         channel_version,
//                     ),
//                 ]),
//             )
//         }
//         HandshakeType::ConnectionChannel {
//             client_a,
//             client_b,
//             port_a,
//             port_b,
//             channel_version,
//             connection_ordering,
//             channel_ordering,
//         } => {
//             assert!(connection_ordering.contains(&channel_ordering));

//             let (connection_sequence_a, (connection_sequence_b, ())) =
//                 get_next_connection_sequences().await;

//             seq([
//                 mk_connection_msgs(
//                     client_a.try_into().unwrap(),
//                     client_b.try_into().unwrap(),
//                     connection_ordering,
//                 ),
//                 mk_wait_for_connection_open(
//                     connection_sequence_a.t.state,
//                     connection_sequence_b.t.state,
//                 ),
//                 mk_channel_msgs(
//                     format!("connection-{}", connection_sequence_a.t.state)
//                         .parse()
//                         .unwrap(),
//                     port_a,
//                     port_b,
//                     channel_ordering,
//                     channel_version,
//                 ),
//             ])
//         }
//         HandshakeType::Connection {
//             client_a,
//             client_b,
//             connection_ordering,
//         } => mk_connection_msgs(
//             client_a.try_into().unwrap(),
//             client_b.try_into().unwrap(),
//             connection_ordering,
//         ),
//         HandshakeType::Channel {
//             connection_a,
//             port_a,
//             port_b,
//             channel_version,
//             channel_ordering,
//         } => mk_channel_msgs(
//             connection_a,
//             port_a,
//             port_b,
//             channel_ordering,
//             channel_version,
//         ),
//     };

//     VoyagerMessage::from_op(msgs)
// }

// async fn mk_init_fetch<A>(a: &A) -> Op<VoyagerMessage>
// where
//     A: block_message::ChainExt,
//     block_message::AnyChainIdentified<block_message::fetch::AnyFetch>:
//         From<block_message::Identified<A, block_message::fetch::Fetch<A>>>,
// {
//     fetch(VoyagerFetch::Block(
//         block_message::id::<A, _>(
//             a.chain_id(),
//             block_message::fetch::FetchBlock::<A> {
//                 height: a.query_latest_height().await.unwrap(),
//             },
//         )
//         .into(),
//     ))
// }

// fn mk_client_id<Hc: LightClientType<Tr>, Tr: Chain>(sequence: u64) -> ClientIdOf<Hc> {
//     format!(
//         "{}-{}",
//         <Hc as LightClientType<Tr>>::TYPE.identifier_prefix(),
//         sequence
//     )
//     .parse()
//     .unwrap()
// }

// #[tokio::test]
// async fn size() {
//     tracing_subscriber::fmt()
//         .with_env_filter(EnvFilter::from_default_env())
//         .init();

//     // dbg!(mem::size_of::<QueueMsg<VoyagerMessageTypes>>());

//     let msg: QueueMsg<VoyagerMessageTypes> =
//         seq([seq([seq([seq([seq([seq([seq([seq([seq([seq(
//             [seq([seq([seq([seq([seq([queue_msg::noop()])])])])])],
//         )])])])])])])])])]);

//     msg.handle(&chains_from_config(Default::default()).await.unwrap(), 0)
//         .await
//         .unwrap();
// }
