#![warn(clippy::unwrap_used)]

use alloy::{
    eips::BlockNumberOrTag,
    providers::{layers::CacheLayer, DynProvider, Provider, ProviderBuilder},
    rpc::types::BlockTransactionsKind,
};
use beacon_api::client::BeaconApiClient;
use beacon_api_types::{chain_spec::PresetBaseKind, slot::Slot};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use tracing::{instrument, trace};
use unionlabs::{ibc::core::client::height::Height, primitives::H256, ErrorReporter};
use voyager_message::{
    core::{ChainId, ConsensusType, Timestamp},
    module::{ConsensusModuleInfo, ConsensusModuleServer},
    ConsensusModule,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub chain_spec: PresetBaseKind,

    pub provider: DynProvider,
    pub beacon_api_client: BeaconApiClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_spec: PresetBaseKind,

    /// The RPC endpoint for the execution chain.
    pub rpc_url: String,
    /// The RPC endpoint for the beacon chain.
    pub beacon_rpc_url: String,

    #[serde(default)]
    pub max_cache_size: u32,
}

impl Module {
    /// Returns (block_number, timestamp)
    async fn query_latest_execution_meta(&self) -> RpcResult<(u64, u64)> {
        Ok(self
            .beacon_api_client
            .finality_update()
            .await
            .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))?
            .fold(
                |f| match f {},
                |_| todo!(),
                |_| todo!(),
                |f| {
                    (
                        f.finalized_header.execution.block_number,
                        f.finalized_header.execution.timestamp,
                    )
                },
                |f| {
                    (
                        f.finalized_header.execution.block_number,
                        f.finalized_header.execution.timestamp,
                    )
                },
                |f| {
                    (
                        f.finalized_header.execution.block_number,
                        f.finalized_header.execution.timestamp,
                    )
                },
            ))
    }

    // TODO: Deduplicate this from ethereum client-update plugin
    #[instrument(skip_all, fields(block_number))]
    async fn beacon_slot_of_execution_block_number(&self, block_number: u64) -> RpcResult<Slot> {
        trace!("fetching beacon slot of execution block {block_number}");

        let block = self
            .provider
            .get_block((block_number + 1).into(), BlockTransactionsKind::Hashes)
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching block: {}", ErrorReporter(e)),
                    None::<()>,
                )
            })?
            .expect("block should exist");

        let beacon_slot = self
            .beacon_api_client
            .block(
                <H256>::from(
                    block
                        .header
                        .parent_beacon_block_root
                        .expect("parent beacon block root should exist"),
                )
                .into(),
            )
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching block: {}", ErrorReporter(e)),
                    None::<()>,
                )
            })?
            .response
            .fold(
                |b| b.message.slot,
                |b| b.message.slot,
                |b| b.message.slot,
                |b| b.message.slot,
                |b| b.message.slot,
                |b| b.message.slot,
            );

        trace!("beacon slot of exution block {block_number} is {beacon_slot}");

        Ok(beacon_slot)
    }
}

impl ConsensusModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ConsensusModuleInfo) -> Result<Self, BoxDynError> {
        let provider = DynProvider::new(
            ProviderBuilder::new()
                .layer(CacheLayer::new(config.max_cache_size))
                .on_builtin(&config.rpc_url)
                .await?,
        );

        let chain_id = ChainId::new(provider.get_chain_id().await?.to_string());

        info.ensure_chain_id(chain_id.to_string())?;
        info.ensure_consensus_type(ConsensusType::ETHEREUM)?;

        let beacon_api_client = BeaconApiClient::new(config.beacon_rpc_url).await?;

        let spec = beacon_api_client
            .spec()
            .await
            .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))?
            .data;

        if spec.preset_base != config.chain_spec {
            return Err(format!(
                "incorrect chain spec: expected `{}`, but found `{}`",
                config.chain_spec, spec.preset_base
            )
            .into());
        }

        Ok(Self {
            chain_id,
            chain_spec: spec.preset_base,
            provider,
            beacon_api_client,
        })
    }
}

#[async_trait]
impl ConsensusModuleServer for Module {
    /// Query the latest finalized height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id, finalized))]
    async fn query_latest_height(&self, _: &Extensions, finalized: bool) -> RpcResult<Height> {
        if finalized {
            self.query_latest_execution_meta()
                .await
                .map(|meta| Height::new(meta.0))
                .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))
        } else {
            self.provider
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
        _: &Extensions,
        finalized: bool,
    ) -> RpcResult<Timestamp> {
        let latest_timestamp = if finalized {
            self.query_latest_execution_meta().await?.1
        } else {
            self.provider
                .get_block(
                    BlockNumberOrTag::Latest.into(),
                    BlockTransactionsKind::Hashes,
                )
                .await
                .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))?
                .ok_or_else(|| ErrorObject::owned(-1, "latest block not found", None::<()>))?
                .header
                .timestamp
        };
        // Normalize to nanos in order to be compliant with cosmos
        Ok(Timestamp::from_secs(latest_timestamp))
    }
}
