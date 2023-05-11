use crate::{chain::Chain, client::LightClient, context::Context};
use anyhow::Result;
use clap::Parser;
use ethereum_consensus::types::H256;

#[derive(Clone, Debug, Parser, PartialEq)]
pub struct InitCommand {
    #[clap(long = "trusted_block_root", help = "Trusted block root")]
    pub trusted_block_root: Option<String>,
    #[clap(long = "untrusted_slot", help = "Untrusted slot")]
    pub untrusted_slot: Option<u64>,
}

impl InitCommand {
    pub async fn run<
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
        const SYNC_COMMITTEE_SIZE: usize,
    >(
        self,
        ctx: Context<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES, SYNC_COMMITTEE_SIZE>,
    ) -> Result<()> {
        let chain = Chain::new(ctx.beacon_endpoint());
        let trusted_block_root = if let Some(trusted_block_root) = self.trusted_block_root {
            Some(H256::from_hex(&trusted_block_root)?)
        } else if let Some(untrusted_slot) = self.untrusted_slot {
            Some(
                chain
                    .rpc_client
                    .get_beacon_header_by_slot(untrusted_slot.into())
                    .await?
                    .data
                    .root,
            )
        } else {
            None
        };

        let genesis = chain.rpc_client.get_genesis().await?.data;
        let lc = LightClient::new(
            ctx,
            chain,
            genesis.genesis_time,
            genesis.genesis_validators_root,
            None,
        );
        lc.init_with_bootstrap(trusted_block_root).await?;
        Ok(())
    }
}
