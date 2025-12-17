use std::sync::LazyLock;

use anyhow::{Result, anyhow};
use clap::Subcommand;
use deployments::Deployments;
use ucs04::UniversalChainId;

use crate::print_json;

pub static DEPLOYMENTS: LazyLock<Deployments<'static>> = LazyLock::new(|| {
    serde_json::from_slice(include_bytes!("../../../deployments/deployments.json")).unwrap()
});

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
