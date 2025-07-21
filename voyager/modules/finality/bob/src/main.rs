#![warn(clippy::unwrap_used)]

use alloy::{
    eips::BlockId,
    network::AnyNetwork,
    providers::{layers::CacheLayer, DynProvider, Provider, ProviderBuilder},
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use unionlabs::{ibc::core::client::height::Height, primitives::H160, ErrorReporter};
use voyager_sdk::{
    anyhow,
    plugin::FinalityModule,
    primitives::{ChainId, ConsensusType, Timestamp},
    rpc::{types::FinalityModuleInfo, FinalityModuleServer},
    ExtensionsExt,
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub l1_chain_id: ChainId,

    pub l1_dispute_game_factory_proxy: H160,

    pub l1_provider: DynProvider,
    pub l2_provider: DynProvider<AnyNetwork>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The chain id of the chain this bob chain chain settles on.
    pub l1_chain_id: ChainId,

    pub l1_dispute_game_factory_proxy: H160,

    /// The RPC endpoint for the settlement (L1) execution chain.
    pub l1_rpc_url: String,

    /// The RPC endpoint for the main (L2) execution chain.
    pub l2_rpc_url: String,

    #[serde(default)]
    pub max_cache_size: u32,
}

impl FinalityModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: FinalityModuleInfo) -> anyhow::Result<Self> {
        let l1_provider = DynProvider::new(
            ProviderBuilder::new()
                .layer(CacheLayer::new(config.max_cache_size))
                .connect(&config.l1_rpc_url)
                .await?,
        );

        let l2_provider = DynProvider::new(
            ProviderBuilder::new()
                .layer(CacheLayer::new(config.max_cache_size))
                .network::<AnyNetwork>()
                .connect(&config.l2_rpc_url)
                .await?,
        );

        let l1_chain_id = ChainId::new(l1_provider.get_chain_id().await?.to_string());
        let l2_chain_id = ChainId::new(l2_provider.get_chain_id().await?.to_string());

        info.ensure_chain_id(l2_chain_id.to_string())?;
        info.ensure_consensus_type(ConsensusType::BOB)?;

        Ok(Self {
            chain_id: l2_chain_id,
            l1_chain_id,
            l1_dispute_game_factory_proxy: config.l1_dispute_game_factory_proxy,
            l1_provider,
            l2_provider,
        })
    }
}

#[async_trait]
impl FinalityModuleServer for Module {
    /// Query the latest finalized height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id, finalized))]
    async fn query_latest_height(&self, e: &Extensions, finalized: bool) -> RpcResult<Height> {
        if finalized {
            let voyager_client = e.voyager_client()?;

            let l1_latest_height = voyager_client
                .query_latest_height(self.l1_chain_id.clone(), true)
                .await?;

            let block_number = bob_client::finalized_l2_block_number_of_l1_block_number(
                &self.l1_provider,
                self.l1_dispute_game_factory_proxy,
                l1_latest_height.height(),
            )
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e)
                        .with_message("error fetching finalized l2 execution block of l1 height"),
                    None::<()>,
                )
            })?;

            Ok(Height::new(block_number))
        } else {
            self.l2_provider
                .get_block_number()
                .await
                .map(Height::new)
                .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))
        }
    }

    /// Query the latest finalized timestamp of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id, finalized))]
    async fn query_latest_timestamp(
        &self,
        e: &Extensions,
        finalized: bool,
    ) -> RpcResult<Timestamp> {
        if finalized {
            let voyager_client = e.voyager_client()?;

            let l1_latest_height = voyager_client
                .query_latest_height(self.l1_chain_id.clone(), true)
                .await?;

            let block_number = bob_client::finalized_l2_block_number_of_l1_block_number(
                &self.l1_provider,
                self.l1_dispute_game_factory_proxy,
                l1_latest_height.height(),
            )
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e)
                        .with_message("error fetching finalized l2 execution block of l1 height"),
                    None::<()>,
                )
            })?;

            let block = self
                .l2_provider
                .get_block(block_number.into())
                .await
                .map_err(|e| {
                    ErrorObject::owned(
                        -1,
                        ErrorReporter(e).with_message("error fetching finalized l2 block"),
                        None::<()>,
                    )
                })?
                .expect("block should exist");

            Ok(Timestamp::from_secs(block.header.timestamp))
        } else {
            self.l2_provider
                .get_block(BlockId::latest())
                .await
                .map(|b| Timestamp::from_secs(b.expect("block exists").header.timestamp))
                .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))
        }
    }
}
