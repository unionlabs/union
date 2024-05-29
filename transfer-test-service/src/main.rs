use std::fs::read_to_string;

use clap::Parser;
use config::Config;
use context::{ Context, IbcTransfer, TransferDirection };
use sqlx::PgPool;

pub mod config;
pub mod context;
pub mod datadog;
pub mod sql_helper;
use std::ffi::OsString;

use tokio::{ signal, time::{ interval, Duration } };

use crate::sql_helper::create_table_if_not_exists; //, events::{ EventType } };

/// Arguments provided to the top-level command.
#[derive(Debug, Parser, Clone)]
pub struct AppArgs {
    /// The path to the configuration file.
    #[arg(long, short = 'c', global = true, default_value = "transfer-test-service-config.json")]
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

    let pool: sqlx::Pool<sqlx::Postgres> = PgPool::connect(&args.database_url).await.unwrap();

    create_table_if_not_exists(&pool).await.unwrap();
    let transfer_test_config: Config = serde_json
        ::from_str(&read_to_string(args.config).unwrap())
        .unwrap();

    println!("{}", serde_json::to_string_pretty(&transfer_test_config).unwrap());
    let output = "transfer-test-service.csv";
    let context = Context::new(transfer_test_config.clone(), output.to_string(), pool).await;

    // Task to handle listening for events
    for connection in &transfer_test_config.connections {
        let context_clone = context.clone();
        let source_chain = connection.source_chain.clone();
        let target_chain = connection.target_chain.clone();
        tokio::spawn(async move {
            context_clone.listen(source_chain.as_str(), target_chain.as_str()).await;
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

                let direction = match (source_chain.as_str(), target_chain.as_str()) {
                    ("osmosis", "union") | ("union", "osmosis") => {
                        TransferDirection::CosmosToCosmos {
                            source_chain: source_chain.clone(),
                            target_chain: target_chain.clone(),
                            channel: transfer_test_config_clone.channel.clone(),
                            contract: if source_chain == "osmosis" {
                                transfer_test_config_clone.osmosis_contract.clone()
                            } else {
                                transfer_test_config_clone.union_contract.clone()
                            },
                            receiver_bech32: if target_chain == "osmosis" {
                                transfer_test_config_clone.osmosis_contract.clone()
                            } else {
                                transfer_test_config_clone.union_contract.clone()
                            },
                            denom: if source_chain == "osmosis" {
                                transfer_test_config_clone.osmosis.fee_denom.clone()
                            } else {
                                transfer_test_config_clone.union.fee_denom.clone()
                            },
                            amount: transfer_test_config_clone.amount.clone(),
                        }
                    }
                    ("ethereum", "osmosis") | ("osmosis", "ethereum") => {
                        TransferDirection::EthToCosmos {
                            // Fill in with appropriate details for Ethereum to Cosmos transfer
                        }
                    }
                    ("union", "ethereum") | ("ethereum", "union") => {
                        TransferDirection::CosmosToEth {
                            // Fill in with appropriate details for Cosmos to Ethereum transfer
                        }
                    }
                    _ => {
                        eprintln!("Unsupported connection: {} -> {}", source_chain, target_chain);
                        continue;
                    }
                };

                context_clone.send_ibc_transfer(direction).await;
            }
        });
    }

    // Start packet monitoring task
    for connection in &transfer_test_config.connections {
        let source_chain = connection.source_chain.clone();
        let target_chain = connection.target_chain.clone();
        let expect_full_circle = connection.expect_full_circle as u64;
        let context_clone = context.clone();

        context_clone.check_packet_sequences(
            source_chain.as_str(),
            target_chain.as_str(),
            expect_full_circle
        ).await;
    }

    signal::ctrl_c().await.unwrap();
}
