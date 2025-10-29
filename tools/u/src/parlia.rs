use anyhow::Result;
use clap::Subcommand;

pub mod parse_extra_data;

#[derive(Debug, Subcommand)]
pub enum Cmd {
    #[clap(visible_alias = "ed")]
    ParseExtraData(parse_extra_data::Cmd),
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        match self {
            Cmd::ParseExtraData(cmd) => cmd.run().await,
        }
    }
}
