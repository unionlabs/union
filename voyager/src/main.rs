#![recursion_limit = "256"]
// *almost* stable, more than safe enough to use imo https://github.com/rust-lang/rfcs/pull/3425
#![feature(return_position_impl_trait_in_trait)]
// #![warn(clippy::pedantic)]
#![allow(
     // required due to return_position_impl_trait_in_trait false positives
    clippy::manual_async_fn,
    clippy::module_name_repetitions
)]

// nix run .# -- tx wasm instantiate 1 '{"default_timeout":10000,"gov_contract":"union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2","allowlist":[]}' --label blah --from alice --gas auto --keyring-backend test --gas-adjustment 1.3 --amount 100stake --no-admin --chain-id union-devnet-1

use std::fs::read_to_string;

use chain_utils::{evm::Evm, union::Union};
use clap::Parser;
use typenum::Unsigned;
use unionlabs::ethereum_consts_traits::{ChainSpec, Mainnet, Minimal};

use crate::{
    chain::AnyChain,
    cli::{AppArgs, Command, IbcCmd, IbcQueryCmd},
    config::{ChainConfig, Config, EvmChainConfig},
    queue::Voyager,
};

pub const DELAY_PERIOD: u64 = 0;

pub mod cli;
pub mod config;

pub mod msg;

pub mod queue;

pub mod chain;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();

    let args = AppArgs::parse();

    do_main(args).await
}

#[allow(clippy::too_many_lines)]
// NOTE: This function is a mess, will be cleaned up
async fn do_main(args: cli::AppArgs) -> Result<(), anyhow::Error> {
    let voyager_config = read_to_string(&args.config_file_path).map_or(Config::default(), |s| {
        serde_json::from_str::<Config>(&s).unwrap()
    });

    match args.command {
        Command::PrintConfig => {
            println!("{}", serde_json::to_string_pretty(&voyager_config).unwrap());

            let queue = Voyager::new(voyager_config.clone()).await;

            queue.run().await;
        }
        // Command::Chain(chain) => match chain {
        //     ChainCmd::Add(add) => {
        //         let (name, cfg, overwrite) = match add {
        //             ChainAddCmd::Evm {
        //                 name,
        //                 config,
        //                 overwrite,
        //                 preset_base,
        //             } => (
        //                 name,
        //                 config::ChainConfig::Evm(match preset_base {
        //                     PresetBaseKind::Mainnet => config::EvmChainConfig::Mainnet(config),
        //                     PresetBaseKind::Minimal => config::EvmChainConfig::Minimal(config),
        //                 }),
        //                 overwrite,
        //             ),
        //             ChainAddCmd::Union {
        //                 name,
        //                 config,
        //                 overwrite,
        //             } => (name, config::ChainConfig::Union(config), overwrite),
        //         };

        //         match voyager_config.chain.entry(name) {
        //             Entry::Vacant(vacant) => {
        //                 vacant.insert(cfg);
        //             }
        //             Entry::Occupied(mut occupied) => {
        //                 if overwrite {
        //                     occupied.insert(cfg)
        //                 } else {
        //                     bail!("Not overwriting existing config file entry 'chain.{}'. Pass --overwrite if this is desired.", occupied.key())
        //                 };
        //             }
        //         };
        //     }
        // },
        // Command::Client(client) => match client {
        //     ClientCmd::Create(create) => match create {
        //         ClientCreateCmd::Evm(ty) => match ty {
        //             EvmClientType::Cometbls {
        //                 on,
        //                 counterparty,
        //                 config: cometbls_config,
        //             } => {
        //                 // match (
        //                 //     voyager_config.get_chain(&on).await.unwrap(),
        //                 //     voyager_config.get_chain(&counterparty).await.unwrap(),
        //                 // ) {
        //                 //     (AnyChain::EvmMainnet(evm), AnyChain::Union(union)) => {
        //                 //         let client_id = evm.create_client(cometbls_config, union).await;
        //                 //         println!("{}", client_id);
        //                 //     }
        //                 //     (AnyChain::EvmMinimal(evm), AnyChain::Union(union)) => {
        //                 //         let client_id = evm.create_client(cometbls_config, union).await;
        //                 //         println!("{}", client_id);
        //                 //     }
        //                 //     _ => {
        //                 //         panic!("invalid chain config")
        //                 //     }
        //                 // }
        //             }
        //         },
        //         ClientCreateCmd::Union(ty) => match ty {
        //             CometblsClientType::Ethereum08Wasm {
        //                 config: ethereum_config,
        //                 on,
        //                 counterparty,
        //             } => {
        //                 // match (
        //                 //     voyager_config.get_chain(&on).await.unwrap(),
        //                 //     voyager_config.get_chain(&counterparty).await.unwrap(),
        //                 // ) {
        //                 //     (AnyChain::Union(union), AnyChain::EvmMainnet(evm)) => {
        //                 //         let client_id = union.create_client(ethereum_config, evm).await;
        //                 //         println!("{}", client_id);
        //                 //     }
        //                 //     (AnyChain::Union(union), AnyChain::EvmMinimal(evm)) => {
        //                 //         let client_id = union.create_client(ethereum_config, evm).await;
        //                 //         println!("{}", client_id);
        //                 //     }
        //                 //     _ => {
        //                 //         panic!("invalid chain config")
        //                 //     }
        //                 // }
        //             }
        //         },
        //     },
        // },
        // Command::Connection(connection) => match connection {
        //     cli::ConnectionCmd::Open {
        //         from_chain: from_chain_name,
        //         from_client,
        //         to_chain: to_chain_name,
        //         to_client,
        //     } => {
        //         let from_chain = voyager_config.get_chain(&from_chain_name).await.unwrap();
        //         let to_chain = voyager_config.get_chain(&to_chain_name).await.unwrap();

        //         match (from_chain, to_chain) {
        //             // union -> evm
        //             (AnyChain::Union(union), AnyChain::EvmMainnet(evm)) => {
        //                 do_connection_handshake(
        //                     (from_client, union.light_client()),
        //                     (to_client, evm.light_client()),
        //                 )
        //                 .await;
        //             }
        //             (AnyChain::Union(union), AnyChain::EvmMinimal(evm)) => {
        //                 do_connection_handshake(
        //                     (from_client, union.light_client()),
        //                     (to_client, evm.light_client()),
        //                 )
        //                 .await;
        //             }

        //             // evm -> union
        //             (AnyChain::EvmMainnet(evm), AnyChain::Union(union)) => {
        //                 do_connection_handshake(
        //                     (from_client, evm.light_client()),
        //                     (to_client, union.light_client()),
        //                 )
        //                 .await;
        //             }
        //             (AnyChain::EvmMinimal(evm), AnyChain::Union(union)) => {
        //                 do_connection_handshake(
        //                     (from_client, evm.light_client()),
        //                     (to_client, union.light_client()),
        //                 )
        //                 .await;
        //             }

        //             _ => {
        //                 bail!("Cannot connect from '{from_chain_name}' to '{to_chain_name}'")
        //             }
        //         }
        //     }
        // },
        // // Command::Channel(channel) => match channel {
        // //     ChannelCmd::Open {
        // //         from_chain: from_chain_name,
        // //         from_connection,
        // //         from_port,
        // //         from_version,
        // //         to_chain: to_chain_name,
        // //         to_connection,
        // //         to_port,
        // //         to_version,
        // //     } => {
        // //         let from_chain = voyager_config.get_chain(&from_chain_name).await.unwrap();
        // //         let to_chain = voyager_config.get_chain(&to_chain_name).await.unwrap();

        // //         match (from_chain, to_chain) {
        // //             // union -> evm
        // //             (AnyChain::Union(union), AnyChain::EvmMainnet(evm)) => {
        // //                 do_channel_handshake(
        // //                     &ChainConnection::<Evm<Mainnet>>::light_client(&union),
        // //                     &evm.light_client(),
        // //                     from_connection,
        // //                     to_connection,
        // //                     from_port,
        // //                     to_port,
        // //                     from_version,
        // //                     to_version,
        // //                 )
        // //                 .await;
        // //             }
        // //             (AnyChain::Union(union), AnyChain::EvmMinimal(evm)) => {
        // //                 do_channel_handshake(
        // //                     &ChainConnection::<Evm<Minimal>>::light_client(&union),
        // //                     &evm.light_client(),
        // //                     from_connection,
        // //                     to_connection,
        // //                     from_port,
        // //                     to_port,
        // //                     from_version,
        // //                     to_version,
        // //                 )
        // //                 .await;
        // //             }

        // //             // evm -> union
        // //             (AnyChain::EvmMainnet(evm), AnyChain::Union(union)) => {
        // //                 do_channel_handshake(
        // //                     &evm.light_client(),
        // //                     &ChainConnection::<Evm<Mainnet>>::light_client(&union),
        // //                     from_connection,
        // //                     to_connection,
        // //                     from_port,
        // //                     to_port,
        // //                     from_version,
        // //                     to_version,
        // //                 )
        // //                 .await;
        // //             }
        // //             (AnyChain::EvmMinimal(evm), AnyChain::Union(union)) => {
        // //                 do_channel_handshake(
        // //                     &evm.light_client(),
        // //                     &ChainConnection::<Evm<Minimal>>::light_client(&union),
        // //                     from_connection,
        // //                     to_connection,
        // //                     from_port,
        // //                     to_port,
        // //                     from_version,
        // //                     to_version,
        // //                 )
        // //                 .await;
        // //             }

        // //             _ => {
        // //                 bail!("Cannot connect from '{from_chain_name}' to '{to_chain_name}'")
        // //             }
        // //         }
        // //     }
        // // },
        // Command::Relay(relay) => {
        //     for cli::Between(a, b) in relay.between {
        //         let a_chain = voyager_config.get_chain(&a).await.unwrap();
        //         let b_chain = voyager_config.get_chain(&b).await.unwrap();

        //         todo!();

        //         // match (a_chain, b_chain) {
        //         //     // union -> evm
        //         //     (AnyChain::Union(union), AnyChain::EvmMainnet(evm)) => {
        //         //         relay_packets(&union, &evm).await;
        //         //     }
        //         //     (AnyChain::Union(union), AnyChain::EvmMinimal(evm)) => {
        //         //         relay_packets(&union, &evm).await;
        //         //     }

        //         //     // evm -> union
        //         //     (AnyChain::EvmMainnet(evm), AnyChain::Union(union)) => {
        //         //         relay_packets(&evm, &union).await;
        //         //     }
        //         //     (AnyChain::EvmMinimal(evm), AnyChain::Union(union)) => {
        //         //         relay_packets(&evm, &union).await;
        //         //     }

        //         //     _ => {
        //         //         bail!("Cannot relay between '{a}' and '{b}'")
        //         //     }
        //         // }
        //     }
        // }
        // Command::SubmitPacket(SubmitPacketCmd::Transfer {
        //     denom,
        //     amount,
        //     receiver,
        //     source_port,
        //     source_channel,
        //     on,
        // }) => {
        //     let chain_config = voyager_config.chain.get(&on).unwrap();

        //     let msg = |sender: String| MsgTransfer {
        //         source_port,
        //         source_channel,
        //         token: Coin {
        //             denom,
        //             amount: amount.to_string(),
        //         },
        //         sender,
        //         receiver,
        //         timeout_height: Height {
        //             revision_number: 1,
        //             revision_height: u64::MAX,
        //         },
        //         timeout_timestamp: None,
        //         memo: None,
        //     };

        //     match chain_config {
        //         ChainConfig::Evm(EvmChainConfig::Minimal(evm_config)) => {
        //             chain::evm::transfer(
        //                 &Evm::<Minimal>::new(evm_config.clone().into()).await,
        //                 msg(hex::encode(evm_config.signer.clone().value().to_bytes())),
        //                 evm_config.ics20_transfer_bank_address.clone(),
        //             )
        //             .await;
        //         }
        //         ChainConfig::Evm(EvmChainConfig::Mainnet(evm_config)) => {
        //             chain::evm::transfer(
        //                 &Evm::<Mainnet>::new(evm_config.clone().into()).await,
        //                 msg(hex::encode(evm_config.signer.clone().value().to_bytes())),
        //                 evm_config.ics20_transfer_bank_address.clone(),
        //             )
        //             .await;
        //         }
        //         ChainConfig::Union(_) => bail!("not currently supported"),
        //     }
        // }
        // Command::Query(query) => match query {
        //     QueryCmd::Balances { on, who, denom } => {
        //         let chain_config = voyager_config.chain.get(&on).unwrap();

        //         match chain_config {
        //             ChainConfig::Evm(EvmChainConfig::Minimal(evm_config)) => {
        //                 chain::evm::balance_of(
        //                     &Evm::<Minimal>::new(evm_config.clone().into()).await,
        //                     evm_config.ics20_bank_address.clone(),
        //                     who.into(),
        //                     denom,
        //                 )
        //                 .await;
        //             }
        //             ChainConfig::Evm(EvmChainConfig::Mainnet(evm_config)) => {
        //                 chain::evm::balance_of(
        //                     &Evm::<Mainnet>::new(evm_config.clone().into()).await,
        //                     evm_config.ics20_bank_address.clone(),
        //                     who.into(),
        //                     denom,
        //                 )
        //                 .await;
        //             }
        //             ChainConfig::Union(_) => bail!("not currently supported"),
        //         }
        //     }
        //     QueryCmd::Client { on, client_id } => {
        //         let json = match voyager_config.chain[&on].clone() {
        //             ChainConfig::Evm(EvmChainConfig::Mainnet(evm)) => {
        //                 let evm = Evm::<Mainnet>::new(evm.into()).await;

        //                 let cometbls = evm.light_client();

        //                 serde_json::to_string_pretty(
        //                     &cometbls
        //                         .query_client_state(client_id.parse().unwrap())
        //                         .await,
        //                 )
        //                 .unwrap()
        //             }
        //             ChainConfig::Evm(EvmChainConfig::Minimal(evm)) => {
        //                 let evm = Evm::<Minimal>::new(evm.into()).await;

        //                 let cometbls = evm.light_client();

        //                 serde_json::to_string_pretty(
        //                     &cometbls
        //                         .query_client_state(client_id.parse().unwrap())
        //                         .await,
        //                 )
        //                 .unwrap()
        //             }
        //             ChainConfig::Union(union) => {
        //                 let union = Union::new(union.into()).await;

        //                 let ethereum: Ethereum<Mainnet> = union.light_client();

        //                 serde_json::to_string_pretty(
        //                     &ethereum
        //                         .query_client_state(client_id.parse().unwrap())
        //                         .await,
        //                 )
        //                 .unwrap()
        //             }
        //         };

        //         println!("{json}");
        //     }
        //     QueryCmd::Connection {} => todo!(),
        //     QueryCmd::Channel {} => todo!(),
        // },
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
                        chain::evm::bind_port(&evm, module_address.into(), port_id).await
                    }
                    AnyChain::EvmMainnet(evm) => {
                        chain::evm::bind_port(&evm, module_address.into(), port_id).await
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
                        chain::evm::setup_initial_channel(
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
            // cli::SetupCmd::SetOperator { on } => {
            //     let chain_config = voyager_config.chain.get(&on).unwrap();

            //     match chain_config {
            //         ChainConfig::Evm(EvmChainConfig::Minimal(evm_config)) => {
            //             chain::evm::ics20_bank_set_operator(
            //                 &Evm::<Minimal>::new(evm_config.clone().into()).await,
            //                 evm_config.ics20_bank_address.clone(),
            //                 evm_config.ics20_transfer_bank_address.clone(),
            //             )
            //             .await;
            //         }
            //         _ => panic!("Not supported."),
            //     }
            // }
            _ => panic!("not supported"),
        },
        Command::Ibc(IbcCmd::Query {
            on,
            at,
            cmd: IbcQueryCmd::Path(path),
        }) => {
            let json = match voyager_config.chain[&on].clone() {
                // ChainConfig::Evm(EvmChainConfig::Mainnet(evm)) => {
                //     let evm = Evm::<Mainnet>::new(evm.into()).await;

                //     path.any_state_proof_to_json::<Union, _>(evm, at).await
                // }
                ChainConfig::Evm(EvmChainConfig::Minimal(evm)) => {
                    let evm = Evm::<Minimal>::new(evm.into()).await;

                    path.any_state_proof_to_json::<Union, _>(evm, at).await
                }
                ChainConfig::Union(union) => {
                    let union = Union::new(union.into()).await;

                    // NOTE: ChainSpec is arbitrary
                    path.any_state_proof_to_json::<Evm<Mainnet>, _>(union, at)
                        .await
                }
                _ => panic!(),
            };

            println!("{json}");
        }
        _ => panic!(),
    }

    std::fs::write(
        args.config_file_path,
        serde_json::to_string_pretty(&voyager_config).unwrap(),
    )
    .unwrap();

    Ok(())
}
