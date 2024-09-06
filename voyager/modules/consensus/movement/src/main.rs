use std::collections::VecDeque;

use aptos_rest_client::error::RestError;
use call::FetchUpdate;
use ethers::providers::{Middleware, Provider, ProviderError, Ws, WsClientError};
use jsonrpsee::core::{async_trait, RpcResult};
use queue_msg::{aggregation::do_callback, call, data, defer, now, promise, seq, Op};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{instrument, warn};
use unionlabs::{
    aptos::{
        account::AccountAddress, state_proof::StateProof,
        transaction_proof::TransactionInfoWithProof,
    },
    hash::H160,
    ibc::{
        core::client::height::Height,
        lightclients::{
            ethereum::{account_proof::AccountProof, storage_proof::StorageProof},
            movement,
        },
    },
    id::ClientId,
    validated::ValidateT,
};
use voyager_message::{
    call::Call,
    callback::Callback,
    data::{Data, DecodedHeaderMeta, OrderedHeaders},
    plugin::{
        ConsensusModuleInfo, ConsensusModuleServer, PluginInfo, PluginKind, PluginModuleServer,
    },
    run_module_server, ChainId, ClientType, VoyagerMessage,
};

use crate::{call::ModuleCall, callback::ModuleCallback, data::ModuleData};

pub mod call;
pub mod callback;
pub mod data;

#[derive(serde::Serialize, serde::Deserialize)]
struct StateProofResponse {
    state_proof: StateProof,
    tx_proof: TransactionInfoWithProof,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_module_server(
        Module::new,
        ConsensusModuleServer::into_rpc,
        voyager_message::default_subcommand_handler,
    )
    .await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId<'static>,

    /// The address of the IBC smart contract.
    pub ibc_handler_address: AccountAddress,

    /// The address of the settlement contract on Eth.
    pub l1_settlement_address: H160,

    pub l1_client_id: ClientId,

    pub aptos_client: aptos_rest_client::Client,

    pub movement_rest_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: AccountAddress,

    /// The address of the settlement contract on Eth.
    pub l1_settlement_address: H160,

    /// Id of the light client that this client depends on
    pub l1_client_id: String,

    /// The RPC endpoint for aptos.
    pub aptos_rest_api: String,

    /// The RPC endpoint for custom movement apis.
    pub movement_rest_url: String,
}

impl Module {
    fn plugin_name(&self) -> String {
        pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

        format!("{PLUGIN_NAME}/{}", self.chain_id)
    }

    pub async fn new(config: Config, _voyager_config: String) -> Result<Self, ModuleInitError> {
        let aptos_client = aptos_rest_client::Client::new(config.aptos_rest_api.parse().unwrap());

        let chain_id = aptos_client.get_index().await?.inner().chain_id;

        Ok(Self {
            chain_id: ChainId::new(chain_id.to_string()),
            ibc_handler_address: config.ibc_handler_address,
            aptos_client,
            l1_settlement_address: config.l1_settlement_address,
            l1_client_id: config.l1_client_id.validate().unwrap(),
            movement_rest_url: config.movement_rest_url,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ModuleInitError {
    #[error("provider error")]
    Rest(#[from] RestError),
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
        let client = reqwest::Client::new();

        match msg {
            ModuleCall::FetchUpdate(FetchUpdate { from, to }) => {
                let state_proof: StateProofResponse = client
                    .get(&format!(
                        "{}/movement/v1/state-proof/{}",
                        self.movement_rest_url, from
                    ))
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();

                Ok(data(OrderedHeaders {
                    headers: vec![(
                        DecodedHeaderMeta {
                            height: Height {
                                revision_number: 0,
                                revision_height: to,
                            },
                        },
                        serde_json::to_value(movement::header::Header {
                            l1_height: Height::default(),
                            trusted_height: Height {
                                revision_number: 0,
                                revision_height: from,
                            },
                            state_proof: state_proof.state_proof,
                            tx_proof: state_proof.tx_proof,
                            state_proof_hash_proof: StorageProof {
                                key: Default::default(),
                                value: Default::default(),
                                proof: Default::default(),
                            },
                            settlement_contract_proof: AccountProof {
                                storage_root: Default::default(),
                                proof: Default::default(),
                            },
                            new_height: to,
                        })
                        .unwrap(),
                    )],
                }))
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn callback(
        &self,
        cb: ModuleCallback,
        _data: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match cb {}
    }
}

#[async_trait]
impl ConsensusModuleServer<ModuleData, ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn consensus_info(&self) -> RpcResult<ConsensusModuleInfo> {
        Ok(ConsensusModuleInfo {
            chain_id: self.chain_id.clone(),
            client_type: ClientType::new(ClientType::MOVEMENT),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_client_state(&self, height: Height) -> RpcResult<Value> {
        Ok(serde_json::to_value(movement::client_state::ClientState {
            chain_id: self.chain_id.to_string(),
            l1_client_id: self.l1_client_id.clone(),
            l1_contract_address: self.l1_settlement_address,
            l2_contract_address: self.ibc_handler_address,
            table_handle: AccountAddress(Default::default()),
            frozen_height: Height {
                revision_number: 0,
                revision_height: 0,
            },
            latest_block_num: height.revision_height,
        })
        .expect("infallible"))
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_consensus_state(&self, _height: Height) -> RpcResult<Value> {
        Ok(
            serde_json::to_value(movement::consensus_state::ConsensusState {
                state_root: Default::default(),
                timestamp: 1000,
                state_proof_hash: Default::default(),
            })
            .expect("infallible"),
        )
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    fn fetch_update_headers(
        &self,
        update_from: Height,
        update_to: Height,
        _counterparty_chain_id: ChainId<'static>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        Ok(call(Call::plugin(
            self.plugin_name(),
            FetchUpdate {
                from: update_from.revision_height,
                to: update_to.revision_height,
            },
        )))
    }
}
