use std::fs::read_to_string;

use analyze::analyze;
use clap::Parser;
use cli::AppArgs;
use config::Config;
use context::Context;
use unionlabs::ethereum_consts_traits::Minimal;

pub mod analyze;
pub mod cli;
pub mod config;
pub mod context;
pub mod events;

#[tokio::main]
async fn main() {
    let args = AppArgs::parse();

    let zerg_config: Config =
        serde_json::from_str(&read_to_string(args.config_file_path).unwrap()).unwrap();

    match args.command {
        cli::Command::PrintConfig => {
            println!("{}", serde_json::to_string_pretty(&zerg_config).unwrap());
        }
        cli::Command::Rush => {
            let evm = chain_utils::evm::Evm::<Minimal>::new(zerg_config.evm.clone())
                .await
                .unwrap();

            let is_rush = matches!(args.command, cli::Command::Rush);
            let context = Context {
                output_file: "output.csv".to_string(),
                zerg_config,
                evm,
                is_rush,
            };
            let _ = tokio::join!(
                context.listen_union(),
                context.listen_eth(),
                context.tx_handler()
            );
        }
        cli::Command::Observe => {
            let evm = chain_utils::evm::Evm::<Minimal>::new(zerg_config.evm.clone())
                .await
                .unwrap();

            let is_rush = matches!(args.command, cli::Command::Rush);
            let context = Context {
                output_file: "output.csv".to_string(),
                zerg_config,
                evm,
                is_rush,
            };
            let _ = tokio::join!(context.listen_union(), context.listen_eth(),);
        }
        cli::Command::Analyze { input_file } => analyze(input_file),
    };
}
