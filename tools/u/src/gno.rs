use anyhow::Result;
use clap::Subcommand;
use deployments::H256;

pub mod lightclient;
pub mod rpc;

#[derive(Debug, Subcommand)]
pub enum Cmd {
    #[command(visible_alias = "r")]
    Rpc(rpc::Cmd),
    #[command(visible_alias = "lc")]
    LightClient(lightclient::Cmd),
    #[command(visible_alias = "bz")]
    RawBytes {
        input: H256,
        #[arg(long)]
        fixed: bool,
    },
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        match self {
            Cmd::Rpc(cmd) => cmd.run().await,
            Cmd::LightClient(cmd) => cmd.run().await,
            Cmd::RawBytes { input, fixed } => {
                println!("{}", gno_bytes(input.get(), fixed));
                Ok(())
            }
        }
    }
}

fn gno_bytes(bz: &[u8], fixed: bool) -> String {
    format!(
        "[{}]byte{{{}}}",
        if fixed {
            bz.len().to_string()
        } else {
            "".to_owned()
        },
        bz.iter().map(|b| b.to_string() + ", ").collect::<String>()
    )
}
