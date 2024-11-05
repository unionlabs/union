use std::ops::Div;

use alloy::{
    eips::BlockNumberOrTag,
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::types::BlockTransactionsKind,
    transports::BoxTransport,
};
use beacon_api::client::BeaconApiClient;
use beacon_api_types::PresetBaseKind;
use ethereum_light_client_types::{AccountProof, ClientState, ConsensusState};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, instrument};
use unionlabs::{
    ethereum::IBC_HANDLER_COMMITMENTS_SLOT, hash::H160, ibc::core::client::height::Height,
    ErrorReporter,
};
use voyager_message::{
    core::{ChainId, ConsensusType},
    module::{ConsensusModuleInfo, ConsensusModuleServer},
    run_consensus_module_server, ConsensusModule,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_consensus_module_server::<Module>().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId<'static>,

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
    pub async fn fetch_account_update(&self, slot: u64) -> AccountProof {
        let execution_height = self
            .beacon_api_client
            .execution_height(beacon_api::client::BlockId::Slot(slot))
            .await
            .unwrap();

        let account_update = self
            .provider
            .get_proof(self.ibc_handler_address.into(), vec![])
            .block_id(
                // NOTE: Proofs are from the execution layer, so we use execution height, not beacon slot.
                execution_height.into(),
            )
            .await
            .unwrap();

        AccountProof {
            storage_root: account_update.storage_hash.into(),
            proof: account_update
                .account_proof
                .into_iter()
                .map(|x| x.to_vec())
                .collect(),
        }
    }
}

impl ConsensusModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ConsensusModuleInfo) -> Result<Self, BoxDynError> {
        let provider = ProviderBuilder::new()
            .on_builtin(&config.eth_rpc_api)
            .await?;

        let chain_id = ChainId::new(provider.get_chain_id().await?.to_string());

        info.ensure_chain_id(chain_id.to_string())?;
        info.ensure_consensus_type(ConsensusType::ETHEREUM)?;

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
impl ConsensusModuleServer for Module {
    /// Query the latest finalized height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_height(&self, _: &Extensions, finalized: bool) -> RpcResult<Height> {
        if finalized {
            self.beacon_api_client
                .finality_update()
                .await
                .map(|response| Height::new(response.data.finalized_header.execution.block_number))
                .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))
        } else {
            Ok(Height::new(self.provider.get_block_number().await.unwrap()))
        }
    }

    /// Query the latest finalized timestamp of this chain.
    // TODO: Use a better timestamp type here
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_timestamp(&self, _: &Extensions, finalized: bool) -> RpcResult<i64> {
        if finalized {
            Ok(self
                .beacon_api_client
                .finality_update()
                .await
                .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))?
                .data
                .attested_header
                .execution
                .timestamp
                .try_into()
                .unwrap())
        } else {
            Ok(self
                .provider
                .get_block(
                    BlockNumberOrTag::Latest.into(),
                    BlockTransactionsKind::Hashes,
                )
                .await
                .unwrap()
                .unwrap()
                .header
                .timestamp
                .try_into()
                .unwrap())
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
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
            latest_slot: height.height(),
            frozen_height: Height::new(0),
            ibc_commitment_slot: IBC_HANDLER_COMMITMENTS_SLOT,
            ibc_contract_address: self.ibc_handler_address,
        })
        .expect("infallible"))
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_consensus_state(&self, _: &Extensions, height: Height) -> RpcResult<Value> {
        let beacon_api_client = &self.beacon_api_client;

        let trusted_header = beacon_api_client
            .header(beacon_api::client::BlockId::Slot(height.height()))
            .await
            .unwrap()
            .data;

        let bootstrap = beacon_api_client
            .bootstrap(trusted_header.root)
            .await
            .unwrap()
            .data;

        let spec = self.beacon_api_client.spec().await.unwrap().data;

        assert!(bootstrap.header.beacon.slot == height.height());

        let light_client_update = {
            let current_period = height.height().div(spec.period());

            debug!(%current_period);

            let light_client_updates = beacon_api_client
                .light_client_updates(current_period, 1)
                .await
                .unwrap();

            let [light_client_update] = &*light_client_updates.0 else {
                panic!()
            };

            light_client_update.data.clone()
        };

        // Normalize to nanos in order to be compliant with cosmos
        let timestamp = bootstrap.header.execution.timestamp * 1_000_000_000;

        Ok(serde_json::to_value(ConsensusState {
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
        })
        .expect("infallible"))
    }
}
