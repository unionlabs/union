use std::ops::Div;

use alloy::providers::{layers::CacheLayer, DynProvider, Provider, ProviderBuilder};
use beacon_api::client::BeaconApiClient;
use beacon_api_types::{altair::SyncCommittee, chain_spec::PresetBaseKind, custom_types::Slot};
use ethereum_light_client_types::{
    client_state::InitialSyncCommittee, ClientState, ClientStateV1, ConsensusState,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, instrument, trace};
use unionlabs::{
    ibc::core::client::height::Height,
    primitives::{H160, H256},
    ErrorReporter,
};
use voyager_message::{
    primitives::{ChainId, ClientType},
    ensure_null, into_value,
    module::{ClientBootstrapModuleInfo, ClientBootstrapModuleServer},
    ClientBootstrapModule,
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

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    pub provider: DynProvider,
    pub beacon_api_client: BeaconApiClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_spec: PresetBaseKind,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The RPC endpoint for the execution chain.
    pub rpc_url: String,
    /// The RPC endpoint for the beacon chain.
    pub beacon_rpc_url: String,

    #[serde(default)]
    pub max_cache_size: u32,
}

impl Module {
    // TODO: Deduplicate this from ethereum client-update plugin
    #[instrument(skip_all, fields(block_number))]
    async fn beacon_slot_of_execution_block_number(&self, block_number: u64) -> RpcResult<Slot> {
        trace!("fetching beacon slot of execution block {block_number}");

        let block = self
            .provider
            .get_block((block_number + 1).into())
            .hashes()
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching execution block: {}", ErrorReporter(e)),
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
                    format!("error fetching beacon block: {}", ErrorReporter(e)),
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

impl ClientBootstrapModule for Module {
    type Config = Config;

    async fn new(
        config: Self::Config,
        info: ClientBootstrapModuleInfo,
    ) -> Result<Self, BoxDynError> {
        let provider = DynProvider::new(
            ProviderBuilder::new()
                .layer(CacheLayer::new(config.max_cache_size))
                .connect(&config.rpc_url)
                .await?,
        );

        let chain_id = ChainId::new(provider.get_chain_id().await?.to_string());

        info.ensure_chain_id(chain_id.to_string())?;
        info.ensure_client_type(ClientType::ETHEREUM)?;

        let beacon_api_client = BeaconApiClient::new(config.beacon_rpc_url).await?;

        let spec = beacon_api_client.spec().await.unwrap().data;

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
            ibc_handler_address: config.ibc_handler_address,
            provider,
            beacon_api_client,
        })
    }
}

#[async_trait]
impl ClientBootstrapModuleServer for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn self_client_state(
        &self,
        _: &Extensions,
        height: Height,
        config: Value,
    ) -> RpcResult<Value> {
        ensure_null(config)?;

        let genesis = self.beacon_api_client.genesis().await.unwrap().data;

        let spec = self.beacon_api_client.spec().await.unwrap().data;

        let beacon_slot = self
            .beacon_slot_of_execution_block_number(height.height())
            .await?;

        let light_client_update = {
            let current_period = beacon_slot.get().div(spec.period());

            debug!(%current_period);

            let light_client_updates = self
                .beacon_api_client
                .light_client_updates(current_period, 1)
                .await
                .map_err(|e| {
                    ErrorObject::owned(
                        -1,
                        format!("error fetching light client update: {}", ErrorReporter(e)),
                        None::<()>,
                    )
                })?;

            let [light_client_update] = &*light_client_updates else {
                return Err(ErrorObject::owned(
                    -1,
                    format!(
                        "received invalid light client updates, expected \
                        1 but received {light_client_updates:?}"
                    ),
                    None::<()>,
                ));
            };

            light_client_update
                .clone()
                .fold::<ethereum_sync_protocol_types::LightClientUpdate>(
                    |e| match e {},
                    |_| todo!("altair not supported"),
                    |_| todo!("bellatrix not supported"),
                    |_| todo!("capella not supported"),
                    |u| u.into(),
                    |u| u.into(),
                )
        };

        let trusted_header = self
            .beacon_api_client
            .header(beacon_api::client::BlockId::Slot(beacon_slot))
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching beacon header: {}", ErrorReporter(e)),
                    None::<()>,
                )
            })?
            .data;

        let current_sync_committee = self
            .beacon_api_client
            .bootstrap(trusted_header.root)
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching beacon bootstrap: {}", ErrorReporter(e)),
                    None::<()>,
                )
            })?
            .fold::<SyncCommittee>(
                |l| match l {},
                |_| todo!("altair not supported"),
                |_| todo!("bellatrix not supported"),
                |l| l.current_sync_committee,
                |l| l.current_sync_committee,
                |l| l.current_sync_committee,
            );

        Ok(serde_json::to_value(ClientState::V1(ClientStateV1 {
            chain_id: self
                .chain_id
                .as_str()
                .parse()
                .expect("self.chain_id is a valid u256"),
            chain_spec: spec.preset_base,
            genesis_validators_root: genesis.genesis_validators_root,
            genesis_time: genesis.genesis_time,
            latest_height: height.height(),
            frozen_height: Height::new(0),
            ibc_contract_address: self.ibc_handler_address,
            initial_sync_committee: Some(InitialSyncCommittee {
                current_sync_committee,
                next_sync_committee: light_client_update.next_sync_committee.unwrap(),
            }),
        }))
        .expect("infallible"))
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn self_consensus_state(
        &self,
        _: &Extensions,
        height: Height,
        config: Value,
    ) -> RpcResult<Value> {
        ensure_null(config)?;

        let beacon_slot = self
            .beacon_slot_of_execution_block_number(height.height())
            .await?;

        let trusted_header = self
            .beacon_api_client
            .header(beacon_api::client::BlockId::Slot(beacon_slot))
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching beacon header: {}", ErrorReporter(e)),
                    None::<()>,
                )
            })?
            .data;

        let bootstrap_header = self
            .beacon_api_client
            .bootstrap(trusted_header.root)
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching beacon bootstrap: {}", ErrorReporter(e)),
                    None::<()>,
                )
            })?
            .fold::<ethereum_sync_protocol_types::LightClientHeader>(
                |l| match l {},
                |_| todo!("altair not supported"),
                |_| todo!("bellatrix not supported"),
                |_| todo!("capella not supported"),
                |l| l.header.into(),
                |l| l.header.into(),
            );

        assert_eq!(bootstrap_header.execution.block_number, height.height());

        // Normalize to nanos in order to be compliant with cosmos
        let timestamp = bootstrap_header.execution.timestamp * 1_000_000_000;

        Ok(into_value(ConsensusState {
            slot: bootstrap_header.beacon.slot,
            state_root: bootstrap_header.execution.state_root,
            storage_root: self
                .provider
                .get_proof(self.ibc_handler_address.into(), vec![])
                .block_id(bootstrap_header.execution.block_number.into())
                .await
                .unwrap()
                .storage_hash
                .0
                .into(),
            timestamp,
        }))
    }
}
