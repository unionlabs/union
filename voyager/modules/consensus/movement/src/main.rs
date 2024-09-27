use std::collections::VecDeque;

use aptos_move_ibc::ibc::ClientExt as _;
use aptos_rest_client::error::RestError;
use call::FetchUpdate;
use jsonrpsee::core::{async_trait, RpcResult};
use queue_msg::{call, data, Op};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, instrument};
use unionlabs::{
    aptos::{
        account::AccountAddress, state_proof::StateProof,
        transaction_proof::TransactionInfoWithProof,
    },
    hash::{hash_v2::Hash, H160},
    ibc::{
        core::client::height::Height,
        lightclients::{
            ethereum::{account_proof::AccountProof, storage_proof::StorageProof},
            movement,
        },
    },
    id::ClientId,
    uint::U256,
    validated::ValidateT,
};
use voyager_message::{
    call::Call,
    core::{ChainId, ClientType},
    data::{Data, DecodedHeaderMeta, OrderedHeaders},
    module::{ConsensusModuleInfo, ConsensusModuleServer, ModuleInfo, QueueInteractionsServer},
    run_module_server, DefaultCmd, ModuleContext, ModuleServer, VoyagerMessage,
};

use crate::{call::ModuleCall, callback::ModuleCallback, data::ModuleData};

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
    run_module_server::<Module, _, _, _>().await
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

impl ModuleContext for Module {
    type Config = Config;
    type Cmd = DefaultCmd;
    type Info = ConsensusModuleInfo;

    async fn new(config: Self::Config) -> Result<Self, chain_utils::BoxDynError> {
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

    fn info(config: Self::Config) -> ModuleInfo<Self::Info> {
        ModuleInfo {
            name: plugin_name(&config.chain_id),
            kind: ConsensusModuleInfo {
                chain_id: config.chain_id,
                client_type: ClientType::new(ClientType::MOVEMENT),
            },
        }
    }
    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

fn plugin_name(chain_id: &ChainId<'_>) -> String {
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
    pub chain_id: ChainId<'static>,

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
impl QueueInteractionsServer<ModuleData, ModuleCall, ModuleCallback> for ModuleServer<Module> {
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn call(
        &self,
        msg: ModuleCall,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        let client = reqwest::Client::new();

        match msg {
            ModuleCall::FetchUpdate(FetchUpdate { from, to }) => {
                let state_proof: StateProofResponse = client
                    .get(format!(
                        "{}/movement/v1/state-proof/{}",
                        self.ctx.movement_rest_url, to
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
                            // dummy value for now, until movement settles on a public L1
                            // 0-1, otherwise it's omitted in the proto encoding(?)
                            l1_height: Height::default().increment(),
                            trusted_height: Height {
                                revision_number: 0,
                                revision_height: from,
                            },
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
                        })
                        .unwrap(),
                    )],
                }))
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn callback(
        &self,
        cb: ModuleCallback,
        _data: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match cb {}
    }
}

#[async_trait]
impl ConsensusModuleServer<ModuleData, ModuleCall, ModuleCallback> for ModuleServer<Module> {
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn self_client_state(&self, height: Height) -> RpcResult<Value> {
        let ledger_version = self
            .ctx
            .ledger_version_of_height(height.revision_height)
            .await;

        let vault_addr = self
            .ctx
            .get_vault_addr(
                (*self.ctx.ibc_handler_address.0.get()).into(),
                Some(ledger_version),
            )
            .await
            .unwrap();

        let table_handle = self
            .ctx
            .aptos_client
            .get_account_resource(
                vault_addr.into(),
                &format!("0x{}::ibc::IBCStore", self.ctx.ibc_handler_address),
            )
            .await
            .unwrap()
            .into_inner()
            .unwrap()
            .data["commitments"]["handle"]
            .clone()
            .as_str()
            .unwrap()
            .to_owned();

        Ok(serde_json::to_value(movement::client_state::ClientState {
            chain_id: self.ctx.chain_id.to_string(),
            l1_client_id: self.ctx.l1_client_id.clone(),
            l1_contract_address: self.ctx.l1_settlement_address,
            l2_contract_address: self.ctx.ibc_handler_address,
            table_handle: AccountAddress(Hash::new(
                U256::from_be_hex(table_handle).unwrap().to_be_bytes(),
            )),
            frozen_height: Height {
                revision_number: 0,
                revision_height: 0,
            },
            latest_block_num: height.revision_height,
        })
        .expect("infallible"))
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
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

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn fetch_update_headers(
        &self,
        update_from: Height,
        update_to: Height,
        _counterparty_chain_id: ChainId<'static>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        Ok(call(Call::plugin(
            self.ctx.plugin_name(),
            FetchUpdate {
                from: update_from.revision_height,
                to: update_to.revision_height,
            },
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_proof_response_serde() {
        let json = r#"
{
  "state_proof": {
    "latest_li_w_sigs": {
      "V0": {
        "ledger_info": {
          "commit_info": {
            "epoch": 1,
            "round": 0,
            "id": "92f8540e3ec839f4813e16c29bd68e3bdb7de9e7aaf6edcd014e6859291a3242",
            "executed_state_id": "63bed3fcf4a51e4c94e93199de82f7b618826c5a21866c87b7f81875b6959954",
            "version": 114,
            "timestamp_usecs": 1725974285935428,
            "next_epoch_state": {
              "epoch": 1,
              "verifier": {
                "validator_infos": [
                  {
                    "address": "d1126ce48bd65fb72190dbd9a6eaa65ba973f1e1664ac0cfba4db1d071fd0c36",
                    "public_key": "0x86fb211f41a07c6399ccc6ab3a8fe568fb0f574ce1b811896c44c6da4f267d543c6cac9fb8f4e9b92a3b809eefb91cbd",
                    "voting_power": 100000000
                  }
                ]
              }
            }
          },
          "consensus_data_hash": "0000000000000000000000000000000000000000000000000000000000000000"
        },
        "signatures": {
          "validator_bitmask": {
            "inner": []
          },
          "sig": null
        }
      }
    },
    "epoch_changes": {
      "ledger_info_with_sigs": [],
      "more": false
    }
  },
  "tx_proof": {
    "ledger_info_to_transaction_info_proof": {
      "siblings": [
        "fe64accd475b2ed5e58d856d0765d3cae338e807d9636a0b19ef9457037380f0",
        "65c0370ceee1ef5adaa85f91ae33dd192aa049d734aafdfc0341bfe5e37ac00b",
        "c6ae59a74870796c4331aaa80dc0554d15fcb7a324868654bd046f52a7cdb389",
        "7ff0d6738e83dd48d8b8d05ac8211c91883d318cedef72b16c8b17e3f4c57d32",
        "ee7e340551ccd5f30b2aecb382be10512ea8e05a70ad90bc1e7cfc9fe809200e",
        "ae1eab32192ab3bacfcdb1529ee19ab4c27ffacae767af086ba777a8eacf1fde",
        "d7a36e16f479b8db00d47b067a9912d8bde2a13b7ad67b9063fc06b6b7eecfdb"
      ],
      "phantom": null
    },
    "transaction_info": {
      "V0": {
        "gas_used": 0,
        "status": "Success",
        "transaction_hash": "d857831148f5e61c04f3e67ed93f04777c8615e555bdb5a8335e8c08042165da",
        "event_root_hash": "414343554d554c41544f525f504c414345484f4c4445525f4841534800000000",
        "state_change_hash": "afb6e14fe47d850fd0a7395bcfb997ffacf4715e0f895cc162c218e4a7564bc6",
        "state_checkpoint_hash": "e142d7896721539b32f737318f4565272c9d9f6eb05661e666c3c16754b23734",
        "state_cemetery_hash": null
      }
    }
  }
}"#;

        let res = serde_json::from_str::<StateProofResponse>(json);

        dbg!(res);
    }
}

#[test]
fn state_proof_resp() {
    let res = r#"{"tx_index":5,"state_proof":{"latest_li_w_sigs":{"V0":{"ledger_info":{"commit_info":{"epoch":1,"round":0,"id":"f4e6ce01b0e1eade7422599157af6b8baad15b665ba32ab223a902fe8609e357","executed_state_id":"646a84844c262c82878c8186dbf4d409097c3a655d05045297d56c003f2583ec","version":5,"timestamp_usecs":1726663664141191,"next_epoch_state":{"epoch":1,"verifier":{"validator_infos":[{"address":"d1126ce48bd65fb72190dbd9a6eaa65ba973f1e1664ac0cfba4db1d071fd0c36","public_key":"0x86fb211f41a07c6399ccc6ab3a8fe568fb0f574ce1b811896c44c6da4f267d543c6cac9fb8f4e9b92a3b809eefb91cbd","voting_power":100000000}]}}},"consensus_data_hash":"0000000000000000000000000000000000000000000000000000000000000000"},"signatures":{"validator_bitmask":{"inner":[]},"sig":null}}},"epoch_changes":{"ledger_info_with_sigs":[],"more":false}},"tx_proof":{"ledger_info_to_transaction_info_proof":{"siblings":["2cdec9e3799fd58a4a8387686a4dee116681a3af462cb6c39a6f3e3b9a933603","414343554d554c41544f525f504c414345484f4c4445525f4841534800000000","d24d4661c12aa515f6e18b48cab1c6e4ef7e961fff345ac02f51012cdf5f0d9c"],"phantom":null},"transaction_info":{"V0":{"gas_used":0,"status":"Success","transaction_hash":"fa047b46005f295eb00e3eb5c7935a6291e50036fb0db8e2679ade38c2df2a59","event_root_hash":"414343554d554c41544f525f504c414345484f4c4445525f4841534800000000","state_change_hash":"afb6e14fe47d850fd0a7395bcfb997ffacf4715e0f895cc162c218e4a7564bc6","state_checkpoint_hash":"7687ce784e3103ec0efe842e04020a87e8cc349eaa80abcd4672ecfce845a81d","state_cemetery_hash":null}}}}"#;

    let resp: StateProofResponse = serde_json::from_str(res).unwrap();
}
