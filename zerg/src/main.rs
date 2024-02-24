use std::fs::read_to_string;

use analyze::analyze;
use clap::Parser;
use cli::AppArgs;
use config::Config;
use context::Context;
use process::process;

pub mod analyze;
pub mod cli;
pub mod config;
pub mod context;
pub mod events;
pub mod process;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = AppArgs::parse();

    let zerg_config: Config = serde_json::from_str(&read_to_string(args.config).unwrap()).unwrap();
    let is_rush = matches!(args.command, cli::Command::Rush { output: _ });

    match args.command {
        cli::Command::PrintConfig => {
            println!("{}", serde_json::to_string_pretty(&zerg_config).unwrap());
        }
        cli::Command::Rush { output } => {
            let context = Context::new(zerg_config, output, is_rush).await;
            let _ = tokio::join!(context.listen(), context.tx_handler());
        }
        cli::Command::Process {
            input_file,
            output: _,
        } => {
            let _ = process(input_file);
        }
        cli::Command::Analyze { input_file, output } => {
            let _ = analyze(input_file, output);
        }
    };
}
