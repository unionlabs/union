// #![warn(clippy::unwrap_used)]

use std::collections::VecDeque;

use alloy::{
    network::AnyNetwork,
    primitives::U64,
    providers::{DynProvider, Provider, ProviderBuilder},
};
use arbitrum_client::finalized_l2_block_of_l1_height;
use arbitrum_light_client_types::{ClientState, Header, L2Header};
use arbitrum_types::{L1_NEXT_NODE_NUM_SLOT, L1_NODES_CONFIRM_DATA_OFFSET, L1_NODES_SLOT};
use ethereum_light_client_types::{AccountProof, StorageProof};
use ibc_union_spec::{path::ClientStatePath, ClientId, IbcUnion};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};
use unionlabs::{
    ibc::core::client::height::Height,
    primitives::{H160, U256},
    ErrorReporter,
};
use voyager_message::{
    call::{Call, FetchUpdateHeaders, WaitForTrustedHeight},
    callback::AggregateSubmitTxFromOrderedHeaders,
    core::{ChainId, ClientType, IbcSpec, QueryHeight},
    data::{Data, DecodedHeaderMeta, OrderedHeaders},
    hook::UpdateHook,
    into_value,
    module::{PluginInfo, PluginServer},
    DefaultCmd, ExtensionsExt, Plugin, PluginMessage, RawClientId, VoyagerClient, VoyagerMessage,
    FATAL_JSONRPC_ERROR_CODE,
};
use voyager_vm::{call, conc, data, noop, pass::PassResult, promise, seq, BoxDynError, Op, Visit};

use crate::{
    call::{FetchL2Update, FetchUpdate, ModuleCall},
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
                        &ClientType::new(ClientType::ARBITRUM),
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
    async fn call(&self, e: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::FetchUpdate(FetchUpdate {
                from_height,
                to_height,
                counterparty_chain_id,
                client_id,
            }) => {
                self.fetch_update(
                    e.try_get()?,
                    from_height,
                    to_height,
                    counterparty_chain_id,
                    client_id,
                )
                .await
            }
            ModuleCall::FetchL2Update(FetchL2Update {
                update_from,
                counterparty_chain_id,
                client_id,
            }) => {
                self.fetch_l2_update(e.try_get()?, update_from, counterparty_chain_id, client_id)
                    .await
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

impl Module {
    /// Fetch the account update of the Rollup contract in the L1 state root at the specified ***L1*** block number.
    #[instrument(
        skip_all,
        fields(
            %l1_block_number,
            l1_contract_address = %self.l1_contract_address
        )
    )]
    pub async fn fetch_l1_rollup_account_update(
        &self,
        l1_block_number: u64,
    ) -> RpcResult<AccountProof> {
        let account_update = self
            .l1_provider
            .get_proof(self.l1_contract_address.into(), vec![])
            .block_id(l1_block_number.into())
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error fetching account update"),
                    None::<()>,
                )
            })?;

        debug!(storage_hash = %account_update.storage_hash, "fetched rollup account update");

        Ok(AccountProof {
            storage_root: account_update.storage_hash.into(),
            proof: account_update
                .account_proof
                .into_iter()
                .map(|x| x.into())
                .collect(),
        })
    }

    /// Fetch the settlement state and proofs, stored in the L1 Rollup contract at the specified ***L1*** block number.
    #[instrument(
        skip_all,
        fields(
            %l1_block_number,
            l1_contract_address = %self.l1_contract_address
        )
    )]
    async fn fetch_l1_latest_confirmed_proofs(
        &self,
        l1_block_number: u64,
    ) -> LatestConfirmedProofs {
        let latest_confirmed = arbitrum_client::next_node_num_at_l1_height(
            &self.l1_provider,
            self.l1_contract_address,
            l1_block_number,
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
            .block_id(l1_block_number.into())
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

    /// Fetch the account update of the IBCHandler contract in the L2 state root at the specified ***L2*** block number.
    async fn fetch_l2_ibc_contract_root_proof(&self, l2_block_number: u64) -> AccountProof {
        let proof = self
            .l2_provider
            .get_proof(self.ibc_handler_address.into(), vec![])
            .block_id(l2_block_number.into())
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
            %update_from,
            %update_to,
            %client_id,
        )
    )]
    async fn fetch_update(
        &self,
        voyager_client: &VoyagerClient,
        update_from: Height,
        update_to: Height,
        counterparty_chain_id: ChainId,
        client_id: RawClientId,
    ) -> RpcResult<Op<VoyagerMessage>> {
        let client_id = client_id.decode_spec::<IbcUnion>().map_err(|e| {
            ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                ErrorReporter(e).with_message("invalid client id"),
                None::<()>,
            )
        })?;

        // client info

        let counterparty_latest_height = voyager_client
            .query_latest_height(counterparty_chain_id.clone(), false)
            .await?;

        let arbitrum_client_state_raw = voyager_client
            .query_ibc_state(
                counterparty_chain_id.clone(),
                counterparty_latest_height,
                ClientStatePath { client_id },
            )
            .await?;

        let arbitrum_client_info = voyager_client
            .client_info::<IbcUnion>(counterparty_chain_id.clone(), client_id)
            .await?;

        let ClientState::V1(arbitrum_client_state) = voyager_client
            .decode_client_state::<IbcUnion, ClientState>(
                arbitrum_client_info.client_type,
                arbitrum_client_info.ibc_interface,
                arbitrum_client_state_raw,
            )
            .await?;

        // the client on the counterparty chain tracking the L1 that the L2 being tracked by the client we're updating settles on
        let l1_client_meta = voyager_client
            .client_state_meta::<IbcUnion>(
                counterparty_chain_id.clone(),
                QueryHeight::Latest,
                arbitrum_client_state.l1_client_id,
            )
            .await?;

        let l1_client_info = voyager_client
            .client_info::<IbcUnion>(
                counterparty_chain_id.clone(),
                arbitrum_client_state.l1_client_id,
            )
            .await?;

        // client update data

        // the L1 height corresponding to the *exact* L2 height we're trying to update to
        // note that this is not necessarily the height that we *will* update to, see below
        let l1_height_of_l2_update_to = self
            .l2_provider
            .get_block(update_to.height().into())
            .await
            .unwrap()
            .unwrap()
            .other
            // TODO: Arbitrum network type so we can avoid this
            .get_deserialized::<U64>("l1BlockNumber")
            .unwrap()
            .unwrap()
            .into_limbs()[0];

        // the L2 block that was settled in a stakeOnNewNode transaction
        // this is the block closest to, but not before, the requested update height
        let l2_settlement_block = finalized_l2_block_of_l1_height(
            &self.l1_provider,
            &self.l2_provider,
            self.l1_contract_address,
            l1_height_of_l2_update_to,
        )
        .await
        .unwrap();

        let l1_height_of_l2_settlement_block = l2_settlement_block
            .other
            // TODO: Arbitrum network type so we can avoid this
            .get_deserialized::<U64>("l1BlockNumber")
            .unwrap()
            .unwrap()
            .into_limbs()[0];

        info!(
            "l2 settlement block height {}",
            l2_settlement_block.header.number
        );

        if l1_client_meta.counterparty_height.height() >= l1_height_of_l2_settlement_block {
            info!(
                "l1 client {l1_client} (trusted height {l1_trusted_height}) \
                    is already updated to a height >= the l1 height of closest \
                    settlement l2 block {l1_height_of_l2_settlement_block}",
                l1_client = arbitrum_client_state.l1_client_id,
                l1_trusted_height = l1_client_meta.counterparty_height,
            );

            Ok(call(PluginMessage::new(
                self.plugin_name(),
                ModuleCall::from(FetchL2Update {
                    update_from,
                    counterparty_chain_id,
                    client_id,
                }),
            )))
        } else {
            Ok(conc([
                promise(
                    [call(FetchUpdateHeaders {
                        client_type: l1_client_info.client_type,
                        chain_id: l1_client_meta.counterparty_chain_id.clone(),
                        counterparty_chain_id: counterparty_chain_id.clone(),
                        client_id: RawClientId::new(arbitrum_client_state.l1_client_id),
                        update_from: l1_client_meta.counterparty_height,
                        update_to: Height::new(l2_settlement_block.header.number),
                    })],
                    [],
                    AggregateSubmitTxFromOrderedHeaders {
                        ibc_spec_id: IbcUnion::ID,
                        chain_id: counterparty_chain_id.clone(),
                        client_id: RawClientId::new(arbitrum_client_state.l1_client_id),
                    },
                ),
                call(PluginMessage::new(
                    self.plugin_name(),
                    ModuleCall::from(FetchL2Update {
                        update_from,
                        counterparty_chain_id,
                        client_id,
                    }),
                )),
            ]))
        }
    }

    #[instrument(
        skip_all,
        fields(
            chain_id = %self.chain_id,
            %counterparty_chain_id,
            %update_from,
            %client_id,
        )
    )]
    async fn fetch_l2_update(
        &self,
        voyager_client: &VoyagerClient,
        update_from: Height,
        counterparty_chain_id: ChainId,
        client_id: ClientId,
    ) -> RpcResult<Op<VoyagerMessage>> {
        // client info

        let counterparty_latest_height = voyager_client
            .query_latest_height(counterparty_chain_id.clone(), false)
            .await?;

        let arbitrum_client_state_raw = voyager_client
            .query_ibc_state(
                counterparty_chain_id.clone(),
                counterparty_latest_height,
                ClientStatePath { client_id },
            )
            .await?;

        let arbitrum_client_info = voyager_client
            .client_info::<IbcUnion>(counterparty_chain_id.clone(), client_id)
            .await?;

        let ClientState::V1(arbitrum_client_state) = voyager_client
            .decode_client_state::<IbcUnion, ClientState>(
                arbitrum_client_info.client_type,
                arbitrum_client_info.ibc_interface,
                arbitrum_client_state_raw,
            )
            .await?;

        // the client on the counterparty chain tracking the L1 that the L2 being tracked by the client we're updating settles on
        let l1_client_meta = voyager_client
            .client_state_meta::<IbcUnion>(
                counterparty_chain_id.clone(),
                QueryHeight::Latest,
                arbitrum_client_state.l1_client_id,
            )
            .await?;

        let l2_settlement_block = finalized_l2_block_of_l1_height(
            &self.l1_provider,
            &self.l2_provider,
            self.l1_contract_address,
            l1_client_meta.counterparty_height.height(),
        )
        .await
        .unwrap();

        if l2_settlement_block.header.number < update_from.height() {
            return Err(ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!(
                    "attempted to update to a height ({to_height}) \
                    < the intended update_from height {update_from}",
                    to_height = l2_settlement_block.header.number
                ),
                None::<()>,
            ));
        }

        let l1_account_proof = self
            .fetch_l1_rollup_account_update(l1_client_meta.counterparty_height.height())
            .await
            .unwrap();

        let l1_latest_confirmed_proofs = self
            .fetch_l1_latest_confirmed_proofs(l1_client_meta.counterparty_height.height())
            .await;

        let l2_ibc_account_proof = self
            .fetch_l2_ibc_contract_root_proof(l2_settlement_block.header.number)
            .await;

        Ok(data(OrderedHeaders {
            headers: vec![(
                DecodedHeaderMeta {
                    height: Height::new(l2_settlement_block.header.number),
                },
                into_value(Header {
                    l1_height: l1_client_meta.counterparty_height,
                    l1_account_proof,
                    l2_ibc_account_proof,
                    l1_next_node_num_slot_proof: l1_latest_confirmed_proofs
                        .latest_confirmed_slot_proof,
                    l1_nodes_slot_proof: l1_latest_confirmed_proofs.nodes_slot_proof,
                    l2_header: L2Header {
                        parent_hash: l2_settlement_block.header.parent_hash.into(),
                        sha3_uncles: l2_settlement_block.header.ommers_hash.into(),
                        miner: l2_settlement_block.header.beneficiary.into(),
                        state_root: l2_settlement_block.header.state_root.into(),
                        transactions_root: l2_settlement_block.header.transactions_root.into(),
                        receipts_root: l2_settlement_block.header.receipts_root.into(),
                        logs_bloom: Box::new(l2_settlement_block.header.logs_bloom.0.into()),
                        difficulty: l2_settlement_block.header.difficulty.into(),
                        number: l2_settlement_block.header.number.into(),
                        gas_limit: l2_settlement_block.header.gas_limit,
                        gas_used: l2_settlement_block.header.gas_used,
                        timestamp: l2_settlement_block.header.timestamp,
                        extra_data: l2_settlement_block
                            .header
                            .extra_data
                            .to_vec()
                            .try_into()
                            .unwrap(),
                        mix_hash: l2_settlement_block
                            .header
                            .mix_hash
                            .unwrap_or_default()
                            .into(),
                        nonce: l2_settlement_block.header.nonce.unwrap_or_default().into(),
                        base_fee_per_gas: l2_settlement_block
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
