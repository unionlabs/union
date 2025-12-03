// #![warn(clippy::unwrap_used)]

use std::collections::VecDeque;

use alloy::{
    network::AnyNetwork,
    providers::{DynProvider, Provider, ProviderBuilder},
};
use base_light_client_types::{
    ClientState, Header,
    header::{L2Header, OutputRootProof},
};
use call::FetchL2Update;
use ethereum_light_client_types::{AccountProof, StorageProof};
use ibc_union_spec::{ClientId, IbcUnion, path::ClientStatePath};
use jsonrpsee::{Extensions, core::async_trait};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};
use unionlabs::{
    ibc::core::client::height::Height,
    never::Never,
    primitives::{ByteArrayExt, Bytes, H160, H256, U256, encoding::HexPrefixed},
};
use voyager_sdk::{
    DefaultCmd, ExtensionsExt, VoyagerClient, anyhow,
    hook::UpdateHook,
    into_value,
    message::{
        PluginMessage, VoyagerMessage,
        call::{Call, FetchUpdateHeaders, WaitForHeightRelative, WaitForTrustedHeight},
        callback::AggregateSubmitTxFromOrderedHeaders,
        data::{Data, DecodedHeaderMeta, OrderedHeaders},
    },
    plugin::Plugin,
    primitives::{ChainId, ClientType, IbcSpec, QueryHeight},
    rpc::{PluginServer, RpcError, RpcResult, types::PluginInfo},
    types::RawClientId,
    vm::{Op, Visit, call, conc, data, pass::PassResult, promise, seq},
};

use crate::call::{FetchUpdate, ModuleCall};

pub mod call;

#[tokio::main]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub l1_dispute_game_factory_proxy: H160,
    pub dispute_game_factory_dispute_game_list_slot: U256,
    pub fault_dispute_game_code_root_claim_index: usize,

    pub l1_provider: DynProvider,
    pub l2_provider: DynProvider<AnyNetwork>,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,
}

fn default_client_type() -> ClientType {
    ClientType::new(ClientType::BASE)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    // Default to base for backward compatibility.
    #[serde(default = "default_client_type")]
    pub client_type: ClientType,

    pub l2_chain_id: ChainId,

    pub l1_dispute_game_factory_proxy: H160,
    pub dispute_game_factory_dispute_game_list_slot: U256,
    pub fault_dispute_game_code_root_claim_index: usize,

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
    pub async fn fetch_dispute_game_factory_account_proof(
        &self,
        block_number: u64,
    ) -> RpcResult<AccountProof> {
        let account_update = self
            .l1_provider
            .get_proof(self.l1_dispute_game_factory_proxy.into(), vec![])
            .block_id(block_number.into())
            .await
            .map_err(RpcError::fatal(
                "error fetching dispute game factory account proof",
            ))?;

        debug!(storage_hash = %account_update.storage_hash, "fetched dispute game factory account update");

        Ok(AccountProof {
            storage_root: account_update.storage_hash.into(),
            proof: account_update
                .account_proof
                .into_iter()
                .map(|x| x.into())
                .collect(),
        })
    }

    pub async fn fetch_game_proof(&self, game_index: U256, height: u64) -> RpcResult<StorageProof> {
        let [proof]: [_; 1] = self
            .l1_provider
            .get_proof(
                self.l1_dispute_game_factory_proxy.into(),
                vec![
                    base_verifier::compute_game_slot(
                        self.dispute_game_factory_dispute_game_list_slot,
                        game_index,
                    )
                    .to_be_bytes()
                    .into(),
                ],
            )
            .block_id(height.into())
            .await
            .map_err(RpcError::fatal("error fetching output proposal proof"))?
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
    type Callback = Never;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> anyhow::Result<Self> {
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
            l1_provider,
            l2_provider,
            ibc_handler_address: config.ibc_handler_address,
            l1_dispute_game_factory_proxy: config.l1_dispute_game_factory_proxy,
            dispute_game_factory_dispute_game_list_slot: config
                .dispute_game_factory_dispute_game_list_slot,
            fault_dispute_game_code_root_claim_index: config
                .fault_dispute_game_code_root_claim_index,
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.l2_chain_id),
            interest_filter: UpdateHook::filter(&config.l2_chain_id, &config.client_type),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

#[async_trait]
impl PluginServer<ModuleCall, Never> for Module {
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
                        &ClientType::new(ClientType::BASE),
                        |fetch| {
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
                    e.voyager_client()?,
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
                self.fetch_l2_update(
                    e.voyager_client()?,
                    update_from,
                    counterparty_chain_id,
                    client_id,
                )
                .await
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn callback(
        &self,
        _: &Extensions,
        cb: Never,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }
}

impl Module {
    async fn fetch_game_account_code(&self, game_account: H160) -> RpcResult<Vec<u8>> {
        Ok(self
            .l1_provider
            .get_code_at(game_account.into())
            .await
            .map_err(RpcError::retryable("error fetching game account code"))?
            .into())
    }

    async fn fetch_game_account_proof(
        &self,
        height: u64,
        game_account: H160,
    ) -> RpcResult<AccountProof> {
        let proof = self
            .l1_provider
            .get_proof(game_account.into(), vec![])
            .block_id(height.into())
            .await
            .map_err(RpcError::retryable("error fetching game account proof"))?;

        Ok(AccountProof {
            storage_root: proof.storage_hash.into(),
            proof: proof.account_proof.into_iter().map(|x| x.into()).collect(),
        })
    }

    async fn fetch_ibc_contract_root_proof(&self, height: u64) -> RpcResult<AccountProof> {
        let proof = self
            .l2_provider
            .get_proof(self.ibc_handler_address.into(), vec![])
            .block_id(height.into())
            .await
            .map_err(RpcError::retryable("error fetching ibc contract proof"))?;

        Ok(AccountProof {
            storage_root: proof.storage_hash.into(),
            proof: proof.account_proof.into_iter().map(|x| x.into()).collect(),
        })
    }

    async fn fetch_output_root_proof(&self, l2_height: u64) -> RpcResult<OutputRootProof> {
        let l2_block = self
            .l2_provider
            .get_block(l2_height.into())
            .await
            .map_err(RpcError::retryable("error fetching output root proof"))?
            .unwrap();

        let message_passer_storage_root = self
            .l2_provider
            // TODO: refactor this in a common crate for opstack
            .get_proof(bob_types::L2_TO_L1_MESSAGE_PASSER.into(), vec![])
            .block_id(l2_height.into())
            .await
            .map_err(RpcError::retryable(
                "error fetching message passer storage root",
            ))?
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
    ) -> RpcResult<Op<VoyagerMessage>> {
        let counterparty_latest_height = voy_client
            .query_latest_height(counterparty_chain_id.clone(), false)
            .await?;

        let raw_base_client_state = voy_client
            .query_ibc_state(
                counterparty_chain_id.clone(),
                QueryHeight::Specific(counterparty_latest_height),
                ClientStatePath { client_id },
            )
            .await?;

        debug!(?raw_base_client_state);

        let base_client_state_info = voy_client
            .client_info::<IbcUnion>(counterparty_chain_id.clone(), client_id)
            .await?;

        debug!(?base_client_state_info);

        let ClientState::V1(base_client_state) = voy_client
            .decode_client_state::<IbcUnion, ClientState>(
                base_client_state_info.client_type.clone(),
                base_client_state_info.ibc_interface,
                raw_base_client_state,
            )
            .await?;

        debug!(?base_client_state);

        if base_client_state.latest_height >= update_to.height() {
            info!("base: irrelevant update");
            Ok(data(OrderedHeaders { headers: vec![] }))
        } else {
            let l1_client_info = voy_client
                .client_info::<IbcUnion>(
                    counterparty_chain_id.clone(),
                    base_client_state.l1_client_id,
                )
                .await?;

            let l1_client_meta = voy_client
                .client_state_meta::<IbcUnion>(
                    counterparty_chain_id.clone(),
                    QueryHeight::Latest,
                    base_client_state.l1_client_id,
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
                        client_id: RawClientId::new(base_client_state.l1_client_id),
                        update_from: l1_client_meta.counterparty_height,
                        update_to: l1_latest_height,
                    })],
                    [],
                    AggregateSubmitTxFromOrderedHeaders {
                        ibc_spec_id: IbcUnion::ID,
                        chain_id: counterparty_chain_id.clone(),
                        client_id: RawClientId::new(base_client_state.l1_client_id),
                    },
                ),
                seq([
                    call(WaitForTrustedHeight {
                        chain_id: counterparty_chain_id.clone(),
                        ibc_spec_id: IbcUnion::ID,
                        client_id: RawClientId::new(base_client_state.l1_client_id),
                        height: l1_latest_height,
                        finalized: false,
                    }),
                    // wait for 1 extra block to ensure that the L1 update is in state, and this update will not end up in the same block (and potentially get reordered)
                    call(WaitForHeightRelative {
                        chain_id: counterparty_chain_id.clone(),
                        height_diff: 1,
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
    ) -> RpcResult<Op<VoyagerMessage>> {
        let counterparty_latest_height = voy_client
            .query_latest_height(counterparty_chain_id.clone(), false)
            .await?;

        let raw_base_client_state = voy_client
            .query_ibc_state(
                counterparty_chain_id.clone(),
                QueryHeight::Specific(counterparty_latest_height),
                ClientStatePath { client_id },
            )
            .await?;

        debug!(?raw_base_client_state);

        let base_client_state_info = voy_client
            .client_info::<IbcUnion>(counterparty_chain_id.clone(), client_id)
            .await?;

        debug!(?base_client_state_info);

        let ClientState::V1(base_client_state) = voy_client
            .decode_client_state::<IbcUnion, ClientState>(
                base_client_state_info.client_type.clone(),
                base_client_state_info.ibc_interface,
                raw_base_client_state,
            )
            .await?;

        debug!(?base_client_state);

        let l1_client_meta = voy_client
            .client_state_meta::<IbcUnion>(
                counterparty_chain_id.clone(),
                QueryHeight::Latest,
                base_client_state.l1_client_id,
            )
            .await?;

        let l1_height = l1_client_meta.counterparty_height.height();

        let l2_block_number = base_client::finalized_l2_block_number_of_l1_block_number(
            &self.l1_provider,
            self.l1_dispute_game_factory_proxy,
            l1_height,
        )
        .await
        .map_err(RpcError::retryable(
            "finalized l2 block number of l1 block number",
        ))?;

        let l2_block = self
            .l2_provider
            .get_block(l2_block_number.into())
            .await
            .map_err(RpcError::retryable("error fetching finalized l2 block"))?
            .expect("block should exist");

        let output_root_proof = self.fetch_output_root_proof(l2_block.header.number).await?;

        let game_index = base_client::latest_game_of_l1_block_number(
            &self.l1_provider,
            l1_height,
            self.l1_dispute_game_factory_proxy,
        )
        .await
        .map_err(RpcError::retryable(
            "error fetching latest game of l1 block number",
        ))?;

        let game_index = game_index - U256::ONE;

        let dispute_game_factory_account_proof = self
            .fetch_dispute_game_factory_account_proof(l1_height)
            .await?;

        let game_proof = self.fetch_game_proof(game_index, l1_height).await?;

        let game_id = game_proof.value.to_be_bytes();
        // TODO: introduce a gameId type with a tryfrom
        // See https://github.com/ethereum-optimism/optimism/blob/4a7cb8a198a1f027e739d2e51dc170faf02b5d28/packages/contracts-bedrock/src/dispute/lib/LibUDT.sol#L70-L79
        let game_account_address = H160::<HexPrefixed>::new(game_id.array_slice::<12, 20>());

        let game_account_proof = self
            .fetch_game_account_proof(l1_height, game_account_address)
            .await?;
        let game_account_code = self.fetch_game_account_code(game_account_address).await?;

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
                        extra_data: <Bytes>::from(l2_block.header.extra_data.clone())
                            .try_into()
                            .unwrap(),
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
                        requests_hash: l2_block.header.requests_hash.unwrap().into(),
                    },
                    dispute_game_factory_account_proof,
                    output_root_proof,
                    game_index,
                    game_proof,
                    game_account_proof,
                    game_account_code: game_account_code.into(),
                }),
            )],
        }))
    }
}
