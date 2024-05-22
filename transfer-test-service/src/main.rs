use std::fs::read_to_string;

use clap::Parser;
use config::Config;
use context::Context;

pub mod config;
pub mod datadog;
pub mod context;
use tokio::time::{ interval, Duration };
use tokio::signal;

use std::ffi::OsString;

/// Arguments provided to the top-level command.
#[derive(Debug, Parser, Clone)]
pub struct AppArgs {
    /// The path to the configuration file.
    #[arg(long, short = 'c', global = true, default_value = "transfer-test-service-config.json")]
    pub config: OsString,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = AppArgs::parse();

    let transfer_test_config: Config = serde_json
        ::from_str(&read_to_string(args.config).unwrap())
        .unwrap();

    println!("{}", serde_json::to_string_pretty(&transfer_test_config).unwrap());
    let output = "transfer-test-service.csv";
    let context = Context::new(transfer_test_config, output.to_string()).await;

    let context_clone = context.clone();

    // Task to handle listening for events
    tokio::spawn(async move {
        context_clone.listen().await;
    });

    let context_for_send = context.clone();

    // Task to send transactions every minute
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            context_for_send.send_ibc_transfer_from_osmosis_to_union().await;
        }
    });

    // Start packet monitoring task
    context.start_packet_monitoring().await;

    // Keep the main task alive to allow other tasks to run
    signal::ctrl_c().await.unwrap();
}
