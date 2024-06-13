use std::fs::read_to_string;

use clap::Parser;
use config::Config;
use context::{Context, IbcTransfer, TransferDirection};
use sqlx::PgPool;

pub mod config;
pub mod context;
// pub mod datadog;
pub mod sql_helper;
use std::ffi::OsString;

use tokio::{
    signal,
    time::{interval, Duration},
};

use crate::{
    config::{ChainConfig, CosmosConfig},
    sql_helper::create_table_if_not_exists,
}; //, events::{ EventType } };

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

    /// The database URL
    #[arg(
        long,
        short = 'd',
        global = true,
        default_value = "postgres://postgres:postgrespassword@127.0.0.1:5432/default" //"postgres://user:password@localhost/dbname"
    )]
    pub database_url: String,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = AppArgs::parse();
    tracing::debug!("args.config: {:?}", args.config);
    let transfer_test_config: Config =
        serde_json::from_str(&read_to_string(args.config).unwrap()).unwrap();
    tracing::debug!(
        "All json: {}",
        serde_json::to_string_pretty(&transfer_test_config).unwrap()
    );

    let pool: sqlx::Pool<sqlx::Postgres> =
        PgPool::connect(&transfer_test_config.db_url).await.unwrap();

    create_table_if_not_exists(&pool).await.unwrap();
    tracing::debug!(
        "{}",
        serde_json::to_string_pretty(&transfer_test_config).unwrap()
    );
    let context = Context::new(transfer_test_config.clone(), pool).await;

    // Task to handle listening for events
    for connection in &transfer_test_config.connections {
        let context_clone = context.clone();
        let source_chain = connection.source_chain.clone();
        let target_chain = connection.target_chain.clone();
        tokio::spawn(async move {
            context_clone
                .listen(source_chain.as_str(), target_chain.as_str())
                .await;
        });
    }

    // Parse connections from config and spawn tasks for each connection
    for connection in &transfer_test_config.connections {
        let source_chain = connection.source_chain.clone();
        let target_chain = connection.target_chain.clone();
        let send_packet_interval = connection.send_packet_interval as u64;
        let context_clone = context.clone();

        let transfer_test_config_clone = transfer_test_config.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(send_packet_interval));
            loop {
                interval.tick().await;

                match (source_chain.as_str(), target_chain.as_str()) {
                    ("osmosis", "union") => {
                        context_clone
                            .send_ibc_transfer(TransferDirection {
                                source_chain: CosmosConfig {
                                    protocol: transfer_test_config_clone.osmosis.protocol.clone(),
                                    chain_config: ChainConfig {
                                        chain_config: context_clone.osmosis.clone().unwrap(),
                                        address: transfer_test_config_clone
                                            .osmosis
                                            .chain_config
                                            .address
                                            .clone(),
                                        channel: transfer_test_config_clone
                                            .osmosis
                                            .chain_config
                                            .channel
                                            .clone(),
                                        counterparty_channel: transfer_test_config_clone
                                            .osmosis
                                            .chain_config
                                            .counterparty_channel
                                            .clone(),
                                    },
                                },
                                destination_chain: CosmosConfig {
                                    protocol: transfer_test_config_clone.union.protocol.clone(),
                                    chain_config: ChainConfig {
                                        chain_config: context_clone.union.clone().unwrap(),
                                        address: transfer_test_config_clone
                                            .union
                                            .chain_config
                                            .address
                                            .clone(),
                                        channel: transfer_test_config_clone
                                            .union
                                            .chain_config
                                            .channel
                                            .clone(),
                                        counterparty_channel: transfer_test_config_clone
                                            .union
                                            .chain_config
                                            .counterparty_channel
                                            .clone(),
                                    },
                                },
                            })
                            .await;
                    }
                    ("union", "osmosis") => {
                        context_clone
                            .send_ibc_transfer(TransferDirection {
                                source_chain: CosmosConfig {
                                    protocol: transfer_test_config_clone.union.protocol.clone(),
                                    chain_config: ChainConfig {
                                        chain_config: context_clone.union.clone().unwrap(),
                                        address: transfer_test_config_clone
                                            .union
                                            .chain_config
                                            .address
                                            .clone(),
                                        channel: transfer_test_config_clone
                                            .union
                                            .chain_config
                                            .channel
                                            .clone(),
                                        counterparty_channel: transfer_test_config_clone
                                            .union
                                            .chain_config
                                            .counterparty_channel
                                            .clone(),
                                    },
                                },
                                destination_chain: CosmosConfig {
                                    protocol: transfer_test_config_clone.osmosis.protocol.clone(),
                                    chain_config: ChainConfig {
                                        chain_config: context_clone.osmosis.clone().unwrap(),
                                        address: transfer_test_config_clone
                                            .osmosis
                                            .chain_config
                                            .address
                                            .clone(),
                                        channel: transfer_test_config_clone
                                            .osmosis
                                            .chain_config
                                            .channel
                                            .clone(),
                                        counterparty_channel: transfer_test_config_clone
                                            .osmosis
                                            .chain_config
                                            .counterparty_channel
                                            .clone(),
                                    },
                                },
                            })
                            .await;
                    }
                    ("ethereum", "union") => {
                        context_clone
                            .send_ibc_transfer(TransferDirection {
                                source_chain: ChainConfig::<_, ethers::types::H160> {
                                    chain_config: (
                                        transfer_test_config_clone.ethereum.chain_config.signers[0]
                                            .clone(),
                                        context_clone.ethereum.clone().unwrap(),
                                    ),

                                    address: transfer_test_config_clone
                                        .ethereum
                                        .address
                                        .clone()
                                        .into(),
                                    channel: transfer_test_config_clone.ethereum.channel.clone(),
                                    counterparty_channel: transfer_test_config_clone
                                        .ethereum
                                        .counterparty_channel
                                        .clone(),
                                },
                                destination_chain: CosmosConfig {
                                    protocol: transfer_test_config_clone.union.protocol.clone(),
                                    chain_config: ChainConfig {
                                        chain_config: context_clone.union.clone().unwrap(),
                                        address: transfer_test_config_clone
                                            .union
                                            .chain_config
                                            .address
                                            .clone(),
                                        channel: transfer_test_config_clone
                                            .union
                                            .chain_config
                                            .channel
                                            .clone(),
                                        counterparty_channel: transfer_test_config_clone
                                            .union
                                            .chain_config
                                            .counterparty_channel
                                            .clone(),
                                    },
                                },
                            })
                            .await;
                    }
                    ("union", "ethereum") => {
                        context_clone
                            .send_ibc_transfer(TransferDirection {
                                destination_chain: ChainConfig::<_, ethers::types::H160> {
                                    chain_config: (
                                        transfer_test_config_clone.ethereum.chain_config.signers[0]
                                            .clone(),
                                        context_clone.ethereum.clone().unwrap(),
                                    ),

                                    address: transfer_test_config_clone
                                        .ethereum
                                        .address
                                        .clone()
                                        .into(),
                                    channel: transfer_test_config_clone.ethereum.channel.clone(),
                                    counterparty_channel: transfer_test_config_clone
                                        .ethereum
                                        .counterparty_channel
                                        .clone(),
                                },
                                source_chain: CosmosConfig {
                                    protocol: transfer_test_config_clone.union.protocol.clone(),
                                    chain_config: ChainConfig {
                                        chain_config: context_clone.union.clone().unwrap(),
                                        address: transfer_test_config_clone
                                            .union
                                            .chain_config
                                            .address
                                            .clone(),
                                        channel: transfer_test_config_clone
                                            .union
                                            .chain_config
                                            .channel
                                            .clone(),
                                        counterparty_channel: transfer_test_config_clone
                                            .union
                                            .chain_config
                                            .counterparty_channel
                                            .clone(),
                                    },
                                },
                            })
                            .await;
                    }

                    // ("ethereum", "union") => {
                    //     context_clone.send_ibc_transfer(TransferDirection {
                    //         source_chain: transfer_test_config_clone.ethereum,
                    //         destination_chain: transfer_test_config_clone.union,
                    //     }).await;
                    // }
                    // ("union", "ethereum") => {
                    //     context_clone.send_ibc_transfer(TransferDirection {
                    //         source_chain: transfer_test_config_clone.union,
                    //         destination_chain: transfer_test_config_clone.ethereum,
                    //     }).await;
                    // }
                    _ => {
                        tracing::error!(
                            "Unsupported connection: {} -> {}",
                            source_chain,
                            target_chain
                        );
                        continue;
                    }
                }

                // context_clone.send_ibc_transfer(direction).await;
            }
        });
    }

    // Start packet monitoring task
    for connection in &transfer_test_config.connections {
        let source_chain = connection.source_chain.clone();
        let target_chain = connection.target_chain.clone();
        let expect_full_circle = connection.expect_full_circle as u64;
        let context_clone = context.clone();

        context_clone
            .check_packet_sequences(
                source_chain.as_str(),
                target_chain.as_str(),
                expect_full_circle,
            )
            .await;
    }

    signal::ctrl_c().await.unwrap();
}
