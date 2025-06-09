use anyhow::{anyhow, Result};
use clap::Subcommand;
use ucs04::UniversalChainId;

use crate::print_json;

#[derive(Debug, Subcommand)]
pub enum Cmd {
    Print {
        chain_id: Option<UniversalChainId<'static>>,
    },
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        match self {
            Cmd::Print { chain_id } => match chain_id {
                Some(chain_id) => {
                    print_json(
                        &DEPLOYMENTS
                            .get(&chain_id)
                            .ok_or_else(|| anyhow!("chain {chain_id} not found"))?,
                    );
                }
                None => {
                    print_json(&*DEPLOYMENTS);
                }
            },
        };

        Ok(())
    }
}
