// #![warn(clippy::unwrap_used)]

use std::collections::VecDeque;

use alloy::{
    network::AnyNetwork,
    providers::{DynProvider, Provider, ProviderBuilder},
};
use bob_client::{finalized_execution_block_of_l1_height, output_index_of_l2_block_on_l1_block};
use bob_light_client_types::{
    header::{L2Header, OutputRootProof},
    ClientState, Header,
};
use bob_types::{L2_OUTPUTS_SLOT, L2_TO_L1_MESSAGE_PASSER};
use bob_verifier::FINALIZATION_PERIOD_SECONDS;
use call::FetchL2Update;
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
    primitives::{H160, H256, U256},
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
};
use voyager_vm::{call, conc, data, noop, pass::PassResult, promise, seq, BoxDynError, Op, Visit};

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

    pub l2_oracle_address: H160,

    pub l1_provider: DynProvider,
    pub l2_provider: DynProvider<AnyNetwork>,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub l2_chain_id: ChainId,

    /// The L2 oracle contract on the L1.
    pub l2_oracle_address: H160,

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
    pub async fn fetch_oracle_account_proof(&self, block_number: u64) -> RpcResult<AccountProof> {
        let account_update = self
            .l1_provider
            .get_proof(self.l2_oracle_address.into(), vec![])
            .block_id(block_number.into())
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error fetching oracle account proof"),
                    None::<()>,
                )
            })?;

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

    pub async fn fetch_output_proposal_proof(
        &self,
        output_index: u32,
        height: u64,
    ) -> RpcResult<StorageProof> {
        let [proof]: [_; 1] = self
            .l1_provider
            .get_proof(
                self.l2_oracle_address.into(),
                vec![bob_verifier::compute_output_proposal_slot(
                    L2_OUTPUTS_SLOT.into(),
                    output_index,
                )
                .to_be_bytes()
                .into()],
            )
            .block_id(height.into())
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching output proposal proof: {}", ErrorReporter(e)),
                    None::<()>,
                )
            })?
            .storage_proof
            .try_into()
            .unwrap();
        Ok(StorageProof {
            key: U256::from_be_bytes(proof.key.as_b256().0),
            value: U256::from_be_bytes(proof.value.to_be_bytes()),
            proof: proof.proof.into_iter().map(|bytes| bytes.into()).collect(),
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

        let l2_chain_id = ChainId::new(l2_provider.get_chain_id().await?.to_string());

        assert_eq!(l2_chain_id, config.l2_chain_id);

        Ok(Self {
            chain_id: l2_chain_id,
            l2_oracle_address: config.l2_oracle_address,
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
                &ClientType::new(ClientType::BOB),
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
                    UpdateHook::new(&self.chain_id, &ClientType::new(ClientType::BOB), |fetch| {
                        Call::Plugin(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(FetchUpdate {
                                from_height: fetch.update_from,
                                to_height: fetch.update_to,
                                counterparty_chain_id: fetch.counterparty_chain_id.clone(),
                                client_id: fetch
                                    .client_id
                                    .clone()
                                    .decode_spec::<IbcUnion>()
                                    .unwrap(),
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
    async fn call(&self, e: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::FetchUpdate(FetchUpdate {
                from_height,
                to_height,
                counterparty_chain_id,
                client_id,
            }) => self
                .fetch_update(
                    e.try_get()?,
                    from_height,
                    to_height,
                    counterparty_chain_id,
                    client_id,
                )
                .await
                .map_err(|e| {
                    ErrorObject::owned(
                        -1,
                        format!("error fetching update: {}", ErrorReporter(&*e)),
                        None::<()>,
                    )
                }),
            ModuleCall::FetchL2Update(FetchL2Update {
                update_from,
                counterparty_chain_id,
                client_id,
            }) => self
                .fetch_l2_update(e.try_get()?, update_from, counterparty_chain_id, client_id)
                .await
                .map_err(|e| {
                    ErrorObject::owned(
                        -1,
                        format!("error fetching l2 update: {}", ErrorReporter(&*e)),
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
    async fn fetch_ibc_contract_root_proof(&self, height: u64) -> RpcResult<AccountProof> {
        let proof = self
            .l2_provider
            .get_proof(self.ibc_handler_address.into(), vec![])
            .block_id(height.into())
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching ibc contract proof: {}", ErrorReporter(e)),
                    None::<()>,
                )
            })?;
        Ok(AccountProof {
            storage_root: proof.storage_hash.into(),
            proof: proof.account_proof.into_iter().map(|x| x.into()).collect(),
        })
    }

    async fn fetch_output_root_proof(&self, height: u64) -> RpcResult<OutputRootProof> {
        let l2_block = self
            .l2_provider
            .get_block(height.into())
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching output root proof: {}", ErrorReporter(e)),
                    None::<()>,
                )
            })?
            .unwrap();
        let message_passer_storage_root = self
            .l2_provider
            .get_proof(L2_TO_L1_MESSAGE_PASSER.into(), vec![])
            .block_id(height.into())
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!(
                        "error fetching message passer storage root: {}",
                        ErrorReporter(e)
                    ),
                    None::<()>,
                )
            })?
            .storage_hash
            .into();
        Ok(OutputRootProof {
            // Seems to always be zero.
            version: H256::default(),
            state_root: l2_block.header.state_root.into(),
            message_passer_storage_root,
            latest_block_hash: l2_block.header.hash.into(),
        })
    }

    #[instrument(
        skip_all,
        fields(
            chain_id = %self.chain_id,
            %counterparty_chain_id,
            %update_from,
            %update_to,
        )
    )]
    async fn fetch_update(
        &self,
        voy_client: &VoyagerClient,
        update_from: Height,
        update_to: Height,
        counterparty_chain_id: ChainId,
        client_id: ClientId,
    ) -> Result<Op<VoyagerMessage>, BoxDynError> {
        let counterparty_latest_height = voy_client
            .query_latest_height(counterparty_chain_id.clone(), false)
            .await?;

        let raw_bob_client_state = voy_client
            .query_ibc_state(
                counterparty_chain_id.clone(),
                counterparty_latest_height,
                ClientStatePath { client_id },
            )
            .await?;

        debug!(?raw_bob_client_state);

        let bob_client_state_info = voy_client
            .client_info::<IbcUnion>(counterparty_chain_id.clone(), client_id)
            .await?;

        debug!(?bob_client_state_info);

        let ClientState::V1(bob_client_state) = voy_client
            .decode_client_state::<IbcUnion, ClientState>(
                bob_client_state_info.client_type.clone(),
                bob_client_state_info.ibc_interface,
                raw_bob_client_state,
            )
            .await?;

        debug!(?bob_client_state);

        if bob_client_state.latest_height >= update_to.height() {
            info!("bob: irrelevant update");
            Ok(noop())
        } else {
            let l1_client_info = voy_client
                .client_info::<IbcUnion>(self.chain_id.clone(), bob_client_state.l1_client_id)
                .await?;

            let l1_client_meta = voy_client
                .client_state_meta::<IbcUnion>(
                    self.chain_id.clone(),
                    QueryHeight::Finalized,
                    bob_client_state.l1_client_id,
                )
                .await?;

            // Latest L1 finalized height
            let l1_latest_height = voy_client
                .query_latest_height(l1_client_meta.counterparty_chain_id.clone(), true)
                .await?;

            Ok(conc([
                promise(
                    [call(FetchUpdateHeaders {
                        client_type: l1_client_info.client_type,
                        chain_id: l1_client_meta.counterparty_chain_id.clone(),
                        counterparty_chain_id: counterparty_chain_id.clone(),
                        client_id: RawClientId::new(bob_client_state.l1_client_id),
                        update_from: l1_client_meta.counterparty_height,
                        update_to: l1_latest_height,
                    })],
                    [],
                    AggregateSubmitTxFromOrderedHeaders {
                        ibc_spec_id: IbcUnion::ID,
                        chain_id: counterparty_chain_id.clone(),
                        client_id: RawClientId::new(bob_client_state.l1_client_id),
                    },
                ),
                seq([
                    call(WaitForTrustedHeight {
                        chain_id: counterparty_chain_id.clone(),
                        ibc_spec_id: IbcUnion::ID,
                        client_id: RawClientId::new(bob_client_state.l1_client_id),
                        height: l1_latest_height,
                        finalized: false,
                    }),
                    call(PluginMessage::new(
                        self.plugin_name(),
                        ModuleCall::from(FetchL2Update {
                            update_from,
                            counterparty_chain_id,
                            client_id,
                        }),
                    )),
                ]),
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
        voy_client: &VoyagerClient,
        update_from: Height,
        counterparty_chain_id: ChainId,
        client_id: ClientId,
    ) -> Result<Op<VoyagerMessage>, BoxDynError> {
        let counterparty_latest_height = voy_client
            .query_latest_height(counterparty_chain_id.clone(), false)
            .await?;

        let raw_bob_client_state = voy_client
            .query_ibc_state(
                counterparty_chain_id.clone(),
                counterparty_latest_height,
                ClientStatePath { client_id },
            )
            .await?;

        debug!(?raw_bob_client_state);

        let bob_client_state_info = voy_client
            .client_info::<IbcUnion>(counterparty_chain_id.clone(), client_id)
            .await?;

        debug!(?bob_client_state_info);

        let ClientState::V1(bob_client_state) = voy_client
            .decode_client_state::<IbcUnion, ClientState>(
                bob_client_state_info.client_type.clone(),
                bob_client_state_info.ibc_interface,
                raw_bob_client_state,
            )
            .await?;

        debug!(?bob_client_state);

        let l1_client_meta = voy_client
            .client_state_meta::<IbcUnion>(
                self.chain_id.clone(),
                QueryHeight::Finalized,
                bob_client_state.l1_client_id,
            )
            .await?;

        let l1_height = l1_client_meta.counterparty_height.height();

        let l2_block = finalized_execution_block_of_l1_height(
            &self.l1_provider,
            &self.l2_provider,
            self.l2_oracle_address,
            FINALIZATION_PERIOD_SECONDS,
            l1_height,
        )
        .await
        .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))?;

        // Guarantee to exist because we know the latest committed exists.
        let output_index = output_index_of_l2_block_on_l1_block(
            &self.l1_provider,
            self.l2_oracle_address,
            l2_block.header.number,
            l1_height,
        )
        .await
        .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))?;

        // Extract proofs for the output proposal of the L2 oracle contract on
        // the L1.
        let output_proposal_proof = self
            .fetch_output_proposal_proof(output_index, l1_height)
            .await?;
        let l2_oracle_account_proof = self.fetch_oracle_account_proof(l1_height).await?;

        // Extract proofs for the preimage of the output proposal committed on
        // L1 (extracted above) and IBC handle contract on the L2.
        let output_root_proof = self.fetch_output_root_proof(l2_block.header.number).await?;
        let l2_ibc_account_proof = self
            .fetch_ibc_contract_root_proof(l2_block.header.number)
            .await?;

        Ok(data(OrderedHeaders {
            headers: vec![(
                DecodedHeaderMeta {
                    height: Height::new(l2_block.header.number),
                },
                into_value(Header {
                    l1_height,
                    l2_oracle_account_proof,
                    l2_ibc_account_proof,
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
                        withdrawals_root: l2_block.header.withdrawals_root.unwrap().into(),
                        blob_gas_used: l2_block.header.blob_gas_used.unwrap(),
                        excess_blob_gas: l2_block.header.excess_blob_gas.unwrap(),
                        parent_beacon_block_root: l2_block
                            .header
                            .parent_beacon_block_root
                            .unwrap()
                            .into(),
                    },
                    l2_oracle_l2_outputs_slot_proof: output_proposal_proof,
                    output_index,
                    output_root_proof,
                }),
            )],
        }))
    }
}
