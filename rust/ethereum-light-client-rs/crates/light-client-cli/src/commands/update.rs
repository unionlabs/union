use crate::{
    chain::Chain,
    client::{LightClient, Target},
    context::Context,
};
use anyhow::Result;
use clap::Parser;
use core::time::Duration;

#[derive(Clone, Debug, Parser, PartialEq)]
pub struct UpdateCommand {
    #[clap(long = "target")]
    target: Option<String>,
}

impl UpdateCommand {
    pub async fn run<
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
        const SYNC_COMMITTEE_SIZE: usize,
    >(
        self,
        ctx: Context<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES, SYNC_COMMITTEE_SIZE>,
    ) -> Result<()> {
        let chain = Chain::new(ctx.beacon_endpoint());
        let target = if let Some(target) = self.target {
            Target::from_string(&ctx, &target)?
        } else {
            Target::None
        };

        let genesis = chain.rpc_client.get_genesis().await?.data;
        let lc = LightClient::new(
            ctx,
            chain,
            genesis.genesis_time,
            genesis.genesis_validators_root,
            None,
        );

        let _ = lc
            .update_until_target(target, Duration::from_secs(5))
            .await?;
        Ok(())
    }
}
