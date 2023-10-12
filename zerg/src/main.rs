use std::fs::read_to_string;

use clap::Parser;
use cli::AppArgs;
use config::Config;
use context::Context;
use unionlabs::ethereum_consts_traits::Minimal;

pub mod cli;
pub mod config;
pub mod context;
pub mod events;

#[tokio::main]
async fn main() {
    let args = AppArgs::parse();

    let zerg_config: Config =
        serde_json::from_str(&read_to_string(args.config_file_path).unwrap()).unwrap();
    let evm = chain_utils::evm::Evm::<Minimal>::new(zerg_config.evm.clone()).await;

    let context = Context {
        output_file: "output.csv".to_string(),
        zerg_config,
        evm,
    };

    let context2 = context.clone();

    let _ = tokio::join!(
        context.listen_union(),
        context.listen_eth(),
        tokio::spawn(async move {
            context2.tx_handler().await;
        })
    );
}
