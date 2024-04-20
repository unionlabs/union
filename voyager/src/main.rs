#![feature(trait_alias, min_exhaustive_patterns)]
// #![warn(clippy::pedantic)]
#![allow(
     // required due to return_position_impl_trait_in_trait false positives
    clippy::manual_async_fn,
    clippy::large_enum_variant,
    clippy::module_name_repetitions,
)]

use std::{
    error::Error, ffi::OsString, fs::read_to_string, iter, marker::PhantomData, process::ExitCode,
    sync::Arc,
};

use chain_utils::{
    cosmos::Cosmos, ethereum::Ethereum, scroll::Scroll, union::Union, wasm::Wasm, AnyChain,
    ChainConfigType, Chains, EthereumChainConfig, LightClientType,
};
use clap::Parser;
use queue_msg::{
    aggregate, aggregation::TupleAggregator, conc, defer_relative, effect, event, fetch, repeat,
    run_to_completion, seq, InMemoryQueue, QueueMsg,
};
use relay_message::{
    aggregate::{
        AggregateWaitForConnectionOpen, AggregateWaitForNextClientSequence,
        AggregateWaitForNextConnectionSequence,
    },
    data::IbcState,
    RelayMessageTypes,
};
use sqlx::{query_as, PgPool};
use tikv_jemallocator::Jemalloc;
use tracing_subscriber::EnvFilter;
use unionlabs::{
    ethereum::config::{Mainnet, Minimal, PresetBaseKind},
    ibc::core::{
        channel::{self, channel::Channel, msg_channel_open_init::MsgChannelOpenInit},
        commitment::merkle_prefix::MerklePrefix,
        connection::{self, msg_connection_open_init::MsgConnectionOpenInit, version::Version},
    },
    ics24::{ConnectionPath, NextClientSequencePath, NextConnectionSequencePath},
    traits::{Chain, ClientIdOf},
    QueryHeight,
};
use voyager_message::{FromQueueMsg, VoyagerFetch, VoyagerMessageTypes};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use crate::{
    cli::{any_state_proof_to_json, AppArgs, Command, Handshake, HandshakeType, QueryCmd},
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
        Command::Handshake(Handshake {
            chain_a,
            chain_b,
            ty,
        }) => {
            let chain_a = voyager_config.get_chain(&chain_a).await?;
            let chain_b = voyager_config.get_chain(&chain_b).await?;

            let chains = Arc::new(chains_from_config(voyager_config.chain).await.unwrap());

            let all_msgs = match (chain_a, chain_b) {
                (AnyChain::Union(union), AnyChain::Cosmos(cosmos)) => {
                    mk_handshake::<Union, Wasm<Cosmos>>(&union, &Wasm(cosmos), ty, chains).await
                }
                (AnyChain::Union(union), AnyChain::EthereumMainnet(ethereum)) => {
                    mk_handshake::<Wasm<Union>, Ethereum<Mainnet>>(
                        &Wasm(union),
                        &ethereum,
                        ty,
                        chains,
                    )
                    .await
                }
                (AnyChain::Union(union), AnyChain::EthereumMinimal(ethereum)) => {
                    mk_handshake::<Wasm<Union>, Ethereum<Minimal>>(
                        &Wasm(union),
                        &ethereum,
                        ty,
                        chains,
                    )
                    .await
                }
                (AnyChain::Union(union), AnyChain::Scroll(scroll)) => {
                    mk_handshake::<Wasm<Union>, Scroll>(&Wasm(union), &scroll, ty, chains).await
                }
                (AnyChain::Cosmos(cosmos), AnyChain::Union(union)) => {
                    mk_handshake::<Wasm<Cosmos>, Union>(&Wasm(cosmos), &union, ty, chains).await
                }
                (AnyChain::Cosmos(cosmos_a), AnyChain::Cosmos(cosmos_b)) => {
                    mk_handshake::<Cosmos, Cosmos>(&cosmos_a, &cosmos_b, ty, chains).await
                }
                (AnyChain::EthereumMainnet(ethereum), AnyChain::Union(union)) => {
                    mk_handshake::<Ethereum<Mainnet>, Wasm<Union>>(
                        &ethereum,
                        &Wasm(union),
                        ty,
                        chains,
                    )
                    .await
                }
                (AnyChain::EthereumMinimal(ethereum), AnyChain::Union(union)) => {
                    mk_handshake::<Ethereum<Minimal>, Wasm<Union>>(
                        &ethereum,
                        &Wasm(union),
                        ty,
                        chains,
                    )
                    .await
                }
                (AnyChain::Scroll(scroll), AnyChain::Union(union)) => {
                    mk_handshake::<Scroll, Wasm<Union>>(&scroll, &Wasm(union), ty, chains).await
                }
                _ => panic!("invalid"),
            };

            println!("{}", serde_json::to_string(&all_msgs).unwrap());
        }
        Command::InitFetch { on } => {
            let on = voyager_config.get_chain(&on).await?;

            let msg = match on {
                AnyChain::Union(on) => mk_init_fetch::<Union>(&on).await,
                AnyChain::Cosmos(on) => mk_init_fetch::<Cosmos>(&on).await,
                AnyChain::EthereumMainnet(on) => mk_init_fetch::<Ethereum<Mainnet>>(&on).await,
                AnyChain::EthereumMinimal(on) => mk_init_fetch::<Ethereum<Minimal>>(&on).await,
                AnyChain::Scroll(on) => mk_init_fetch::<Scroll>(&on).await,
            };

            println!("{}", serde_json::to_string(&msg).unwrap());
        }
    }

    Ok(())
}

async fn mk_handshake<A, B>(
    a: &A,
    b: &B,
    ty: HandshakeType,
    chains: Arc<Chains>,
) -> QueueMsg<VoyagerMessageTypes>
where
    A: relay_message::ChainExt + LightClientType<B>,
    B: relay_message::ChainExt + LightClientType<A>,

    relay_message::AnyLightClientIdentified<relay_message::fetch::AnyFetch>:
        From<relay_message::Identified<A, B, relay_message::fetch::Fetch<A, B>>>,
    relay_message::AnyLightClientIdentified<relay_message::fetch::AnyFetch>:
        From<relay_message::Identified<B, A, relay_message::fetch::Fetch<B, A>>>,

    relay_message::AnyLightClientIdentified<relay_message::data::AnyData>:
        From<relay_message::Identified<A, B, relay_message::data::Data<A, B>>>,
    relay_message::AnyLightClientIdentified<relay_message::data::AnyData>:
        From<relay_message::Identified<B, A, relay_message::data::Data<B, A>>>,

    relay_message::AnyLightClientIdentified<relay_message::aggregate::AnyAggregate>:
        From<relay_message::Identified<A, B, relay_message::aggregate::Aggregate<A, B>>>,
    relay_message::AnyLightClientIdentified<relay_message::aggregate::AnyAggregate>:
        From<relay_message::Identified<B, A, relay_message::aggregate::Aggregate<B, A>>>,

    relay_message::AnyLightClientIdentified<relay_message::event::AnyEvent>:
        From<relay_message::Identified<A, B, relay_message::event::Event<A, B>>>,
    relay_message::AnyLightClientIdentified<relay_message::event::AnyEvent>:
        From<relay_message::Identified<B, A, relay_message::event::Event<B, A>>>,

    relay_message::AnyLightClientIdentified<relay_message::effect::AnyEffect>:
        From<relay_message::Identified<A, B, relay_message::effect::Effect<A, B>>>,
    relay_message::AnyLightClientIdentified<relay_message::effect::AnyEffect>:
        From<relay_message::Identified<B, A, relay_message::effect::Effect<B, A>>>,

    relay_message::Identified<A, B, relay_message::data::IbcState<NextClientSequencePath, A, B>>:
        relay_message::use_aggregate::IsAggregateData,
    relay_message::Identified<B, A, relay_message::data::IbcState<NextClientSequencePath, B, A>>:
        relay_message::use_aggregate::IsAggregateData,

    relay_message::Identified<
        A,
        B,
        relay_message::data::IbcState<NextConnectionSequencePath, A, B>,
    >: relay_message::use_aggregate::IsAggregateData,
    relay_message::Identified<
        B,
        A,
        relay_message::data::IbcState<NextConnectionSequencePath, B, A>,
    >: relay_message::use_aggregate::IsAggregateData,
{
    let get_next_client_sequences = || async {
        run_to_completion::<
            TupleAggregator,
            RelayMessageTypes,
            (
                relay_message::Identified<A, B, IbcState<NextClientSequencePath, A, B>>,
                (
                    relay_message::Identified<B, A, IbcState<NextClientSequencePath, B, A>>,
                    (),
                ),
            ),
            InMemoryQueue<RelayMessageTypes>,
        >(
            TupleAggregator,
            chains.clone(),
            (),
            [
                fetch(relay_message::id::<A, B, _>(
                    a.chain_id(),
                    relay_message::fetch::FetchState {
                        at: QueryHeight::Latest,
                        path: NextClientSequencePath {}.into(),
                    },
                )),
                fetch(relay_message::id::<B, A, _>(
                    b.chain_id(),
                    relay_message::fetch::FetchState {
                        at: QueryHeight::Latest,
                        path: NextClientSequencePath {}.into(),
                    },
                )),
            ],
        )
        .await
    };

    let get_next_connection_sequences = || async {
        run_to_completion::<
            TupleAggregator,
            RelayMessageTypes,
            (
                relay_message::Identified<A, B, IbcState<NextConnectionSequencePath, A, B>>,
                (
                    relay_message::Identified<B, A, IbcState<NextConnectionSequencePath, B, A>>,
                    (),
                ),
            ),
            InMemoryQueue<RelayMessageTypes>,
        >(
            TupleAggregator,
            chains.clone(),
            (),
            [
                fetch(relay_message::id::<A, B, _>(
                    a.chain_id(),
                    relay_message::fetch::FetchState {
                        at: QueryHeight::Latest,
                        path: NextConnectionSequencePath {}.into(),
                    },
                )),
                fetch(relay_message::id::<B, A, _>(
                    b.chain_id(),
                    relay_message::fetch::FetchState {
                        at: QueryHeight::Latest,
                        path: NextConnectionSequencePath {}.into(),
                    },
                )),
            ],
        )
        .await
    };

    let mk_create_client_msgs =
        |client_a_config: serde_json::Value,
         client_b_config: serde_json::Value,
         next_client_sequence_a,
         next_client_sequence_b,
         msgs: QueueMsg<RelayMessageTypes>| {
            let client_config_a =
                serde_json::from_value::<<A as relay_message::ChainExt>::Config>(client_a_config)
                    .unwrap();
            let client_config_b =
                serde_json::from_value::<<B as relay_message::ChainExt>::Config>(client_b_config)
                    .unwrap();

            seq([
                // create both clients, in parallel
                conc::<RelayMessageTypes>([
                    aggregate(
                        [
                            fetch(relay_message::id::<B, A, _>(
                                b.chain_id(),
                                relay_message::fetch::FetchSelfClientState {
                                    at: QueryHeight::Latest,
                                    __marker: PhantomData,
                                },
                            )),
                            fetch(relay_message::id::<B, A, _>(
                                b.chain_id(),
                                relay_message::fetch::FetchSelfConsensusState {
                                    at: QueryHeight::Latest,
                                    __marker: PhantomData,
                                },
                            )),
                        ],
                        [],
                        relay_message::id::<A, B, _>(
                            a.chain_id(),
                            relay_message::aggregate::AggregateMsgCreateClient {
                                config: client_config_a,
                                __marker: PhantomData,
                            },
                        ),
                    ),
                    aggregate(
                        [
                            fetch(relay_message::id::<A, B, _>(
                                a.chain_id(),
                                relay_message::fetch::FetchSelfClientState {
                                    at: QueryHeight::Latest,
                                    __marker: PhantomData,
                                },
                            )),
                            fetch(relay_message::id::<A, B, _>(
                                a.chain_id(),
                                relay_message::fetch::FetchSelfConsensusState {
                                    at: QueryHeight::Latest,
                                    __marker: PhantomData,
                                },
                            )),
                        ],
                        [],
                        relay_message::id::<B, A, _>(
                            b.chain_id(),
                            relay_message::aggregate::AggregateMsgCreateClient {
                                config: client_config_b,
                                __marker: PhantomData,
                            },
                        ),
                    ),
                ]),
                // wait for the next client sequence to increase
                conc([
                    aggregate(
                        [fetch(relay_message::id::<A, B, _>(
                            a.chain_id(),
                            relay_message::fetch::FetchState {
                                at: QueryHeight::Latest,
                                path: NextClientSequencePath {}.into(),
                            },
                        ))],
                        [],
                        relay_message::id::<A, B, _>(
                            a.chain_id(),
                            AggregateWaitForNextClientSequence {
                                // increment because we wait for the current next sequence to increase
                                sequence: next_client_sequence_a + 1,
                                __marker: PhantomData,
                            },
                        ),
                    ),
                    aggregate(
                        [fetch(relay_message::id::<B, A, _>(
                            b.chain_id(),
                            relay_message::fetch::FetchState {
                                at: QueryHeight::Latest,
                                path: NextClientSequencePath {}.into(),
                            },
                        ))],
                        [],
                        relay_message::id::<B, A, _>(
                            b.chain_id(),
                            AggregateWaitForNextClientSequence {
                                // increment because we wait for the current next sequence to increase
                                sequence: next_client_sequence_b + 1,
                                __marker: PhantomData,
                            },
                        ),
                    ),
                ]),
                // queue update messages, along with any additional messages to be handled after the clients are created (i.e. connection and channel handshakes)
                conc(
                    [
                        repeat(
                            None,
                            seq([
                                event(relay_message::id::<A, B, _>(
                                    a.chain_id(),
                                    relay_message::event::Command::UpdateClient {
                                        client_id: mk_client_id::<A, B>(next_client_sequence_a),
                                        __marker: PhantomData,
                                    },
                                )),
                                defer_relative(10),
                            ]),
                        ),
                        repeat(
                            None,
                            seq([
                                event(relay_message::id::<B, A, _>(
                                    b.chain_id(),
                                    relay_message::event::Command::UpdateClient {
                                        client_id: mk_client_id::<B, A>(next_client_sequence_b),
                                        __marker: PhantomData,
                                    },
                                )),
                                defer_relative(10),
                            ]),
                        ),
                    ]
                    .into_iter()
                    .chain([msgs]),
                ),
            ])
        };

    let mk_connection_msgs = |client_a_id, client_b_id, connection_ordering| {
        effect::<RelayMessageTypes>(relay_message::id::<A, B, _>(
            a.chain_id(),
            relay_message::effect::MsgConnectionOpenInitData(MsgConnectionOpenInit {
                client_id: client_a_id,
                counterparty: connection::counterparty::Counterparty {
                    client_id: client_b_id,
                    connection_id: "".to_string().parse().unwrap(),
                    prefix: MerklePrefix {
                        key_prefix: b"ibc".to_vec(),
                    },
                },
                version: Version {
                    identifier: "1".into(),
                    features: connection_ordering,
                },
                delay_period: unionlabs::DELAY_PERIOD,
            }),
        ))
    };

    let mk_wait_for_connection_open = |sequence_a: u64, sequence_b: u64| {
        seq([
            aggregate(
                [fetch(relay_message::id::<A, B, _>(
                    a.chain_id(),
                    relay_message::fetch::FetchState {
                        at: QueryHeight::Latest,
                        path: NextConnectionSequencePath {}.into(),
                    },
                ))],
                [],
                relay_message::id::<A, B, _>(
                    a.chain_id(),
                    AggregateWaitForNextConnectionSequence {
                        sequence: sequence_a + 1,
                        __marker: PhantomData,
                    },
                ),
            ),
            aggregate(
                [fetch(relay_message::id::<B, A, _>(
                    b.chain_id(),
                    relay_message::fetch::FetchState {
                        at: QueryHeight::Latest,
                        path: NextConnectionSequencePath {}.into(),
                    },
                ))],
                [],
                relay_message::id::<B, A, _>(
                    b.chain_id(),
                    AggregateWaitForNextConnectionSequence {
                        sequence: sequence_b + 1,
                        __marker: PhantomData,
                    },
                ),
            ),
            // wait for the connection on chain B to be open, since if B is open then A will also be open
            aggregate(
                [fetch(relay_message::id::<B, A, _>(
                    b.chain_id(),
                    relay_message::fetch::FetchState {
                        at: QueryHeight::Latest,
                        path: ConnectionPath {
                            connection_id: format!("connection-{}", sequence_b).parse().unwrap(),
                        }
                        .into(),
                    },
                ))],
                [],
                relay_message::id::<B, A, _>(
                    b.chain_id(),
                    AggregateWaitForConnectionOpen {
                        connection_id: format!("connection-{}", sequence_b).parse().unwrap(),
                        __marker: PhantomData,
                    },
                ),
            ),
        ])
    };

    let mk_channel_msgs = |connection_a_id, port_a, port_b, channel_ordering, channel_version| {
        effect::<RelayMessageTypes>(relay_message::id::<A, B, _>(
            a.chain_id(),
            relay_message::effect::MsgChannelOpenInitData {
                msg: MsgChannelOpenInit {
                    port_id: port_a,
                    channel: Channel {
                        state: channel::state::State::Init,
                        ordering: channel_ordering,
                        counterparty: channel::counterparty::Counterparty {
                            port_id: port_b,
                            channel_id: "".to_string(),
                        },
                        connection_hops: vec![connection_a_id],
                        version: channel_version,
                    },
                },
                __marker: PhantomData,
            },
        ))
    };

    let msgs = match ty {
        HandshakeType::Client {
            client_a_config,
            client_b_config,
        } => {
            let (sequence_a, (sequence_b, ())) = get_next_client_sequences().await;

            mk_create_client_msgs(
                client_a_config,
                client_b_config,
                sequence_a.t.state,
                sequence_b.t.state,
                QueueMsg::Noop,
            )
        }
        HandshakeType::ClientConnection {
            client_a_config,
            client_b_config,
            connection_ordering,
        } => {
            let (client_sequence_a, (client_sequence_b, ())) = get_next_client_sequences().await;

            mk_create_client_msgs(
                client_a_config,
                client_b_config,
                client_sequence_a.t.state,
                client_sequence_b.t.state,
                mk_connection_msgs(
                    mk_client_id::<A, B>(client_sequence_a.t.state),
                    mk_client_id::<B, A>(client_sequence_b.t.state),
                    connection_ordering,
                ),
            )
        }
        HandshakeType::ClientConnectionChannel {
            client_a_config,
            client_b_config,
            port_a,
            port_b,
            channel_version,
            connection_ordering,
            channel_ordering,
        } => {
            assert!(connection_ordering.contains(&channel_ordering));

            let (client_sequence_a, (client_sequence_b, ())) = get_next_client_sequences().await;
            let (connection_sequence_a, (connection_sequence_b, ())) =
                get_next_connection_sequences().await;

            mk_create_client_msgs(
                client_a_config,
                client_b_config,
                client_sequence_a.t.state,
                client_sequence_b.t.state,
                seq([
                    mk_connection_msgs(
                        mk_client_id::<A, B>(client_sequence_a.t.state),
                        mk_client_id::<B, A>(client_sequence_b.t.state),
                        connection_ordering,
                    ),
                    mk_wait_for_connection_open(
                        connection_sequence_a.t.state,
                        connection_sequence_b.t.state,
                    ),
                    mk_channel_msgs(
                        format!("connection-{}", connection_sequence_a.t.state)
                            .parse()
                            .unwrap(),
                        port_a,
                        port_b,
                        channel_ordering,
                        channel_version,
                    ),
                ]),
            )
        }
        HandshakeType::ConnectionChannel {
            client_a,
            client_b,
            port_a,
            port_b,
            channel_version,
            connection_ordering,
            channel_ordering,
        } => {
            assert!(connection_ordering.contains(&channel_ordering));

            let (connection_sequence_a, (connection_sequence_b, ())) =
                get_next_connection_sequences().await;

            seq([
                mk_connection_msgs(
                    // NOTE: We do this so we don't have to bound a very nested type to Debug, will not be an issue once we have associated type bounds
                    client_a.try_into().ok().unwrap(),
                    client_b.try_into().ok().unwrap(),
                    connection_ordering,
                ),
                mk_wait_for_connection_open(
                    connection_sequence_a.t.state,
                    connection_sequence_b.t.state,
                ),
                mk_channel_msgs(
                    format!("connection-{}", connection_sequence_a.t.state)
                        .parse()
                        .unwrap(),
                    port_a,
                    port_b,
                    channel_ordering,
                    channel_version,
                ),
            ])
        }
        HandshakeType::Connection {
            client_a,
            client_b,
            connection_ordering,
        } => mk_connection_msgs(
            // NOTE: We do this so we don't have to bound a very nested type to Debug, will not be an issue once we have associated type bounds
            client_a.try_into().ok().unwrap(),
            client_b.try_into().ok().unwrap(),
            connection_ordering,
        ),
        HandshakeType::Channel {
            connection_a,
            port_a,
            port_b,
            channel_version,
            channel_ordering,
        } => mk_channel_msgs(
            connection_a,
            port_a,
            port_b,
            channel_ordering,
            channel_version,
        ),
    };

    VoyagerMessageTypes::from_queue_msg(msgs)
}

async fn mk_init_fetch<A>(a: &A) -> QueueMsg<VoyagerMessageTypes>
where
    A: block_message::ChainExt,
    block_message::AnyChainIdentified<block_message::fetch::AnyFetch>:
        From<block_message::Identified<A, block_message::fetch::Fetch<A>>>,
{
    fetch(VoyagerFetch::Block(
        block_message::id::<A, _>(
            a.chain_id(),
            block_message::fetch::FetchBlock::<A> {
                height: a.query_latest_height().await.unwrap(),
            },
        )
        .into(),
    ))
}

fn mk_client_id<Hc: LightClientType<Tr>, Tr: Chain>(sequence: u64) -> ClientIdOf<Hc> {
    format!(
        "{}-{}",
        <Hc as LightClientType<Tr>>::TYPE.identifier_prefix(),
        sequence
    )
    .parse()
    .unwrap()
}
