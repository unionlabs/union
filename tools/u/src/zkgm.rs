use anyhow::Result;
use clap::Subcommand;

pub mod make;
pub mod predict_wrapped_token;

#[derive(Debug, Subcommand)]
pub enum Cmd {
    #[command(visible_alias = "pwt")]
    PredictWrappedToken(predict_wrapped_token::Cmd),
    #[command(visible_alias = "mk", subcommand)]
    Make(make::Cmd),
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        match self {
            Cmd::PredictWrappedToken(cmd) => cmd.run().await,
            Cmd::Make(cmd) => cmd.run(),
        }
    }
}
