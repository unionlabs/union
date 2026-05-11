use anyhow::Result;
use clap::Subcommand;

pub mod lightclient;
pub mod rpc;

#[derive(Debug, Subcommand)]
pub enum Cmd {
    #[command(visible_alias = "r")]
    Rpc(rpc::Cmd),
    #[command(visible_alias = "lc")]
    LightClient(lightclient::Cmd),
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        match self {
            Cmd::Rpc(cmd) => cmd.run().await,
            Cmd::LightClient(cmd) => cmd.run().await,
        }
    }
}
