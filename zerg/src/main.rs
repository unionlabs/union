use std::fs::read_to_string;

use analyze::analyze;
use clap::Parser;
use cli::AppArgs;
use config::Config;
use context::Context;

pub mod analyze;
pub mod cli;
pub mod config;
pub mod context;
pub mod events;

#[tokio::main]
async fn main() {
    let args = AppArgs::parse();

    let zerg_config: Config = serde_json::from_str(&read_to_string(args.config).unwrap()).unwrap();
    let is_rush = matches!(args.command, cli::Command::Rush { output: _ });

    match args.command {
        cli::Command::PrintConfig => {
            println!("{}", serde_json::to_string_pretty(&zerg_config).unwrap());
        }
        cli::Command::Rush { output: _ } => {
            let context = Context::new(zerg_config, is_rush).await;
            let _ = tokio::join!(
                context.listen_union(),
                context.listen_eth(),
                context.tx_handler()
            );
        }
        cli::Command::Observe { output: _ } => {
            let context = Context::new(zerg_config, is_rush).await;
            let _ = tokio::join!(context.listen_union(), context.listen_eth(),);
        }
        cli::Command::Analyze {
            input_file,
            output: _,
        } => analyze(input_file),
    };
}
