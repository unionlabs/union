use alloy::{network::AnyNetwork, providers::ProviderBuilder};
use anyhow::Result;
use arbitrum_client::v2::{
    assertion_created_event_at_l1_block_height, finalized_l2_block_of_l1_height,
};
use clap::Subcommand;
use unionlabs::primitives::{H160, H256};

use crate::print_json;

#[derive(Debug, Subcommand)]
pub enum Cmd {
    AssertionCreatedEventAtL1BlockHeight {
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
    AssertionsSlot {
        assertion_hash: H256,
    },
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        match self {
            Cmd::AssertionCreatedEventAtL1BlockHeight {
                l1_rpc_url,
                l1_contract_address,
                l1_height,
            } => {
                let assertion_created_event = assertion_created_event_at_l1_block_height(
                    ProviderBuilder::new().connect(&l1_rpc_url).await?,
                    l1_contract_address,
                    l1_height,
                )
                .await
                .map_err(anyhow::Error::from_boxed)?;

                print_json(&assertion_created_event);
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
            Cmd::AssertionsSlot { assertion_hash } => {
                print_json(&arbitrum_types::v2::rollup_core_assertions_slot(
                    assertion_hash,
                ));
            }
        }

        Ok(())
    }
}
