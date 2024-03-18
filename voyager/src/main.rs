#![feature(trait_alias)]
// #![warn(clippy::pedantic)]
#![allow(
     // required due to return_position_impl_trait_in_trait false positives
    clippy::manual_async_fn,
    clippy::module_name_repetitions,
    clippy::large_enum_variant
)]
// #![deny(clippy::unwrap_used)]

use std::{error::Error, ffi::OsString, fs::read_to_string, iter, process::ExitCode, sync::Arc};

use chain_utils::{cosmos::Cosmos, evm::Evm, union::Union, wasm::Wasm};
use clap::Parser;
use sqlx::PgPool;
use tikv_jemallocator::Jemalloc;
use tracing_subscriber::EnvFilter;
use unionlabs::ethereum::config::{Mainnet, Minimal, PresetBaseKind};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use crate::{
    chain::AnyChain,
    cli::{any_state_proof_to_json, AppArgs, Command, QueryCmd},
    config::{ChainConfigType, Config, EvmChainConfig, GetChainError},
    queue::{
        chains_from_config, AnyQueueConfig, PgQueueConfig, RunError, Voyager, VoyagerInitError,
    },
};

pub mod cli;
pub mod config;

pub mod queue;

pub mod chain;

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
                            ChainConfigType::Evm(EvmChainConfig {
                                preset_base: PresetBaseKind::Mainnet,
                                ..
                            }),
                        ) => {
                            any_state_proof_to_json::<Wasm<Union>, Evm<Mainnet>>(
                                chains,
                                path,
                                Wasm(union),
                                at,
                            )
                            .await
                        }
                        (
                            AnyChain::Union(union),
                            ChainConfigType::Evm(EvmChainConfig {
                                preset_base: PresetBaseKind::Minimal,
                                ..
                            }),
                        ) => {
                            any_state_proof_to_json::<Wasm<Union>, Evm<Minimal>>(
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
                        (AnyChain::EvmMainnet(evm), ChainConfigType::Union(_)) => {
                            any_state_proof_to_json::<Evm<Mainnet>, Wasm<Union>>(
                                chains, path, evm, at,
                            )
                            .await
                        }

                        (AnyChain::EvmMinimal(evm), ChainConfigType::Union(_)) => {
                            any_state_proof_to_json::<Evm<Minimal>, Wasm<Union>>(
                                chains, path, evm, at,
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

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use block_message::BlockPollingTypes;
    use chain_utils::{cosmos::Cosmos, evm::Evm, union::Union, wasm::Wasm};
    use hex_literal::hex;
    use queue_msg::{
        aggregate, defer_relative, event, fetch, msg, repeat, seq, QueueMsg, QueueMsgTypes,
    };
    use relay_message::{
        aggregate::AggregateCreateClient,
        chain_impls::{
            cosmos_sdk::fetch::{AbciQueryType, FetchAbciQuery},
            evm::EvmConfig,
            union::UnionFetch,
        },
        event::IbcEvent,
        fetch::{FetchSelfClientState, FetchSelfConsensusState, LightClientSpecificFetch},
        msg::{MsgChannelOpenInitData, MsgConnectionOpenInitData},
        RelayerMsgTypes, WasmConfig,
    };
    use unionlabs::{
        ethereum::config::Minimal,
        events::ConnectionOpenTry,
        hash::{H160, H256},
        ibc::core::{
            channel::{
                self, channel::Channel, msg_channel_open_init::MsgChannelOpenInit, order::Order,
            },
            commitment::merkle_prefix::MerklePrefix,
            connection::{self, msg_connection_open_init::MsgConnectionOpenInit, version::Version},
        },
        proof,
        uint::U256,
        QueryHeight, DELAY_PERIOD,
    };

    use crate::queue::{FromQueueMsg, VoyagerMessageTypes};

    macro_rules! parse {
        ($expr:expr) => {
            $expr.parse().unwrap()
        };
    }

    #[test]
    fn msg_serde() {
        let union_chain_id: String = parse!("union-devnet-1");
        let eth_chain_id: U256 = parse!("32382");
        let cosmos_chain_id: String = parse!("simd-devnet-1");

        println!("---------------------------------------");
        println!("Union - Eth (Sending to Union) Connection Open: ");
        println!("---------------------------------------");
        print_json::<RelayerMsgTypes>(msg(relay_message::id::<Wasm<Union>, Evm<Minimal>, _>(
            union_chain_id.clone(),
            MsgConnectionOpenInitData(MsgConnectionOpenInit {
                client_id: parse!("08-wasm-0"),
                counterparty: connection::counterparty::Counterparty {
                    client_id: parse!("cometbls-0"),
                    connection_id: parse!(""),
                    prefix: MerklePrefix {
                        key_prefix: b"ibc".to_vec(),
                    },
                },
                version: Version {
                    identifier: "1".into(),
                    features: [Order::Ordered, Order::Unordered].into_iter().collect(),
                },
                delay_period: DELAY_PERIOD,
            }),
        )));

        println!("---------------------------------------");
        println!("Fetch Client State: ");
        println!("---------------------------------------");
        print_json::<RelayerMsgTypes>(fetch(relay_message::id::<Wasm<Union>, Evm<Minimal>, _>(
            union_chain_id.clone(),
            LightClientSpecificFetch(UnionFetch::AbciQuery(FetchAbciQuery {
                path: proof::Path::ClientStatePath(proof::ClientStatePath {
                    client_id: parse!("client-id"),
                }),
                height: parse!("123-456"),
                ty: AbciQueryType::State,
            })),
        )));

        println!("---------------------------------------");
        println!("Eth - Union (Sending to Union) Channel Open: ");
        println!("---------------------------------------");
        print_json::<RelayerMsgTypes>(msg(relay_message::id::<Wasm<Union>, Evm<Minimal>, _>(
            union_chain_id.clone(),
            MsgChannelOpenInitData {
                msg: MsgChannelOpenInit {
                    port_id: parse!("WASM_PORT_ID"),
                    channel: Channel {
                        state: channel::state::State::Init,
                        ordering: channel::order::Order::Unordered,
                        counterparty: channel::counterparty::Counterparty {
                            port_id: parse!("ucs01-relay"),
                            channel_id: parse!(""),
                        },
                        connection_hops: vec![parse!("connection-8")],
                        version: "ucs01-0".to_string(),
                    },
                },
                __marker: PhantomData,
            },
        )));

        println!("---------------------------------------");
        println!("Eth - Union (Starting on Union) Channel Open: ");
        println!("---------------------------------------");
        print_json::<RelayerMsgTypes>(msg(relay_message::id::<Evm<Minimal>, Wasm<Union>, _>(
            eth_chain_id,
            MsgChannelOpenInitData {
                msg: MsgChannelOpenInit {
                    port_id: parse!("ucs01-relay"),
                    channel: Channel {
                        state: channel::state::State::Init,
                        ordering: channel::order::Order::Ordered,
                        counterparty: channel::counterparty::Counterparty {
                            port_id: parse!("ucs01-relay"),
                            channel_id: parse!(""),
                        },
                        connection_hops: vec![parse!("connection-8")],
                        version: "ucs001-pingpong".to_string(),
                    },
                },
                __marker: PhantomData,
            },
        )));

        println!("---------------------------------------");
        println!("Eth - Union (Sending to Eth) Connection Open: ");
        println!("---------------------------------------");
        print_json::<RelayerMsgTypes>(msg(relay_message::id::<Evm<Minimal>, Wasm<Union>, _>(
            eth_chain_id,
            MsgConnectionOpenInitData(MsgConnectionOpenInit {
                client_id: parse!("cometbls-0"),
                counterparty: connection::counterparty::Counterparty {
                    client_id: parse!("08-wasm-0"),
                    connection_id: parse!(""),
                    prefix: MerklePrefix {
                        key_prefix: b"ibc".to_vec(),
                    },
                },
                version: Version {
                    identifier: "1".into(),
                    features: [Order::Ordered, Order::Unordered].into_iter().collect(),
                },
                delay_period: DELAY_PERIOD,
            }),
        )));

        println!("---------------------------------------");
        println!("Eth - Union (Sending to Eth) Connection Try: ");
        println!("---------------------------------------");
        print_json::<RelayerMsgTypes>(event(relay_message::id::<Evm<Minimal>, Wasm<Union>, _>(
            eth_chain_id,
            IbcEvent {
                tx_hash: H256([0; 32]),
                height: parse!("0-2941"),
                event: unionlabs::events::IbcEvent::ConnectionOpenTry(ConnectionOpenTry {
                    connection_id: parse!("connection-0"),
                    client_id: parse!("cometbls-0"),
                    counterparty_client_id: parse!("08-wasm-1"),
                    counterparty_connection_id: parse!("connection-14"),
                }),
            },
        )));

        println!("---------------------------------------");
        println!("Eth - Union (Sending to Eth) Update Client: ");
        println!("---------------------------------------");
        print_json::<RelayerMsgTypes>(repeat(
            u64::MAX,
            seq([
                event(relay_message::id::<Evm<Minimal>, Wasm<Union>, _>(
                    eth_chain_id,
                    relay_message::event::Command::UpdateClient {
                        client_id: parse!("cometbls-0"),
                        counterparty_client_id: parse!("08-wasm-0"),
                    },
                )),
                defer_relative(10),
            ]),
        ));

        println!("---------------------------------------");
        println!("Eth - Union (Sending to Union) Update Client: ");
        println!("---------------------------------------");
        print_json::<RelayerMsgTypes>(repeat(
            u64::MAX,
            seq([
                event(relay_message::id::<Wasm<Union>, Evm<Minimal>, _>(
                    union_chain_id.clone(),
                    relay_message::event::Command::UpdateClient {
                        client_id: parse!("08-wasm-0"),
                        counterparty_client_id: parse!("cometbls-0"),
                    },
                )),
                defer_relative(10),
            ]),
        ));

        println!("---------------------------------------");
        println!("Cosmos - Union (Sending to Cosmos) Update Client: ");
        println!("---------------------------------------");
        print_json::<RelayerMsgTypes>(repeat(
            u64::MAX,
            seq([
                event(relay_message::id::<Wasm<Cosmos>, Union, _>(
                    cosmos_chain_id.clone(),
                    relay_message::event::Command::UpdateClient {
                        client_id: parse!("08-wasm-0"),
                        counterparty_client_id: parse!("07-tendermint-0"),
                    },
                )),
                defer_relative(10),
            ]),
        ));

        println!("---------------------------------------");
        println!("Cosmos - Union (Sending to Union) Update Client: ");
        println!("---------------------------------------");
        print_json::<RelayerMsgTypes>(repeat(
            u64::MAX,
            seq([
                event(relay_message::id::<Union, Wasm<Cosmos>, _>(
                    union_chain_id.clone(),
                    relay_message::event::Command::UpdateClient {
                        client_id: parse!("07-tendermint-0"),
                        counterparty_client_id: parse!("08-wasm-0"),
                    },
                )),
                defer_relative(10),
            ]),
        ));

        println!("---------------------------------------");
        println!("Union - Eth Create Both Clients: ");
        println!("---------------------------------------");
        print_json::<RelayerMsgTypes>(seq([
            aggregate(
                [
                    fetch(relay_message::id::<Wasm<Union>, Evm<Minimal>, _>(
                        union_chain_id.clone(),
                        FetchSelfClientState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                    fetch(relay_message::id::<Wasm<Union>, Evm<Minimal>, _>(
                        union_chain_id.clone(),
                        FetchSelfConsensusState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                ],
                [],
                relay_message::id::<Evm<Minimal>, Wasm<Union>, _>(
                    eth_chain_id,
                    AggregateCreateClient {
                        config: EvmConfig {
                            client_type: "cometbls".to_string(),
                            client_address: H160(hex!("83428c7db9815f482a39a1715684dcf755021997")),
                        },
                        __marker: PhantomData,
                    },
                ),
            ),
            aggregate(
                [
                    fetch(relay_message::id::<Evm<Minimal>, Wasm<Union>, _>(
                        eth_chain_id,
                        FetchSelfClientState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                    fetch(relay_message::id::<Evm<Minimal>, Wasm<Union>, _>(
                        eth_chain_id,
                        FetchSelfConsensusState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                ],
                [],
                relay_message::id::<Wasm<Union>, Evm<Minimal>, _>(
                    union_chain_id.clone(),
                    AggregateCreateClient {
                        config: WasmConfig {
                            checksum: H256(hex!(
                                "78266014ea77f3b785e45a33d1f8d3709444a076b3b38b2aeef265b39ad1e494"
                            )),
                        },
                        __marker: PhantomData,
                    },
                ),
            ),
        ]));

        println!("---------------------------------------");
        println!("Union - Cosmos Create Both Client: ");
        println!("---------------------------------------");
        print_json::<RelayerMsgTypes>(seq([
            aggregate(
                [
                    fetch(relay_message::id::<Wasm<Cosmos>, Union, _>(
                        cosmos_chain_id.clone(),
                        FetchSelfClientState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                    fetch(relay_message::id::<Wasm<Cosmos>, Union, _>(
                        cosmos_chain_id.clone(),
                        FetchSelfConsensusState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                ],
                [],
                relay_message::id::<Union, Wasm<Cosmos>, _>(
                    union_chain_id.clone(),
                    AggregateCreateClient {
                        config: (),
                        __marker: PhantomData,
                    },
                ),
            ),
            aggregate(
                [
                    fetch(relay_message::id::<Union, Wasm<Cosmos>, _>(
                        union_chain_id.clone(),
                        FetchSelfClientState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                    fetch(relay_message::id::<Union, Wasm<Cosmos>, _>(
                        union_chain_id.clone(),
                        FetchSelfConsensusState {
                            at: QueryHeight::Latest,
                            __marker: PhantomData,
                        },
                    )),
                ],
                [],
                relay_message::id::<Wasm<Cosmos>, Union, _>(
                    cosmos_chain_id,
                    AggregateCreateClient {
                        config: WasmConfig {
                            checksum: H256(hex!(
                                "78266014ea77f3b785e45a33d1f8d3709444a076b3b38b2aeef265b39ad1e494"
                            )),
                        },
                        __marker: PhantomData,
                    },
                ),
            ),
        ]));

        print_json::<BlockPollingTypes>(fetch(block_message::id::<Cosmos, _>(
            "simd-devnet-1".parse().unwrap(),
            block_message::fetch::FetchBlock {
                height: unionlabs::ibc::core::client::height::Height {
                    revision_number: 1,
                    revision_height: 1,
                },
            },
        )));

        print_json::<BlockPollingTypes>(fetch(block_message::id::<Union, _>(
            "union-devnet-1".parse().unwrap(),
            block_message::fetch::FetchBlock {
                height: unionlabs::ibc::core::client::height::Height {
                    revision_number: 1,
                    revision_height: 1,
                },
            },
        )));
    }

    fn print_json<T: QueueMsgTypes>(msg: QueueMsg<T>)
    where
        VoyagerMessageTypes: FromQueueMsg<T>,
    {
        let msg = VoyagerMessageTypes::from_queue_msg(msg);

        let json = serde_json::to_string(&msg).unwrap();

        println!("{json}\n");

        let from_json = serde_json::from_str(&json).unwrap();

        assert_eq!(&msg, &from_json, "json roundtrip failed");
    }
}
