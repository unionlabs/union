use anyhow::Result;
use clap::Subcommand;

pub mod rpc;

#[derive(Debug, Subcommand)]
pub enum Cmd {
    #[command(visible_alias = "r")]
    Rpc(rpc::Cmd),
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        match self {
            Cmd::Rpc(cmd) => cmd.run().await,
        }
    }
}
