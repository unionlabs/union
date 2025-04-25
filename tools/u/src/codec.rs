use anyhow::Result;
use clap::Subcommand;

pub mod lightclient;

#[derive(Debug, Subcommand)]
pub enum Cmd {
    #[command(visible_alias = "lc", subcommand)]
    LightClient(lightclient::Cmd),
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        match self {
            Cmd::LightClient(cmd) => cmd.run(),
        }
    }
}
