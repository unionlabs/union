// #![warn(clippy::unwrap_used)]

use std::collections::VecDeque;

use alloy::{
    network::AnyNetwork,
    providers::{DynProvider, Provider, ProviderBuilder},
};
use arbitrum_light_client_types::{Header, L2Header};
use arbitrum_types::{L1_NEXT_NODE_NUM_SLOT, L1_NODES_CONFIRM_DATA_OFFSET, L1_NODES_SLOT};
use ethereum_light_client_types::{AccountProof, StorageProof};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};
use unionlabs::{
    ibc::core::client::height::Height,
    primitives::{H160, H64, U256},
    ErrorReporter,
};
use voyager_message::{
    call::Call,
    core::{ChainId, ClientType},
    data::{Data, DecodedHeaderMeta, OrderedHeaders},
    hook::UpdateHook,
    into_value,
    module::{PluginInfo, PluginServer},
    DefaultCmd, Plugin, PluginMessage, RawClientId, VoyagerMessage,
};
use voyager_vm::{pass::PassResult, BoxDynError, Op, Visit};

use crate::{
    call::{FetchUpdate, ModuleCall},
    callback::ModuleCallback,
};

pub mod call;
pub mod callback;
pub mod data;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub l1_chain_id: ChainId,

    pub l1_contract_address: H160,

    pub l1_provider: DynProvider,
    pub l2_provider: DynProvider<AnyNetwork>,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub l2_chain_id: ChainId,

    /// The chain id of the chain this arbitrum chain chain settles on.
    pub l1_chain_id: ChainId,

    /// The Rollup contract on the L1.
    pub l1_contract_address: H160,

    /// The RPC endpoint for the settlement (L1) execution chain.
    pub l1_rpc_url: String,

    /// The RPC endpoint for the main (L2) execution chain.
    pub l2_rpc_url: String,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    #[serde(default)]
    pub max_cache_size: u32,
}

fn plugin_name(chain_id: &ChainId) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}", chain_id)
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }

    #[instrument(
        skip_all,
        fields(
            %block_number,
            ibc_handler_address = %self.ibc_handler_address
        )
    )]
    pub async fn fetch_account_update(&self, block_number: u64) -> RpcResult<AccountProof> {
        let account_update = self
            .l2_provider
            .get_proof(self.ibc_handler_address.into(), vec![])
            .block_id(
                // NOTE: Proofs are from the execution layer, so we use execution height, not beacon slot.
                block_number.into(),
            )
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error fetching account update"),
                    None::<()>,
                )
            })?;

        // tokio::time::sleep(std::time::Duration::from_millis(500));

        debug!(storage_hash = %account_update.storage_hash, "fetched account update");

        Ok(AccountProof {
            storage_root: account_update.storage_hash.into(),
            proof: account_update
                .account_proof
                .into_iter()
                .map(|x| x.into())
                .collect(),
        })
    }
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        let l1_provider =
            DynProvider::new(ProviderBuilder::new().connect(&config.l1_rpc_url).await?);

        let l2_provider = DynProvider::new(
            ProviderBuilder::new()
                .network::<AnyNetwork>()
                .connect(&config.l2_rpc_url)
                .await?,
        );

        let l1_chain_id = ChainId::new(l1_provider.get_chain_id().await?.to_string());
        let l2_chain_id = ChainId::new(l2_provider.get_chain_id().await?.to_string());

        assert_eq!(l1_chain_id, config.l1_chain_id);
        assert_eq!(l2_chain_id, config.l2_chain_id);

        Ok(Self {
            chain_id: l2_chain_id,
            l1_chain_id,
            l1_contract_address: config.l1_contract_address,
            l1_provider,
            l2_provider,
            ibc_handler_address: config.ibc_handler_address,
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.l2_chain_id),
            interest_filter: UpdateHook::filter(
                &config.l2_chain_id,
                &ClientType::new(ClientType::ARBITRUM),
            ),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
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
                    UpdateHook::new(
                        &self.chain_id,
                        &ClientType::new(ClientType::ETHEREUM),
                        |fetch| {
                            Call::Plugin(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(FetchUpdate {
                                    from_height: fetch.update_from,
                                    to_height: fetch.update_to,
                                    counterparty_chain_id: fetch.counterparty_chain_id.clone(),
                                    client_id: fetch.client_id.clone(),
                                }),
                            ))
                        },
                    )
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
            ModuleCall::FetchUpdate(FetchUpdate {
                from_height,
                to_height,
                counterparty_chain_id,
                client_id,
            }) => self
                .fetch_update(from_height, to_height, counterparty_chain_id, client_id)
                .await
                .map_err(|e| {
                    ErrorObject::owned(
                        -1,
                        format!("error fetching update: {}", ErrorReporter(&*e)),
                        None::<()>,
                    )
                }),
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

impl Module {
    async fn fetch_latest_confirmed_proofs(&self, height: u64) -> LatestConfirmedProofs {
        let latest_confirmed = arbitrum_client::next_node_num_at_l1_height(
            &self.l1_provider,
            self.l1_contract_address,
            height,
        )
        .await
        .unwrap();

        let [latest_confirmed_slot_proof, nodes_slot_proof]: [_; 2] = self
            .l1_provider
            .get_proof(
                self.l1_contract_address.into(),
                vec![
                    L1_NEXT_NODE_NUM_SLOT.to_be_bytes().into(),
                    arbitrum_verifier::nodes_confirm_data_mapping_key(
                        L1_NODES_SLOT.into(),
                        latest_confirmed,
                        L1_NODES_CONFIRM_DATA_OFFSET.into(),
                    )
                    .to_be_bytes()
                    .into(),
                ],
            )
            .block_id(height.into())
            .await
            .unwrap()
            .storage_proof
            .try_into()
            .unwrap();

        LatestConfirmedProofs {
            latest_confirmed,
            // TODO: Extract this logic into a fn, we do it all over the place
            latest_confirmed_slot_proof: StorageProof {
                key: U256::from_be_bytes(latest_confirmed_slot_proof.key.as_b256().0),
                value: latest_confirmed_slot_proof.value.into(),
                proof: latest_confirmed_slot_proof
                    .proof
                    .into_iter()
                    .map(|bytes| bytes.into())
                    .collect(),
            },
            nodes_slot_proof: StorageProof {
                key: U256::from_be_bytes(nodes_slot_proof.key.as_b256().0),
                value: nodes_slot_proof.value.into(),
                proof: nodes_slot_proof
                    .proof
                    .into_iter()
                    .map(|bytes| bytes.into())
                    .collect(),
            },
        }
    }

    // TODO: Use?
    // async fn fetch_l1_contract_root_proof(&self, height: u64) -> AccountProof {
    //     let proof = self
    //         .l1_provider
    //         .get_proof(self.l1_contract_address.into(), vec![])
    //         .block_id(height.into())
    //         .await
    //         .unwrap();

    //     AccountProof {
    //         storage_root: proof.storage_hash.into(),
    //         proof: proof.account_proof.into_iter().map(|x| x.into()).collect(),
    //     }
    // }

    async fn fetch_ibc_contract_root_proof(&self, height: u64) -> AccountProof {
        let proof = self
            .l2_provider
            .get_proof(self.ibc_handler_address.into(), vec![])
            .block_id(height.into())
            .await
            .unwrap();

        AccountProof {
            storage_root: proof.storage_hash.into(),
            proof: proof.account_proof.into_iter().map(|x| x.into()).collect(),
        }
    }

    #[instrument(
        skip_all,
        fields(
            chain_id = %self.chain_id,
            %counterparty_chain_id,
            %update_from_block_number,
            %update_to_block_number
        )
    )]
    async fn fetch_update(
        &self,
        update_from_block_number: Height,
        update_to_block_number: Height,
        counterparty_chain_id: ChainId,
        _client_id: RawClientId,
    ) -> Result<Op<VoyagerMessage>, BoxDynError> {
        let l1_account_proof = self
            .fetch_account_update(update_from_block_number.height())
            .await
            .unwrap();

        let l2_ibc_account_proof = self
            .fetch_ibc_contract_root_proof(update_from_block_number.height())
            .await;

        let latest_confirmed_proofs = self
            .fetch_latest_confirmed_proofs(update_from_block_number.height())
            .await;

        let l2_block = self
            .l2_provider
            .get_block(update_from_block_number.height().into())
            .await
            .unwrap()
            .unwrap();

        // TODO: Arbitrum network type so we can avoid this
        let l1_height = u64::from_be_bytes(
            *l2_block
                .other
                .get_deserialized::<H64>("l1BlockNumber")
                .unwrap()
                .unwrap()
                .get(),
        );

        Ok(voyager_vm::data(OrderedHeaders {
            headers: vec![(
                DecodedHeaderMeta {
                    height: Height::new(l2_block.header.number),
                },
                into_value(Header {
                    l1_height: Height::new(l1_height),
                    l1_account_proof,
                    l2_ibc_account_proof,
                    l1_next_node_num_slot_proof: latest_confirmed_proofs
                        .latest_confirmed_slot_proof,
                    l1_nodes_slot_proof: latest_confirmed_proofs.nodes_slot_proof,
                    l2_header: L2Header {
                        parent_hash: l2_block.header.parent_hash.into(),
                        sha3_uncles: l2_block.header.ommers_hash.into(),
                        miner: l2_block.header.beneficiary.into(),
                        state_root: l2_block.header.state_root.into(),
                        transactions_root: l2_block.header.transactions_root.into(),
                        receipts_root: l2_block.header.receipts_root.into(),
                        logs_bloom: Box::new(l2_block.header.logs_bloom.0.into()),
                        difficulty: l2_block.header.difficulty.into(),
                        number: l2_block.header.number.into(),
                        gas_limit: l2_block.header.gas_limit,
                        gas_used: l2_block.header.gas_used,
                        timestamp: l2_block.header.timestamp,
                        extra_data: l2_block.header.extra_data.to_vec().try_into().unwrap(),
                        mix_hash: l2_block.header.mix_hash.unwrap_or_default().into(),
                        nonce: l2_block.header.nonce.unwrap_or_default().into(),
                        base_fee_per_gas: l2_block
                            .header
                            .base_fee_per_gas
                            .unwrap_or_default()
                            .into(),
                    },
                }),
            )],
        }))
    }
}

pub struct LatestConfirmedProofs {
    pub latest_confirmed: u64,
    pub latest_confirmed_slot_proof: StorageProof,
    pub nodes_slot_proof: StorageProof,
}
