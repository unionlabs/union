use anyhow::Result;
use clap::Subcommand;

pub mod instantiate2_address;
pub mod wrapped_token;

#[derive(Debug, Subcommand)]
pub enum Cmd {
    #[command(visible_alias = "wt")]
    WrappedToken(wrapped_token::Cmd),
    #[command(visible_alias = "cwi2")]
    Instantiate2Address(instantiate2_address::Cmd),
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        match self {
            Cmd::WrappedToken(args) => args.run().await,
            Cmd::Instantiate2Address(cmd) => cmd.run(),
        }
    }
}
