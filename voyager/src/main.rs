// *almost* stable, more than safe enough to use imo https://github.com/rust-lang/rfcs/pull/3425
#![feature(return_position_impl_trait_in_trait)]
// #![warn(clippy::pedantic)]
#![allow(
     // required due to return_position_impl_trait_in_trait false positives
    clippy::manual_async_fn,
    clippy::module_name_repetitions
)]

// nix run .# -- tx wasm instantiate 1 '{"default_timeout":10000,"gov_contract":"union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2","allowlist":[]}' --label blah --from alice --gas auto --keyring-backend test --gas-adjustment 1.3 --amount 100stake --no-admin --chain-id union-devnet-1

use std::{collections::btree_map::Entry, fmt::Debug, fs::read_to_string};

use anyhow::bail;
use clap::Parser;
use futures::{future::join, FutureExt, Stream, StreamExt};
use prost::Message;
use unionlabs::{
    cosmos::base::coin::Coin,
    ethereum_consts_traits::{Mainnet, Minimal, PresetBaseKind},
    ibc::{
        applications::transfer::msg_transfer::MsgTransfer,
        core::{
            channel::{
                self, channel::Channel, msg_channel_open_ack::MsgChannelOpenAck,
                msg_channel_open_confirm::MsgChannelOpenConfirm,
                msg_channel_open_init::MsgChannelOpenInit, msg_channel_open_try::MsgChannelOpenTry,
                msg_recv_packet::MsgRecvPacket, order::Order, packet::Packet,
            },
            client::height::Height,
            commitment::merkle_prefix::MerklePrefix,
            connection::{
                self, msg_connection_open_ack::MsgConnectionOpenAck,
                msg_connection_open_confirm::MsgConnectionOpenConfirm,
                msg_connection_open_init::MsgConnectionOpenInit,
                msg_connection_open_try::MsgConnectionOpenTry, version::Version,
            },
        },
    },
    IntoProto,
};

use crate::{
    chain::{
        cosmos::{Ethereum, Union},
        evm::Evm,
        proof::{
            ChannelEndPath, ClientConsensusStatePath, ClientStatePath, CommitmentPath,
            ConnectionPath, IbcStateRead, IbcStateReadPaths,
        },
        AnyChain, Chain, ChainConnection, ClientState, ClientStateOf, Connect, CreateClient,
        LightClient,
    },
    cli::{
        AppArgs, ChainAddCmd, ChainCmd, ChannelCmd, ClientCmd, ClientCreateCmd, CometblsClientType,
        Command, EvmClientType, IbcCmd, IbcQueryCmd, QueryCmd, SubmitPacketCmd,
    },
    config::{ChainConfig, Config, EvmChainConfig},
};

pub mod cli;
pub mod config;

pub mod chain;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();

    let args = AppArgs::parse();

    do_main(args).await
}

#[allow(clippy::too_many_lines)]
async fn do_main(args: cli::AppArgs) -> Result<(), anyhow::Error> {
    let mut voyager_config = read_to_string(&args.config_file_path)
        .map_or(Config::default(), |s| {
            serde_json::from_str::<Config>(&s).unwrap()
        });

    match args.command {
        Command::PrintConfig => {
            println!("{}", serde_json::to_string_pretty(&voyager_config).unwrap());
        }
        Command::Chain(chain) => match chain {
            ChainCmd::Add(add) => {
                let (name, cfg, overwrite) = match add {
                    ChainAddCmd::Evm {
                        name,
                        config,
                        overwrite,
                        preset_base,
                    } => (
                        name,
                        config::ChainConfig::Evm(match preset_base {
                            PresetBaseKind::Mainnet => config::EvmChainConfig::Mainnet(config),
                            PresetBaseKind::Minimal => config::EvmChainConfig::Minimal(config),
                        }),
                        overwrite,
                    ),
                    ChainAddCmd::Union {
                        name,
                        config,
                        overwrite,
                    } => (name, config::ChainConfig::Union(config), overwrite),
                };

                match voyager_config.chain.entry(name) {
                    Entry::Vacant(vacant) => {
                        vacant.insert(cfg);
                    }
                    Entry::Occupied(mut occupied) => {
                        if overwrite {
                            occupied.insert(cfg)
                        } else {
                            bail!("Not overwriting existing config file entry 'chain.{}'. Pass --overwrite if this is desired.", occupied.key())
                        };
                    }
                };
            }
        },
        Command::Client(client) => match client {
            ClientCmd::Create(create) => match create {
                ClientCreateCmd::Evm(ty) => match ty {
                    EvmClientType::Cometbls {
                        on,
                        counterparty,
                        config: cometbls_config,
                    } => {
                        match (
                            voyager_config.get_chain(&on).await.unwrap(),
                            voyager_config.get_chain(&counterparty).await.unwrap(),
                        ) {
                            (AnyChain::EvmMainnet(evm), AnyChain::Union(union)) => {
                                let (client_id, _) =
                                    evm.create_client(cometbls_config, union).await;
                                println!("{}", client_id);
                            }
                            (AnyChain::EvmMinimal(evm), AnyChain::Union(union)) => {
                                let (client_id, _) =
                                    evm.create_client(cometbls_config, union).await;
                                println!("{}", client_id);
                            }
                            _ => {
                                panic!("invalid chain config")
                            }
                        }
                    }
                },
                ClientCreateCmd::Union(ty) => match ty {
                    CometblsClientType::Ethereum08Wasm {
                        config: ethereum_config,
                        on,
                        counterparty,
                    } => {
                        match (
                            voyager_config.get_chain(&on).await.unwrap(),
                            voyager_config.get_chain(&counterparty).await.unwrap(),
                        ) {
                            (AnyChain::Union(union), AnyChain::EvmMainnet(evm)) => {
                                let (client_id, _) =
                                    union.create_client(ethereum_config, evm).await;
                                println!("{}", client_id);
                            }
                            (AnyChain::Union(union), AnyChain::EvmMinimal(evm)) => {
                                let (client_id, _) =
                                    union.create_client(ethereum_config, evm).await;
                                println!("{}", client_id);
                            }
                            _ => {
                                panic!("invalid chain config")
                            }
                        }
                    }
                },
            },
        },
        Command::Connection(connection) => match connection {
            cli::ConnectionCmd::Open {
                from_chain: from_chain_name,
                from_client,
                to_chain: to_chain_name,
                to_client,
            } => {
                let from_chain = voyager_config.get_chain(&from_chain_name).await.unwrap();
                let to_chain = voyager_config.get_chain(&to_chain_name).await.unwrap();

                match (from_chain, to_chain) {
                    // union -> evm
                    (AnyChain::Union(union), AnyChain::EvmMainnet(evm)) => {
                        do_connection_handshake(
                            (from_client, union.light_client()),
                            (to_client, evm.light_client()),
                        )
                        .await;
                    }
                    (AnyChain::Union(union), AnyChain::EvmMinimal(evm)) => {
                        do_connection_handshake(
                            (from_client, union.light_client()),
                            (to_client, evm.light_client()),
                        )
                        .await;
                    }

                    // evm -> union
                    (AnyChain::EvmMainnet(evm), AnyChain::Union(union)) => {
                        do_connection_handshake(
                            (from_client, evm.light_client()),
                            (to_client, union.light_client()),
                        )
                        .await;
                    }
                    (AnyChain::EvmMinimal(evm), AnyChain::Union(union)) => {
                        do_connection_handshake(
                            (from_client, evm.light_client()),
                            (to_client, union.light_client()),
                        )
                        .await;
                    }

                    _ => {
                        bail!("Cannot connect from '{from_chain_name}' to '{to_chain_name}'")
                    }
                }
            }
        },
        Command::Channel(channel) => match channel {
            ChannelCmd::Open {
                from_chain: from_chain_name,
                from_connection,
                from_port,
                from_version,
                to_chain: to_chain_name,
                to_connection,
                to_port,
                to_version,
            } => {
                let from_chain = voyager_config.get_chain(&from_chain_name).await.unwrap();
                let to_chain = voyager_config.get_chain(&to_chain_name).await.unwrap();

                match (from_chain, to_chain) {
                    // union -> evm
                    (AnyChain::Union(union), AnyChain::EvmMainnet(evm)) => {
                        do_channel_handshake(
                            &union.light_client(),
                            &evm.light_client(),
                            from_connection,
                            to_connection,
                            from_port,
                            to_port,
                            from_version,
                            to_version,
                        )
                        .await;
                    }
                    (AnyChain::Union(union), AnyChain::EvmMinimal(evm)) => {
                        do_channel_handshake(
                            &union.light_client(),
                            &evm.light_client(),
                            from_connection,
                            to_connection,
                            from_port,
                            to_port,
                            from_version,
                            to_version,
                        )
                        .await;
                    }

                    // evm -> union
                    (AnyChain::EvmMainnet(evm), AnyChain::Union(union)) => {
                        do_channel_handshake(
                            &evm.light_client(),
                            &union.light_client(),
                            from_connection,
                            to_connection,
                            from_port,
                            to_port,
                            from_version,
                            to_version,
                        )
                        .await;
                    }
                    (AnyChain::EvmMinimal(evm), AnyChain::Union(union)) => {
                        do_channel_handshake(
                            &evm.light_client(),
                            &union.light_client(),
                            from_connection,
                            to_connection,
                            from_port,
                            to_port,
                            from_version,
                            to_version,
                        )
                        .await;
                    }

                    _ => {
                        bail!("Cannot connect from '{from_chain_name}' to '{to_chain_name}'")
                    }
                }
            }
        },
        Command::Relay(relay) => {
            for cli::Between(a, b) in relay.between {
                let a_chain = voyager_config.get_chain(&a).await.unwrap();
                let b_chain = voyager_config.get_chain(&b).await.unwrap();

                match (a_chain, b_chain) {
                    // union -> evm
                    (AnyChain::Union(union), AnyChain::EvmMainnet(evm)) => {
                        relay_packets(&union, &evm).await;
                    }
                    (AnyChain::Union(union), AnyChain::EvmMinimal(evm)) => {
                        relay_packets(&union, &evm).await;
                    }

                    // evm -> union
                    (AnyChain::EvmMainnet(evm), AnyChain::Union(union)) => {
                        relay_packets(&evm, &union).await;
                    }
                    (AnyChain::EvmMinimal(evm), AnyChain::Union(union)) => {
                        relay_packets(&evm, &union).await;
                    }

                    _ => {
                        bail!("Cannot relay between '{a}' and '{b}'")
                    }
                }
            }
        }
        Command::SubmitPacket(SubmitPacketCmd::Transfer {
            denom,
            amount,
            receiver,
            source_port,
            source_channel,
            on,
        }) => {
            let chain_config = voyager_config.chain.get(&on).unwrap();

            let msg = |sender: String| MsgTransfer {
                source_port,
                source_channel,
                token: Coin {
                    denom,
                    amount: amount.to_string(),
                },
                sender,
                receiver,
                timeout_height: Height {
                    revision_number: 1,
                    revision_height: u64::MAX,
                },
                timeout_timestamp: None,
                memo: None,
            };
        }
        Command::Query(query) => match query {
            QueryCmd::Client { on, client_id } => {
                let json = match voyager_config.chain[&on].clone() {
                    ChainConfig::Evm(EvmChainConfig::Mainnet(evm)) => {
                        let evm = Evm::<Mainnet>::new(evm).await;

                        let cometbls = evm.light_client();

                        serde_json::to_string_pretty(&cometbls.query_client_state(client_id).await)
                            .unwrap()
                    }
                    ChainConfig::Evm(EvmChainConfig::Minimal(evm)) => {
                        let evm = Evm::<Minimal>::new(evm).await;

                        let cometbls = evm.light_client();

                        serde_json::to_string_pretty(&cometbls.query_client_state(client_id).await)
                            .unwrap()
                    }
                    ChainConfig::Union(union) => {
                        let union = Union::new(union).await;

                        let ethereum: Ethereum<Mainnet> = union.light_client();

                        serde_json::to_string_pretty(&ethereum.query_client_state(client_id).await)
                            .unwrap()
                    }
                };

                println!("{json}");
            }
            QueryCmd::Connection {} => todo!(),
            QueryCmd::Channel {} => todo!(),
            QueryCmd::Balances { on, who, denom } => todo!(),
        },
        Command::Setup(cmd) => match cmd {
            // TODO(aeryz): this might go into channel as well, since it's highly coupled with it
            cli::SetupCmd::BindPort {
                on,
                module_address,
                port_id,
            } => {
                let chain = voyager_config.get_chain(&on).await.unwrap();

                match chain {
                    AnyChain::EvmMinimal(evm) => {
                        evm.bind_port(module_address.into(), port_id).await
                    }
                    AnyChain::EvmMainnet(evm) => {
                        evm.bind_port(module_address.into(), port_id).await
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
                let chain = voyager_config.get_chain(&on).await.unwrap();

                match chain {
                    AnyChain::EvmMinimal(evm) => {
                        evm.setup_initial_channel(
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
            cli::SetupCmd::SetOperator { on } => todo!(),
        },
        Command::Ibc(IbcCmd::Query {
            on,
            at,
            cmd: IbcQueryCmd::Path(path),
        }) => {
            let json = match voyager_config.chain[&on].clone() {
                ChainConfig::Evm(EvmChainConfig::Mainnet(evm)) => {
                    let evm = Evm::<Mainnet>::new(evm).await;

                    path.any_state_proof_to_json::<Union, _>(evm, at).await
                }
                ChainConfig::Evm(EvmChainConfig::Minimal(evm)) => {
                    let evm = Evm::<Minimal>::new(evm).await;

                    path.any_state_proof_to_json::<Union, _>(evm, at).await
                }
                ChainConfig::Union(union) => {
                    let union = Union::new(union).await;

                    // NOTE: ChainSpec is arbitrary
                    path.any_state_proof_to_json::<Evm<Mainnet>, _>(union, at)
                        .await
                }
            };

            println!("{json}");
        }
        _ => bail!("Command not supported"),
    }

    std::fs::write(
        args.config_file_path,
        serde_json::to_string_pretty(&voyager_config).unwrap(),
    )
    .unwrap();

    Ok(())
}

/// Returns (c1 conn id, c2 conn id)
async fn do_connection_handshake<L2, L1>(
    (cometbls_client_id, cometbls): (String, L2),
    (ethereum_client_id, ethereum): (String, L1),
) -> (String, String)
where
    L2: Connect<L1>,
    L1: Connect<L2>,
    L2::HostChain: IbcStateReadPaths<L2::CounterpartyChain>,
    L1::HostChain: IbcStateReadPaths<L1::CounterpartyChain>,
    ClientStateOf<<L2 as LightClient>::CounterpartyChain>: Debug + ClientState + IntoProto,
    ClientStateOf<<L1 as LightClient>::CounterpartyChain>: Debug + ClientState + IntoProto,
{
    const DELAY_PERIOD: u64 = 6;

    let cometbls_id = cometbls.chain().chain_id().await;
    let ethereum_id = ethereum.chain().chain_id().await;

    tracing::info!(cometbls_id, ethereum_id);

    let cometbls_latest_height = cometbls.chain().query_latest_height().await;
    let ethereum_latest_height = ethereum.chain().query_latest_height().await;

    let cometbls_latest_trusted_height = ethereum
        .query_client_state(ethereum_client_id.clone())
        .await
        .height();
    let ethereum_latest_trusted_height = cometbls
        .query_client_state(cometbls_client_id.clone())
        .await
        .height();

    tracing::info!(%cometbls_latest_trusted_height, %cometbls_latest_height);
    tracing::info!(%ethereum_latest_trusted_height, %ethereum_latest_height);

    let cometbls_latest_height = cometbls
        .update_counterparty_client(
            &ethereum,
            ethereum_client_id.clone(),
            cometbls_latest_trusted_height,
            cometbls_latest_height,
        )
        .await;

    let ethereum_latest_height = ethereum
        .update_counterparty_client(
            &cometbls,
            cometbls_client_id.clone(),
            ethereum_latest_trusted_height,
            ethereum_latest_height,
        )
        .await;

    let (_connection_open_init_height, connection_open_init) = cometbls
        .connection_open_init(MsgConnectionOpenInit {
            client_id: cometbls_client_id.clone(),
            counterparty: connection::counterparty::Counterparty {
                client_id: ethereum_client_id.clone(),
                // TODO(benluelo): Create a new struct with this field omitted as it's unused for open init
                connection_id: String::new(),
                prefix: MerklePrefix {
                    key_prefix: b"ibc".to_vec(),
                },
            },
            version: Version {
                identifier: "1".into(),
                features: [Order::Ordered, Order::Unordered].into_iter().collect(),
            },
            delay_period: DELAY_PERIOD,
        })
        .await;

    tracing::info!(
        cometbls_connection_id = connection_open_init.connection_id,
        %cometbls_latest_height,
        %ethereum_latest_height,
        cometbls_client_id,
        ethereum_client_id,
        "right after connection init"
    );

    let cometbls_update_from = cometbls_latest_height;
    let cometbls_update_to = cometbls.chain().query_latest_height().await;

    let cometbls_latest_height = cometbls
        .update_counterparty_client(
            &ethereum,
            ethereum_client_id.clone(),
            cometbls_update_from,
            cometbls_update_to,
        )
        .await;

    tracing::info!(
        chain_id = cometbls_id,
        connection_id = connection_open_init.connection_id,
        latest_height = ?cometbls_latest_height,
        "right after updating cosmos"
    );

    // generate state proofs

    let cometbls_client_state_proof = cometbls
        .chain()
        .state_proof(
            ClientStatePath {
                client_id: cometbls_client_id.clone(),
            },
            cometbls_latest_height,
        )
        .await;
    let cometbls_consensus_state_proof = cometbls
        .chain()
        .state_proof(
            ClientConsensusStatePath {
                client_id: cometbls_client_id.clone(),
                height: ethereum_latest_height,
            },
            cometbls_latest_height,
        )
        .await;
    let cometbls_connection_state_proof = cometbls
        .chain()
        .state_proof(
            ConnectionPath {
                connection_id: connection_open_init.connection_id.clone(),
            },
            cometbls_latest_height,
        )
        .await;

    let (connection_open_try_height, connection_open_try) = ethereum
        .connection_open_try(MsgConnectionOpenTry {
            client_id: ethereum_client_id.clone(),
            counterparty: connection::counterparty::Counterparty {
                client_id: cometbls_client_id.clone(),
                connection_id: connection_open_init.connection_id.clone(),
                prefix: MerklePrefix {
                    key_prefix: b"ibc".to_vec(),
                },
            },
            delay_period: DELAY_PERIOD,
            client_state: cometbls_client_state_proof.state,
            counterparty_versions: cometbls_connection_state_proof.state.versions,
            proof_height: cometbls_consensus_state_proof.proof_height,
            proof_init: cometbls_connection_state_proof.proof,
            proof_client: cometbls_client_state_proof.proof,
            proof_consensus: cometbls_consensus_state_proof.proof,
            consensus_height: ethereum_latest_height,
        })
        .await;

    tracing::info!(
        "Connection open try executed at {:?}",
        connection_open_try_height
    );

    let ethereum_update_from = ethereum_latest_height;
    let ethereum_update_to = loop {
        let height = ethereum.chain().query_latest_height().await;
        if height >= connection_open_try_height.increment() {
            break connection_open_try_height.increment();
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    };

    tracing::info!("Querying proof at {:?}", connection_open_try_height);

    let _ = ethereum
        .update_counterparty_client(
            &cometbls,
            cometbls_client_id.clone(),
            ethereum_update_from,
            ethereum_update_to,
        )
        .await;

    let ethereum_connection_state_proof = ethereum
        .chain()
        .state_proof(
            ConnectionPath {
                connection_id: connection_open_try.connection_id.clone(),
            },
            connection_open_try_height,
        )
        .await;
    let ethereum_client_state_proof = ethereum
        .chain()
        .state_proof(
            ClientStatePath {
                client_id: ethereum_client_id.clone(),
            },
            connection_open_try_height,
        )
        .await;
    let ethereum_consensus_state_proof = ethereum
        .chain()
        .state_proof(
            ClientConsensusStatePath {
                client_id: ethereum_client_id.clone(),
                height: cometbls
                    .process_height_for_counterparty(cometbls_latest_height)
                    .await,
            },
            connection_open_try_height,
        )
        .await;

    let cl = cometbls
        .query_client_state(cometbls_client_id.clone())
        .await;

    tracing::debug!(
        "Cometbls client state {:?}",
        ethers::utils::hex::encode(cl.into_proto().encode_to_vec())
    );

    let cl = ethereum
        .query_client_state(ethereum_client_id.clone())
        .await;

    tracing::debug!(
        "Evm client state {:?}",
        ethers::utils::hex::encode(cl.into_proto().encode_to_vec())
    );

    tracing::debug!(
        "Proof Connection {:?}",
        ethers::utils::hex::encode(&ethereum_connection_state_proof.proof)
    );
    tracing::debug!(
        "Proof Client {:?}",
        ethers::utils::hex::encode(&ethereum_client_state_proof.proof)
    );
    tracing::debug!(
        "Proof Consensus {:?}",
        ethers::utils::hex::encode(&ethereum_consensus_state_proof.proof)
    );

    let (_, connection_open_ack) = cometbls
        .connection_open_ack(MsgConnectionOpenAck {
            connection_id: connection_open_try.counterparty_connection_id.clone(),
            counterparty_connection_id: connection_open_try.connection_id.clone(),

            version: Version {
                identifier: "1".into(),
                features: [Order::Ordered, Order::Unordered].into_iter().collect(),
            },
            client_state: ethereum_client_state_proof.state,
            proof_height: ethereum_update_to,
            proof_try: ethereum_connection_state_proof.proof,
            proof_client: ethereum_client_state_proof.proof,
            proof_consensus: ethereum_consensus_state_proof.proof,
            consensus_height: ethereum_consensus_state_proof.proof_height,
        })
        .await;

    let cometbls_update_from = cometbls_latest_height;
    let cometbls_update_to = cometbls.chain().query_latest_height().await;

    let cometbls_latest_height = cometbls
        .update_counterparty_client(
            &ethereum,
            ethereum_client_id.clone(),
            cometbls_update_from,
            cometbls_update_to,
        )
        .await;

    let cometbls_connection_state_proof = cometbls
        .chain()
        .state_proof(
            ConnectionPath {
                connection_id: connection_open_ack.connection_id.clone(),
            },
            cometbls_latest_height,
        )
        .await;

    let (_, connection_open_confirm) = ethereum
        .connection_open_confirm(MsgConnectionOpenConfirm {
            connection_id: connection_open_ack.counterparty_connection_id.clone(),
            proof_ack: cometbls_connection_state_proof.proof,
            proof_height: cometbls_connection_state_proof.proof_height,
        })
        .await;

    tracing::info!(?connection_open_confirm, "connection opened");

    (
        connection_open_confirm.connection_id,
        connection_open_confirm.counterparty_connection_id,
    )
}

#[allow(clippy::too_many_arguments)] // fight me clippy
async fn do_channel_handshake<L2, L1>(
    cometbls: &L2,
    ethereum: &L1,
    cometbls_connection_id: String,
    ethereum_connection_id: String,
    cometbls_port_id: String,
    ethereum_port_id: String,
    cometbls_channel_version: String,
    ethereum_channel_version: String,
) -> (String, String)
where
    L2: Connect<L1>,
    L1: Connect<L2>,
    L2::HostChain: IbcStateReadPaths<L2::CounterpartyChain>,
    L1::HostChain: IbcStateReadPaths<L1::CounterpartyChain>,
    ClientStateOf<<L2 as LightClient>::CounterpartyChain>: Debug + ClientState,
    ClientStateOf<<L1 as LightClient>::CounterpartyChain>: Debug + ClientState,
{
    let cometbls_id = cometbls.chain().chain_id().await;
    let ethereum_id = ethereum.chain().chain_id().await;

    let ethereum_client_id = ethereum
        .chain()
        .state_proof(
            ConnectionPath {
                connection_id: ethereum_connection_id.clone(),
            },
            ethereum.chain().query_latest_height().await,
        )
        .await
        .state
        .client_id;

    let cometbls_client_id = cometbls
        .chain()
        .state_proof(
            ConnectionPath {
                connection_id: cometbls_connection_id.clone(),
            },
            cometbls.chain().query_latest_height().await,
        )
        .await
        .state
        .client_id;

    tracing::info!(cometbls_id, ethereum_id);

    tracing::debug!("ChannelOpenInit");

    let (_channel_open_init_height, channel_open_init) = cometbls
        .channel_open_init(MsgChannelOpenInit {
            port_id: cometbls_port_id.to_string(),
            channel: Channel {
                state: channel::state::State::Init,
                ordering: Order::Unordered,
                counterparty: channel::counterparty::Counterparty {
                    port_id: ethereum_port_id.to_string(),
                    // TODO(benluelo): Make a struct without this field?
                    channel_id: String::new(),
                },
                connection_hops: vec![cometbls_connection_id.clone()],
                version: cometbls_channel_version.clone(),
            },
        })
        .await;

    let ethereum_latest_trusted_height = ethereum
        .query_client_state(ethereum_client_id.clone())
        .await
        .height();

    let cometbls_latest_height = cometbls.chain().query_latest_height().await;

    let cometbls_latest_height = cometbls
        .update_counterparty_client(
            ethereum,
            ethereum_client_id.clone(),
            ethereum_latest_trusted_height,
            cometbls_latest_height,
        )
        .await;

    let proof = cometbls
        .chain()
        .state_proof(
            ChannelEndPath {
                port_id: channel_open_init.port_id.clone(),
                channel_id: channel_open_init.channel_id.clone(),
            },
            cometbls_latest_height,
        )
        .await;

    tracing::debug!("ChannelOpenTry");

    let (channel_open_try_height, channel_open_try) = ethereum
        .channel_open_try(MsgChannelOpenTry {
            port_id: ethereum_port_id.clone(),
            channel: Channel {
                state: channel::state::State::Tryopen,
                ordering: Order::Unordered,
                counterparty: channel::counterparty::Counterparty {
                    port_id: channel_open_init.port_id.clone(),
                    channel_id: channel_open_init.channel_id.clone(),
                },
                connection_hops: vec![ethereum_connection_id.clone()],
                version: ethereum_channel_version.clone(),
            },
            counterparty_version: cometbls_channel_version.clone(),
            proof_init: proof.proof,
            proof_height: proof.proof_height,
        })
        .await;

    let cometbls_latest_trusted_height = cometbls
        .query_client_state(cometbls_client_id.clone())
        .await
        .height();
    let ethereum_update_to = loop {
        let height = ethereum.chain().query_latest_height().await;
        if height >= channel_open_try_height.increment() {
            break channel_open_try_height.increment();
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    };

    tracing::info!("Querying proof at {:?}", channel_open_try_height);

    let _ = ethereum
        .update_counterparty_client(
            cometbls,
            cometbls_client_id.clone(),
            cometbls_latest_trusted_height,
            ethereum_update_to,
        )
        .await;

    let proof = ethereum
        .chain()
        .state_proof(
            ChannelEndPath {
                port_id: channel_open_try.port_id.clone(),
                channel_id: channel_open_try.channel_id.clone(),
            },
            channel_open_try_height,
        )
        .await;

    tracing::debug!("ChannelOpenAck");

    let (_channel_open_ack_height, channel_open_ack) = cometbls
        .channel_open_ack(MsgChannelOpenAck {
            port_id: cometbls_port_id.clone(),
            channel_id: channel_open_try.counterparty_channel_id.clone(),
            counterparty_channel_id: channel_open_try.channel_id.clone(),
            counterparty_version: channel_open_try.version.clone(),
            proof_try: proof.proof,
            proof_height: ethereum_update_to,
        })
        .await;

    let update_to = cometbls.chain().query_latest_height().await;

    let cometbls_latest_height = cometbls
        .update_counterparty_client(
            ethereum,
            ethereum_client_id.clone(),
            cometbls_latest_height,
            update_to,
        )
        .await;

    let proof = cometbls
        .chain()
        .state_proof(
            ChannelEndPath {
                port_id: channel_open_ack.port_id.clone(),
                channel_id: channel_open_ack.channel_id.clone(),
            },
            cometbls_latest_height,
        )
        .await;

    tracing::debug!("ChannelOpenConfirm");

    let (_, confirm) = ethereum
        .channel_open_confirm(MsgChannelOpenConfirm {
            port_id: channel_open_ack.counterparty_port_id.clone(),
            channel_id: channel_open_ack.counterparty_channel_id.clone(),
            proof_ack: proof.proof,
            proof_height: proof.proof_height,
        })
        .await;

    tracing::info!(?confirm, "channel opened");

    (confirm.channel_id, confirm.counterparty_channel_id)
}

async fn relay_packets<FromChain, ToChain>(from: &FromChain, to: &ToChain)
where
    FromChain: ChainConnection<ToChain>,
    ToChain: ChainConnection<FromChain>,
    <<FromChain as ChainConnection<ToChain>>::LightClient as LightClient>::HostChain:
        IbcStateReadPaths<<<FromChain as ChainConnection<ToChain>>::LightClient as LightClient>::CounterpartyChain>,
    <<ToChain as ChainConnection<FromChain>>::LightClient as LightClient>::HostChain:
        IbcStateReadPaths<<<ToChain as ChainConnection<FromChain>>::LightClient as LightClient>::CounterpartyChain>,
    ClientStateOf<FromChain>: IntoProto,
    ClientStateOf<ToChain>: IntoProto,
{
    async fn relay_packets_inner<L1, L2>(
        lc1: &L2,
        lc1_event_stream: impl Stream<Item = (Height, Packet)>,
        lc2: &L1,
    ) where
        L1: Connect<L2>,
        L2: Connect<L1>,
        L2::HostChain: IbcStateReadPaths<L2::CounterpartyChain>,
        L1::HostChain: IbcStateReadPaths<L1::CounterpartyChain>,
    {
        lc1_event_stream
            .for_each(move |(event_height, packet)| async move {
                tracing::info!("received packet");

                let sequence = packet.sequence;

                let lc2_client_id = lc1
                    .chain()
                    .state_proof(
                        ChannelEndPath {
                            channel_id: packet.source_channel.clone(),
                            port_id: packet.source_port.clone(),
                        },
                        event_height,
                    )
                    .then(|channel| {
                        lc1.chain()
                            .state_proof(
                                ConnectionPath {
                                    connection_id: channel.state.connection_hops[0].clone(),
                                },
                                event_height,
                            )
                            .map(|connection| connection.state.counterparty.client_id)
                    })
                    .await;

                tracing::info!("relaying packet to {lc2_client_id}");

                let lc2_latest_trusted_height =
                    lc2.query_client_state(lc2_client_id.clone()).await.height();

                tracing::info!(
                    "latest trusted height on {lc2_client_id} is {lc2_latest_trusted_height}"
                );

                let lc1_update_to = loop {
                    let height = lc1.chain().query_latest_height().await;
                    if height >= event_height.increment() {
                        break event_height.increment();
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                };

                let lc1_updated_to = lc1
                    .update_counterparty_client(
                        lc2,
                        lc2_client_id.clone(),
                        lc2_latest_trusted_height,
                        lc1_update_to,
                    )
                    .await;

                tracing::info!("updated {lc2_client_id} to {lc1_updated_to}");

                let commitment_proof = lc1
                    .chain()
                    .state_proof(
                        CommitmentPath {
                            port_id: packet.source_port.clone(),
                            channel_id: packet.source_channel.clone(),
                            sequence,
                        },
                        event_height,
                    )
                    .await;

                lc2.recv_packet(MsgRecvPacket {
                    packet,
                    proof_height: lc1.process_height_for_counterparty(lc1_updated_to).await,
                    proof_commitment: commitment_proof.proof,
                })
                .await;
            })
            .await;
    }

    let l1 = from.light_client();
    let l2 = to.light_client();

    let l1_packet_stream = from.packet_stream().await;
    let l2_packet_stream = to.packet_stream().await;

    join(
        relay_packets_inner(&l2, l2_packet_stream, &l1),
        relay_packets_inner(&l1, l1_packet_stream, &l2),
    )
    .await;
}
