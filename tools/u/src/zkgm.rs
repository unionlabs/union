use anyhow::Result;
use clap::Subcommand;

pub mod decode;
pub mod make;
pub mod predict_proxy_account_address;
pub mod predict_wrapped_token;

#[derive(Debug, Subcommand)]
pub enum Cmd {
    #[command(visible_alias = "pwt")]
    PredictWrappedToken(predict_wrapped_token::Cmd),
    #[command(visible_alias = "proxy")]
    PredictProxyAccountAddress(predict_proxy_account_address::Cmd),
    #[command(visible_alias = "mk", subcommand)]
    Make(make::Cmd),
    #[command(visible_alias = "d")]
    Decode(decode::Cmd),
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        match self {
            Cmd::PredictWrappedToken(cmd) => cmd.run().await,
            Cmd::PredictProxyAccountAddress(cmd) => cmd.run().await,
            Cmd::Make(cmd) => cmd.run(),
            Cmd::Decode(cmd) => cmd.run(),
        }
    }
}
