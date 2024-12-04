use std::collections::VecDeque;

use aptos_rest_client::error::RestError;
use call::FetchUpdate;
use ethereum_light_client_types::{account_proof::AccountProof, storage_proof::StorageProof};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};
use unionlabs::{
    aptos::{
        account::AccountAddress, state_proof::StateProof,
        transaction_proof::TransactionInfoWithProof,
    },
    hash::H160,
    ibc::core::client::height::Height,
};
use voyager_message::{
    call::Call,
    core::ChainId,
    data::{Data, DecodedHeaderMeta, OrderedHeaders},
    hook::UpdateHook,
    module::{PluginInfo, PluginServer, UnexpectedChainIdError},
    DefaultCmd, Plugin, PluginMessage, VoyagerMessage,
};
use voyager_vm::{data, pass::PassResult, Op, Visit};

use crate::{call::ModuleCall, callback::ModuleCallback};

pub mod call;
pub mod callback;
pub mod data;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct StateProofResponse {
    tx_index: u64,
    state_proof: StateProof,
    tx_proof: TransactionInfoWithProof,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    /// The address of the IBC smart contract.
    pub ibc_handler_address: AccountAddress,

    /// The address of the settlement contract on Eth.
    pub l1_settlement_address: H160,

    pub l1_client_id: u32,

    pub aptos_client: aptos_rest_client::Client,

    pub movement_rest_url: String,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, chain_utils::BoxDynError> {
        let aptos_client = aptos_rest_client::Client::new(config.aptos_rest_api.parse().unwrap());

        let chain_id = aptos_client.get_index().await?.inner().chain_id.to_string();

        if chain_id != config.chain_id.as_str() {
            return Err(UnexpectedChainIdError {
                expected: config.chain_id,
                found: chain_id,
            }
            .into());
        }

        Ok(Self {
            chain_id: config.chain_id,
            ibc_handler_address: config.ibc_handler_address,
            aptos_client,
            l1_settlement_address: config.l1_settlement_address,
            l1_client_id: config.l1_client_id,
            movement_rest_url: config.movement_rest_url,
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: UpdateHook::filter(&config.chain_id),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

fn plugin_name(chain_id: &ChainId) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}", chain_id)
}

impl aptos_move_ibc::ibc::ClientExt for Module {
    fn client(&self) -> &aptos_rest_client::Client {
        &self.aptos_client
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The identifier of the chain
    pub chain_id: ChainId,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: AccountAddress,

    /// The address of the settlement contract on Eth.
    pub l1_settlement_address: H160,

    /// Id of the light client that this client depends on
    pub l1_client_id: u32,

    /// The RPC endpoint for aptos.
    pub aptos_rest_api: String,

    /// The RPC endpoint for custom movement apis.
    pub movement_rest_url: String,
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }

    pub async fn ledger_version_of_height(&self, height: u64) -> u64 {
        let ledger_version = self
            .aptos_client
            .get_block_by_height(height, false)
            .await
            // .map_err(rest_error_to_rpc_error)?
            .unwrap()
            .into_inner()
            .last_version
            .0;

        debug!("height {height} is ledger version {ledger_version}");

        ledger_version
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ModuleInitError {
    #[error("provider error")]
    Rest(#[from] RestError),
}

#[async_trait]
impl PluginServer<ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn run_pass(
        &self,
        _: &Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        Ok(PassResult {
            optimize_further: vec![],
            ready: msgs
                .into_iter()
                .map(|mut op| {
                    UpdateHook::new(&self.chain_id, |fetch| {
                        Call::Plugin(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(FetchUpdate {
                                from: fetch.update_from.revision(),
                                to: fetch.update_to.height(),
                            }),
                        ))
                    })
                    .visit_op(&mut op);

                    op
                })
                .enumerate()
                .map(|(i, op)| (vec![i], op))
                .collect(),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(&self, _: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::FetchUpdate(FetchUpdate { from, to }) => {
                // NOTE(aeryz): This only works with Union's custom Movement node. When the following PR is merged,
                // we will uncomment this: https://github.com/movementlabsxyz/movement/pull/645
                // let header = get_lc_header(&self.movement_rest_url, from, to).await;
                Ok(data(OrderedHeaders {
                    headers: vec![(
                        DecodedHeaderMeta {
                            height: Height::new(to),
                        },
                        serde_json::to_value(movement_light_client_types::Header {
                            // dummy value for now, until movement settles on a public L1
                            l1_height: 0,
                            trusted_height: Height::new(from),
                            state_proof: StateProof::default(),
                            tx_index: 0,
                            tx_proof: TransactionInfoWithProof::default(),
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
        _: &Extensions,
        cb: ModuleCallback,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }
}

pub async fn get_lc_header(
    movement_rest_url: &str,
    from: u64,
    to: u64,
) -> movement_light_client_types::Header {
    let client = reqwest::Client::new();

    let state_proof: StateProofResponse = client
        .get(format!("{movement_rest_url}/movement/v1/state-proof/{to}",))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    movement_light_client_types::Header {
        // dummy value for now, until movement settles on a public L1
        l1_height: 0,
        trusted_height: Height::new(from),
        state_proof: state_proof.state_proof,
        tx_index: state_proof.tx_index,
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
    }
}
#[test]
fn state_proof_resp() {
    let res = r#"{"tx_index":5,"state_proof":{"latest_li_w_sigs":{"V0":{"ledger_info":{"commit_info":{"epoch":1,"round":0,"id":"f4e6ce01b0e1eade7422599157af6b8baad15b665ba32ab223a902fe8609e357","executed_state_id":"646a84844c262c82878c8186dbf4d409097c3a655d05045297d56c003f2583ec","version":5,"timestamp_usecs":1726663664141191,"next_epoch_state":{"epoch":1,"verifier":{"validator_infos":[{"address":"d1126ce48bd65fb72190dbd9a6eaa65ba973f1e1664ac0cfba4db1d071fd0c36","public_key":"0x86fb211f41a07c6399ccc6ab3a8fe568fb0f574ce1b811896c44c6da4f267d543c6cac9fb8f4e9b92a3b809eefb91cbd","voting_power":100000000}]}}},"consensus_data_hash":"0000000000000000000000000000000000000000000000000000000000000000"},"signatures":{"validator_bitmask":{"inner":[]},"sig":null}}},"epoch_changes":{"ledger_info_with_sigs":[],"more":false}},"tx_proof":{"ledger_info_to_transaction_info_proof":{"siblings":["2cdec9e3799fd58a4a8387686a4dee116681a3af462cb6c39a6f3e3b9a933603","414343554d554c41544f525f504c414345484f4c4445525f4841534800000000","d24d4661c12aa515f6e18b48cab1c6e4ef7e961fff345ac02f51012cdf5f0d9c"],"phantom":null},"transaction_info":{"V0":{"gas_used":0,"status":"Success","transaction_hash":"fa047b46005f295eb00e3eb5c7935a6291e50036fb0db8e2679ade38c2df2a59","event_root_hash":"414343554d554c41544f525f504c414345484f4c4445525f4841534800000000","state_change_hash":"afb6e14fe47d850fd0a7395bcfb997ffacf4715e0f895cc162c218e4a7564bc6","state_checkpoint_hash":"7687ce784e3103ec0efe842e04020a87e8cc349eaa80abcd4672ecfce845a81d","state_cemetery_hash":null}}}}"#;

    let _resp: StateProofResponse = serde_json::from_str(res).unwrap();
}
