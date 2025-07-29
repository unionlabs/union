use anyhow::Result;
use clap::Subcommand;

pub mod fungible_asset_order;

#[derive(Debug, Subcommand)]
pub enum Cmd {
    #[command(visible_alias = "fao", subcommand)]
    TokenOrderV1(fungible_asset_order::Cmd),
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        match self {
            Cmd::TokenOrderV1(cmd) => cmd.run(),
        }
    }
}
