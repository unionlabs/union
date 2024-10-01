use std::ops::Div;

use beacon_api::client::BeaconApiClient;
use ethers::providers::{Middleware, Provider, ProviderError, Ws, WsClientError};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, instrument};
use unionlabs::{
    ethereum::{config::PresetBaseKind, IBC_HANDLER_COMMITMENTS_SLOT},
    hash::H160,
    ibc::{
        core::client::height::Height,
        lightclients::ethereum::{
            self, account_proof::AccountProof, account_update::AccountUpdate,
        },
    },
};
use voyager_message::{
    core::{ChainId, ClientType},
    module::{ConsensusModuleInfo, ConsensusModuleServer, ModuleInfo},
    run_module_server, DefaultCmd, ModuleContext,
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_module_server::<Module>().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId<'static>,

    pub chain_spec: PresetBaseKind,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    pub provider: Provider<Ws>,
    pub beacon_api_client: BeaconApiClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub chain_id: ChainId<'static>,

    pub chain_spec: PresetBaseKind,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: String,
    /// The RPC endpoint for the beacon chain.
    pub eth_beacon_rpc_api: String,
}

impl Module {
    pub async fn fetch_account_update(&self, slot: u64) -> AccountUpdate {
        let execution_height = self
            .beacon_api_client
            .execution_height(beacon_api::client::BlockId::Slot(slot))
            .await
            .unwrap();

        let account_update = self
            .provider
            .get_proof(
                ethers::types::H160::from(self.ibc_handler_address),
                vec![],
                // NOTE: Proofs are from the execution layer, so we use execution height, not beacon slot.
                Some(execution_height.into()),
            )
            .await
            .unwrap();

        AccountUpdate {
            account_proof: AccountProof {
                storage_root: account_update.storage_hash.into(),
                proof: account_update
                    .account_proof
                    .into_iter()
                    .map(|x| x.to_vec())
                    .collect(),
            },
        }
    }
}

impl ModuleContext for Module {
    type Config = Config;
    type Cmd = DefaultCmd;
    type Info = ConsensusModuleInfo;

    async fn new(config: Self::Config) -> Result<Self, chain_utils::BoxDynError> {
        let provider = Provider::new(Ws::connect(config.eth_rpc_api).await?);

        let chain_id = ChainId::new(provider.get_chainid().await?.to_string());

        if chain_id != config.chain_id {
            return Err(format!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.chain_id, chain_id
            )
            .into());
        }

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

    fn info(config: Self::Config) -> ModuleInfo<Self::Info> {
        ModuleInfo {
            kind: ConsensusModuleInfo {
                chain_id: config.chain_id,
                client_type: ClientType::new(match config.chain_spec {
                    PresetBaseKind::Minimal => ClientType::ETHEREUM_MINIMAL,
                    PresetBaseKind::Mainnet => ClientType::ETHEREUM_MAINNET,
                }),
            },
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ModuleInitError {
    #[error("unable to connect to websocket")]
    Ws(#[from] WsClientError),
    #[error("provider error")]
    Provider(#[from] ProviderError),
    #[error("beacon error")]
    Beacon(#[from] beacon_api::client::NewError),
}

#[async_trait]
impl ConsensusModuleServer for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_client_state(&self, _: &Extensions, height: Height) -> RpcResult<Value> {
        let genesis = self.beacon_api_client.genesis().await.unwrap().data;

        let spec = self.beacon_api_client.spec().await.unwrap().data;

        Ok(serde_json::to_value(ethereum::client_state::ClientState {
            chain_id: self
                .chain_id
                .as_str()
                .parse()
                .expect("self.chain_id is a valid u256"),
            genesis_validators_root: genesis.genesis_validators_root,
            genesis_time: genesis.genesis_time,
            fork_parameters: spec.to_fork_parameters(),
            seconds_per_slot: spec.seconds_per_slot,
            slots_per_epoch: spec.slots_per_epoch,
            epochs_per_sync_committee_period: spec.epochs_per_sync_committee_period,
            latest_slot: height.revision_height,
            min_sync_committee_participants: 0,
            frozen_height: Height {
                revision_number: 0,
                revision_height: 0,
            },
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
            .header(beacon_api::client::BlockId::Slot(height.revision_height))
            .await
            .unwrap()
            .data;

        let bootstrap = beacon_api_client
            .bootstrap(trusted_header.root)
            .await
            .unwrap()
            .data;

        let spec = self.beacon_api_client.spec().await.unwrap().data;

        assert!(bootstrap.header.beacon.slot == height.revision_height);

        let light_client_update = {
            let current_period = height.revision_height.div(spec.period());

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

        Ok(
            serde_json::to_value(ethereum::consensus_state::ConsensusState {
                slot: bootstrap.header.beacon.slot,
                state_root: bootstrap.header.execution.state_root,
                storage_root: self
                    .provider
                    .get_proof(
                        ethers::types::H160::from(*self.ibc_handler_address.get()),
                        vec![],
                        Some(bootstrap.header.execution.block_number.into()),
                    )
                    .await
                    .unwrap()
                    .storage_hash
                    .0
                    .into(),
                timestamp,
                current_sync_committee: bootstrap.current_sync_committee.aggregate_pubkey,
                next_sync_committee: light_client_update
                    .next_sync_committee
                    .map(|nsc| nsc.aggregate_pubkey),
            })
            .expect("infallible"),
        )
    }
}
