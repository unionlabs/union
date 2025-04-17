use std::{collections::VecDeque, fmt::Debug, num::ParseIntError};

use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use beacon_api_types::{chain_spec::Mainnet, deneb};
use berachain_light_client_types::{ClientState, Header};
use ethereum_light_client_types::AccountProof;
use ibc_union_spec::{IbcUnion, path::ClientStatePath};
use jsonrpsee::{
    Extensions,
    core::{RpcResult, async_trait},
    types::ErrorObject,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use unionlabs::{
    ErrorReporter,
    berachain::LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX,
    encoding::{DecodeAs, Ssz},
    ibc::core::commitment::merkle_proof::MerkleProof,
    never::Never,
    primitives::H160,
};
use voyager_sdk::{
    DefaultCmd,
    anyhow::{self, bail},
    hook::UpdateHook,
    into_value,
    message::{
        PluginMessage, VoyagerMessage,
        call::{Call, FetchUpdateHeaders, WaitForTrustedHeight},
        data::{Data, DecodedHeaderMeta, OrderedHeaders},
    },
    plugin::Plugin,
    primitives::{ChainId, ClientType, IbcSpecId},
    rpc::{PluginServer, types::PluginInfo},
    types::RawClientId,
    vm::{Op, Visit, call, conc, data, pass::PassResult, seq},
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
    pub ibc_handler_address: H160,
    pub eth_provider: DynProvider,
    pub cometbft_client: cometbft_rpc::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,
    pub ibc_handler_address: H160,
    pub comet_rpc_url: String,
    pub eth_rpc_url: String,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = Never;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> anyhow::Result<Self> {
        let eth_provider =
            DynProvider::new(ProviderBuilder::new().connect(&config.eth_rpc_url).await?);

        let chain_id = ChainId::new(eth_provider.get_chain_id().await?.to_string());

        if chain_id != config.chain_id {
            return Err(format!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.chain_id, chain_id
            )
            .into());
        }

        let tm_client = cometbft_rpc::Client::new(config.comet_rpc_url).await?;

        Ok(Self {
            chain_id: config.chain_id,
            ibc_handler_address: config.ibc_handler_address,
            eth_provider,
            cometbft_client: tm_client,
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: UpdateHook::filter(
                &config.chain_id,
                &ClientType::new(ClientType::BEACON_KIT),
            ),
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

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }

    // TODO: deduplicate with eth client update module?
    pub async fn fetch_account_update(&self, block_number: u64) -> RpcResult<AccountProof> {
        let account_update = self
            .eth_provider
            .get_proof(self.ibc_handler_address.into(), vec![])
            .block_id(block_number.into())
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error fetching account update"),
                    None::<()>,
                )
            })?;

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

#[derive(Debug, thiserror::Error)]
#[error("unable to parse chain id: expected format `<chain>-<revision-number>`, found `{found}`")]
pub struct ChainIdParseError {
    found: String,
    #[source]
    source: Option<ParseIntError>,
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
                        &ClientType::new(ClientType::BEACON_KIT),
                        |fetch| {
                            Call::Plugin(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(FetchUpdate {
                                    counterparty_chain_id: fetch.counterparty_chain_id.clone(),
                                    update_from: fetch.update_from,
                                    update_to: fetch.update_to,
                                    client_id: fetch
                                        .client_id
                                        .clone()
                                        .decode_spec::<IbcUnion>()
                                        .expect("bad client id?"),
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
                counterparty_chain_id,
                update_from,
                update_to,
                client_id,
            }) => {
                let voyager_client = e.try_get::<VoyagerClient>()?;

                let counterparty_latest_height = voyager_client
                    .query_latest_height(counterparty_chain_id.clone(), false)
                    .await?;

                let beacon_kit_client_state_raw = voyager_client
                    .query_ibc_state(
                        counterparty_chain_id.clone(),
                        counterparty_latest_height,
                        ClientStatePath { client_id },
                    )
                    .await?;

                let beacon_kit_client_info = voyager_client
                    .client_info::<IbcUnion>(counterparty_chain_id.clone(), client_id)
                    .await?;

                let ClientState::V1(beacon_kit_client_state) = voyager_client
                    .decode_client_state::<IbcUnion, ClientState>(
                        beacon_kit_client_info.client_type,
                        beacon_kit_client_info.ibc_interface,
                        beacon_kit_client_state_raw,
                    )
                    .await?;

                let l1_client_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        counterparty_chain_id.clone(),
                        QueryHeight::Latest,
                        beacon_kit_client_state.l1_client_id,
                    )
                    .await?;

                // NOTE: The implementation is simple because the cometbft/beacon/execution heights are guaranteed to be the same (i.e. we don't need to find the consensus height to match the execution height)
                let query_result = self
                    .cometbft_client
                    .abci_query(
                        "store/beacon/key",
                        [LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX],
                        // proof for height H must be queried at H-1
                        Some((update_to.height() as i64 - 1).try_into().unwrap()),
                        true,
                    )
                    .await
                    .unwrap();

                let execution_header =
                    deneb::ExecutionPayloadHeaderSsz::<Mainnet>::decode_as::<Ssz>(
                        query_result.response.value.expect("big trouble").as_ref(),
                    )
                    .expect("big trouble");

                let account_proof = self.fetch_account_update(update_to.height()).await?;

                let header = Header {
                    l1_height: update_to,
                    execution_header: execution_header.into(),
                    execution_header_proof:
                        MerkleProof::try_from(protos::ibc::core::commitment::v1::MerkleProof {
                            proofs: query_result
                                .response
                                .proof_ops
                                .unwrap()
                                .ops
                                .into_iter()
                                .map(|op| {
                                    <protos::cosmos::ics23::v1::CommitmentProof as prost::Message>::decode(
                                        &*op.data,
                                    )
                                        .unwrap()
                                })
                                .collect::<Vec<_>>(),
                        }).unwrap(),
                    account_proof,
                };

                // Recursively dispatch a L1 update before dispatching the L2 update.
                Ok(conc([
                    promise(
                        [call(FetchUpdateHeaders {
                            client_type: ClientType::new(ClientType::TENDERMINT),
                            counterparty_chain_id: counterparty_chain_id.clone(),
                            chain_id: l1_client_meta.counterparty_chain_id.clone(),
                            client_id: RawClientId::new(beacon_kit_client_state.l1_client_id),
                            update_from,
                            update_to,
                        })],
                        [],
                        AggregateSubmitTxFromOrderedHeaders {
                            ibc_spec_id: IbcUnion::ID,
                            chain_id: l1_client_meta.counterparty_chain_id.clone(),
                            client_id: RawClientId::new(beacon_kit_client_state.l1_client_id),
                        },
                    ),
                    seq([call(WaitForTrustedHeight {
                        chain_id: counterparty_chain_id,
                        ibc_spec_id: IbcUnion::ID,
                        client_id: RawClientId::new(beacon_kit_client_state.l1_client_id),
                        height: update_to,
                        finalized: true,
                    })]),
                    data(OrderedHeaders {
                        headers: vec![(
                            DecodedHeaderMeta { height: update_to },
                            into_value(header),
                        )],
                    }),
                ]))
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn callback(
        &self,
        _: &Extensions,
        callback: Never,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match callback {}
    }
}
