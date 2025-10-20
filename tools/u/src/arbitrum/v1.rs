use alloy::{network::AnyNetwork, providers::ProviderBuilder};
use anyhow::Result;
use arbitrum_client::v1::{finalized_l2_block_of_l1_height, next_node_num_at_l1_height};
use clap::Subcommand;
use unionlabs::primitives::H160;

use crate::print_json;

#[derive(Debug, Subcommand)]
pub enum Cmd {
    NextNodeNumAtL1Height {
        #[arg(long)]
        l1_rpc_url: String,
        #[arg(long)]
        l1_contract_address: H160,
        #[arg(long)]
        l1_height: u64,
    },
    FinalizedL2BlockOfL1Height {
        #[arg(long)]
        l1_rpc_url: String,
        #[arg(long)]
        l2_rpc_url: String,
        #[arg(long)]
        l1_contract_address: H160,
        #[arg(long)]
        l1_height: u64,
    },
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        match self {
            Cmd::NextNodeNumAtL1Height {
                l1_rpc_url,
                l1_contract_address,
                l1_height,
            } => {
                let next_node_num = next_node_num_at_l1_height(
                    ProviderBuilder::new().connect(&l1_rpc_url).await?,
                    l1_contract_address,
                    l1_height,
                )
                .await
                .map_err(anyhow::Error::from_boxed)?;

                print_json(&next_node_num);
            }
            Cmd::FinalizedL2BlockOfL1Height {
                l1_rpc_url,
                l2_rpc_url,
                l1_contract_address,
                l1_height,
            } => {
                let l2_block = finalized_l2_block_of_l1_height(
                    ProviderBuilder::new().connect(&l1_rpc_url).await?,
                    ProviderBuilder::new()
                        .network::<AnyNetwork>()
                        .connect(&l2_rpc_url)
                        .await?,
                    l1_contract_address,
                    l1_height,
                )
                .await
                .map_err(anyhow::Error::from_boxed)?;

                print_json(&l2_block);
            }
        }

        Ok(())
    }
}
