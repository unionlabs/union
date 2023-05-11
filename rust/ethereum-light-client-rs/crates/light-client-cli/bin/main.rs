use clap::Parser;
use ethereum_light_client_cli::cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    Ok(Cli::parse().run().await?)
}
