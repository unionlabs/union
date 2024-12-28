use std::ops::Div;

use alloy::{
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::types::BlockTransactionsKind,
    transports::BoxTransport,
};
use beacon_api::client::BeaconApiClient;
use beacon_api_types::PresetBaseKind;
use ethereum_light_client_types::{ClientState, ConsensusState};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, instrument, trace};
use unionlabs::{
    hash::{H160, H256},
    ibc::core::client::height::Height,
    ErrorReporter,
};
use voyager_message::{
    core::{ChainId, ClientType},
    into_value,
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

    pub provider: RootProvider<BoxTransport>,
    pub beacon_api_client: BeaconApiClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub chain_spec: PresetBaseKind,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: String,
    /// The RPC endpoint for the beacon chain.
    pub eth_beacon_rpc_api: String,
}

impl Module {
    // TODO: Deduplicate this from ethereum client-update plugin
    #[instrument(skip_all, fields(block_number))]
    async fn beacon_slot_of_execution_block_number(&self, block_number: u64) -> RpcResult<u64> {
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
            .data
            .message
            .slot;

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
        let provider = ProviderBuilder::new()
            .on_builtin(&config.eth_rpc_api)
            .await?;

        let chain_id = ChainId::new(provider.get_chain_id().await?.to_string());

        info.ensure_chain_id(chain_id.to_string())?;
        info.ensure_client_type(ClientType::ETHEREUM)?;

        let beacon_api_client = BeaconApiClient::new(config.eth_beacon_rpc_api).await?;

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
    async fn self_client_state(&self, _: &Extensions, height: Height) -> RpcResult<Value> {
        let genesis = self.beacon_api_client.genesis().await.unwrap().data;

        let spec = self.beacon_api_client.spec().await.unwrap().data;

        Ok(serde_json::to_value(ClientState {
            chain_id: self
                .chain_id
                .as_str()
                .parse()
                .expect("self.chain_id is a valid u256"),
            chain_spec: spec.preset_base,
            genesis_validators_root: genesis.genesis_validators_root,
            genesis_time: genesis.genesis_time,
            fork_parameters: spec.to_fork_parameters(),
            latest_height: height.height(),
            frozen_height: Height::new(0),
            ibc_contract_address: self.ibc_handler_address,
        })
        .expect("infallible"))
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn self_consensus_state(&self, _: &Extensions, height: Height) -> RpcResult<Value> {
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

        let bootstrap = self
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
            .data;

        let spec = self.beacon_api_client.spec().await.unwrap().data;

        assert_eq!(bootstrap.header.execution.block_number, height.height());

        let light_client_update = {
            let current_period = beacon_slot.div(spec.period());

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

            let [light_client_update] = &*light_client_updates.0 else {
                return Err(ErrorObject::owned(
                    -1,
                    format!(
                        "received invalid light client updates, expected \
                        1 but received {light_client_updates:?}"
                    ),
                    None::<()>,
                ));
            };

            light_client_update.data.clone()
        };

        // Normalize to nanos in order to be compliant with cosmos
        let timestamp = bootstrap.header.execution.timestamp * 1_000_000_000;

        Ok(into_value(ConsensusState {
            slot: bootstrap.header.beacon.slot,
            state_root: bootstrap.header.execution.state_root,
            storage_root: self
                .provider
                .get_proof(self.ibc_handler_address.into(), vec![])
                .block_id(bootstrap.header.execution.block_number.into())
                .await
                .unwrap()
                .storage_hash
                .0
                .into(),
            timestamp,
            current_sync_committee: bootstrap.current_sync_committee.aggregate_pubkey,
            // TODO(aeryz): can this be None?
            next_sync_committee: light_client_update
                .next_sync_committee
                .unwrap()
                .aggregate_pubkey,
        }))
    }
}
