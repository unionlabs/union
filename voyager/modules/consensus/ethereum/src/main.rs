use std::{collections::VecDeque, ops::Div};

use beacon_api::client::BeaconApiClient;
use bitvec::{order::Msb0, vec::BitVec};
use ethers::providers::{Middleware, Provider, ProviderError, Ws, WsClientError};
use jsonrpsee::core::{async_trait, RpcResult};
use queue_msg::{aggregation::do_callback, call, data, defer_relative, promise, seq, Op};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, info, instrument, warn};
use unionlabs::{
    ethereum::IBC_HANDLER_COMMITMENTS_SLOT,
    hash::H160,
    ibc::{
        core::client::height::Height,
        lightclients::ethereum::{
            self, account_proof::AccountProof, account_update::AccountUpdate,
        },
    },
};
use voyager_message::{
    call::Call,
    callback::Callback,
    data::Data,
    plugin::{
        ConsensusModuleInfo, ConsensusModuleServer, PluginInfo, PluginKind, PluginModuleServer,
    },
    run_module_server, ChainId, ClientType, VoyagerMessage,
};

use crate::{
    call::{
        FetchAccountUpdate, FetchBeaconGenesis, FetchBeaconSpec, FetchBootstrap,
        FetchFinalityUpdate, FetchLightClientUpdate, FetchLightClientUpdates, ModuleCall,
    },
    callback::{MakeCreateUpdates, ModuleCallback},
    data::{
        AccountUpdateData, BeaconGenesis, BeaconSpec, BootstrapData, FinalityUpdate,
        LightClientUpdates, ModuleData,
    },
};

pub mod call;
pub mod callback;
pub mod data;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    run_module_server(Module::new, ConsensusModuleServer::into_rpc).await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId<'static>,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    pub provider: Provider<Ws>,
    pub beacon_api_client: BeaconApiClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: String,
    /// The RPC endpoint for the beacon chain.
    pub eth_beacon_rpc_api: String,
}

impl Module {
    fn plugin_name(&self) -> String {
        pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

        format!("{PLUGIN_NAME}/{}", self.chain_id)
    }

    pub async fn new(config: Config) -> Result<Self, ModuleInitError> {
        let provider = Provider::new(Ws::connect(config.eth_rpc_api).await?);

        let chain_id = provider.get_chainid().await?;

        Ok(Self {
            chain_id: ChainId::new(chain_id.to_string()),
            ibc_handler_address: config.ibc_handler_address,
            provider,
            beacon_api_client: BeaconApiClient::new(config.eth_beacon_rpc_api).await?,
        })
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
impl PluginModuleServer<ModuleData, ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn info(&self) -> RpcResult<PluginInfo> {
        Ok(PluginInfo {
            name: self.plugin_name(),
            kind: Some(PluginKind::Consensus),
            interest_filter: None,
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(
        &self,
        msg: ModuleCall,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        let beacon_api_client = &self.beacon_api_client;

        match msg {
            ModuleCall::FetchFinalityUpdate(FetchFinalityUpdate {}) => {
                let finality_update = beacon_api_client.finality_update().await.unwrap().data;

                let has_supermajority = {
                    let scb = BitVec::<u8, Msb0>::try_from(
                        finality_update.sync_aggregate.sync_committee_bits.clone(),
                    )
                    .unwrap();

                    let sync_committee_size = self
                        .beacon_api_client
                        .spec()
                        .await
                        .unwrap()
                        .data
                        .sync_committee_size;

                    dbg!(format!("{scb:x}"));

                    assert_eq!(scb.len() as u64, sync_committee_size);

                    scb.count_ones() * 3 < scb.len() * 2
                };

                if has_supermajority {
                    info!(
                        signature_slot = finality_update.signature_slot,
                        "signature supermajority not hit"
                    );

                    Ok(seq([
                        defer_relative(1),
                        call(Call::plugin(self.plugin_name(), FetchFinalityUpdate {})),
                    ]))
                } else {
                    Ok(data(Data::plugin(
                        self.plugin_name(),
                        FinalityUpdate { finality_update },
                    )))
                }
            }
            ModuleCall::FetchLightClientUpdates(FetchLightClientUpdates {
                trusted_period,
                target_period,
            }) => Ok(data(Data::plugin(
                self.plugin_name(),
                LightClientUpdates {
                    light_client_updates: beacon_api_client
                        .light_client_updates(trusted_period + 1, target_period - trusted_period)
                        .await
                        .unwrap()
                        .0
                        .into_iter()
                        .map(|x| x.data)
                        .collect(),
                },
            ))),
            ModuleCall::FetchLightClientUpdate(FetchLightClientUpdate { period }) => {
                Ok(data(Data::plugin(
                    self.plugin_name(),
                    crate::data::LightClientUpdate {
                        update: beacon_api_client
                            .light_client_updates(period, 1)
                            .await
                            .unwrap()
                            .0
                            .into_iter()
                            .map(|x| x.data)
                            .collect::<Vec<_>>()
                            .pop()
                            .unwrap(),
                    },
                )))
            }
            ModuleCall::FetchBootstrap(FetchBootstrap { slot }) => Ok(data(Data::plugin(
                self.plugin_name(),
                BootstrapData {
                    slot,
                    bootstrap: beacon_api_client
                        .bootstrap_for_slot(slot)
                        .await
                        .unwrap()
                        .data,
                },
            ))),
            ModuleCall::FetchAccountUpdate(FetchAccountUpdate { slot }) => {
                let execution_height = beacon_api_client
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

                Ok(data(Data::plugin(
                    self.plugin_name(),
                    AccountUpdateData {
                        slot,
                        update: AccountUpdate {
                            account_proof: AccountProof {
                                storage_root: account_update.storage_hash.into(),
                                proof: account_update
                                    .account_proof
                                    .into_iter()
                                    .map(|x| x.to_vec())
                                    .collect(),
                            },
                        },
                    },
                )))
            }
            ModuleCall::FetchBeaconGenesis(FetchBeaconGenesis {}) => Ok(data(Data::plugin(
                self.plugin_name(),
                BeaconGenesis {
                    genesis: beacon_api_client.genesis().await.unwrap().data,
                },
            ))),
            ModuleCall::FetchBeaconSpec(FetchBeaconSpec {}) => Ok(data(Data::plugin(
                self.plugin_name(),
                BeaconSpec {
                    spec: beacon_api_client.spec().await.unwrap().data,
                },
            ))),
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    fn callback(
        &self,
        cb: ModuleCallback,
        data: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        Ok(match cb {
            ModuleCallback::CreateUpdate(aggregate) => do_callback(aggregate, data),
            ModuleCallback::MakeCreateUpdates(aggregate) => do_callback(aggregate, data),
            ModuleCallback::MakeCreateUpdatesFromLightClientUpdates(aggregate) => {
                do_callback(aggregate, data)
            }
            ModuleCallback::AggregateHeaders(aggregate) => {
                queue_msg::data(aggregate.aggregate(data))
            }
        })
    }
}

#[async_trait]
impl ConsensusModuleServer<ModuleData, ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn consensus_info(&self) -> RpcResult<ConsensusModuleInfo> {
        Ok(ConsensusModuleInfo {
            chain_id: self.chain_id.clone(),
            client_type: ClientType::new(ClientType::ETHEREUM_MINIMAL),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_client_state(&self, height: Height) -> RpcResult<Value> {
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
    async fn self_consensus_state(&self, height: Height) -> RpcResult<Value> {
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
                        ethers::types::H160::from(self.ibc_handler_address.0),
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

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    fn fetch_update_headers(
        &self,
        update_from: Height,
        update_to: Height,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        Ok(promise(
            [
                call(Call::plugin(self.plugin_name(), FetchFinalityUpdate {})),
                call(Call::plugin(self.plugin_name(), FetchBeaconSpec {})),
            ],
            [],
            Callback::plugin(
                self.plugin_name(),
                MakeCreateUpdates {
                    update_from,
                    update_to,
                },
            ),
        ))
    }
}
