use std::{collections::VecDeque, fmt::Debug, num::ParseIntError};

use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use beacon_api_types::{chain_spec::Mainnet, deneb};
use berachain_light_client_types::Header;
use ethereum_light_client_types::AccountProof;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use unionlabs::{
    berachain::LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX,
    encoding::{DecodeAs, Ssz},
    ibc::core::commitment::merkle_proof::MerkleProof,
    primitives::H160,
    ErrorReporter,
};
use voyager_message::{
    call::{Call, FetchUpdateHeaders, WaitForTrustedHeight},
    core::{ChainId, ClientType, IbcSpecId},
    data::{Data, DecodedHeaderMeta, OrderedHeaders},
    hook::UpdateHook,
    into_value,
    module::{PluginInfo, PluginServer},
    DefaultCmd, Plugin, PluginMessage, RawClientId, VoyagerMessage,
};
use voyager_vm::{call, conc, data, pass::PassResult, seq, BoxDynError, Op, Visit};

use crate::{
    call::{FetchUpdate, ModuleCall},
    callback::ModuleCallback,
};

pub mod call;
pub mod callback;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub l1_client_id: u32,
    pub l1_chain_id: ChainId,
    pub l2_chain_id: ChainId,
    pub ibc_handler_address: H160,
    pub eth_provider: DynProvider,
    pub cometbft_client: cometbft_rpc::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub l1_client_id: u32,
    pub l1_chain_id: ChainId,
    pub l2_chain_id: ChainId,
    pub ibc_handler_address: H160,
    pub comet_rpc_url: String,
    pub eth_rpc_url: String,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        let eth_provider =
            DynProvider::new(ProviderBuilder::new().connect(&config.eth_rpc_url).await?);

        let chain_id = ChainId::new(eth_provider.get_chain_id().await?.to_string());

        if chain_id != config.l2_chain_id {
            return Err(format!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.l2_chain_id, chain_id
            )
            .into());
        }

        let tm_client = cometbft_rpc::Client::new(config.comet_rpc_url).await?;

        Ok(Self {
            l1_client_id: config.l1_client_id,
            l2_chain_id: config.l2_chain_id,
            l1_chain_id: config.l1_chain_id,
            ibc_handler_address: config.ibc_handler_address,
            eth_provider,
            cometbft_client: tm_client,
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.l2_chain_id),
            interest_filter: UpdateHook::filter(
                &config.l2_chain_id,
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
        plugin_name(&self.l2_chain_id)
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
impl PluginServer<ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all, fields(chain_id = %self.l2_chain_id))]
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
                        &self.l2_chain_id,
                        &ClientType::new(ClientType::BEACON_KIT),
                        |fetch| {
                            Call::Plugin(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(FetchUpdate {
                                    counterparty_chain_id: fetch.counterparty_chain_id.clone(),
                                    update_from: fetch.update_from,
                                    update_to: fetch.update_to,
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

    #[instrument(skip_all, fields(chain_id = %self.l2_chain_id))]
    async fn call(&self, _: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::FetchUpdate(FetchUpdate {
                counterparty_chain_id,
                update_from,
                update_to,
            }) => {
                // NOTE: the implementation is simple because the
                // cometbft/beacon/execution heights are guaranteed to be the
                // same
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
                    call(FetchUpdateHeaders {
                        client_type: ClientType::new(ClientType::BEACON_KIT),
                        counterparty_chain_id: counterparty_chain_id.clone(),
                        chain_id: self.l1_chain_id.clone(),
                        client_id: RawClientId::new(self.l1_client_id),
                        update_from,
                        update_to,
                    }),
                    seq([call(WaitForTrustedHeight {
                        chain_id: counterparty_chain_id,
                        ibc_spec_id: IbcSpecId::new(IbcSpecId::UNION),
                        // TODO: abstract away the L1 client id and read it from
                        // the L2 client state (l2_client_id) on the
                        // `counterparty_chain_id`
                        client_id: RawClientId::new(self.l1_client_id),
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

    #[instrument(skip_all, fields(chain_id = %self.l2_chain_id))]
    async fn callback(
        &self,
        _: &Extensions,
        callback: ModuleCallback,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match callback {}
    }
}
