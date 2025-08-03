use anyhow::Result;
use clap::Subcommand;

pub mod create3;
pub mod instantiate2_address;
pub mod wrapped_token;

#[derive(Debug, Subcommand)]
pub enum Cmd {
    #[command(visible_alias = "wt")]
    WrappedToken(wrapped_token::Cmd),
    #[command(visible_alias = "i2")]
    Instantiate2Address(instantiate2_address::Cmd),
    #[command(visible_alias = "c3")]
    Create3(create3::Cmd),
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        match self {
            Cmd::WrappedToken(args) => args.run().await,
            Cmd::Instantiate2Address(cmd) => cmd.run(),
            Cmd::Create3(cmd) => cmd.run(),
        }
    }
}
