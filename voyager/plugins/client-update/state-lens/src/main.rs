use std::{collections::VecDeque, fmt::Debug};

use call::FetchUpdateAfterL1Update;
use ibc_union_spec::{ClientStatePath, ConsensusStatePath, IbcUnion};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use state_lens_light_client_types::Header;
use tracing::{debug, instrument};
use unionlabs::ErrorReporter;
use voyager_message::{
    call::{Call, FetchUpdateHeaders, WaitForTrustedHeight},
    callback::AggregateMsgUpdateClientsFromOrderedHeaders,
    core::{ChainId, ClientType, IbcSpec, QueryHeight},
    data::{Data, DecodedHeaderMeta, OrderedHeaders},
    hook::UpdateHook,
    into_value,
    module::{PluginInfo, PluginServer},
    rpc::missing_state,
    DefaultCmd, ExtensionsExt, Plugin, PluginMessage, RawClientId, VoyagerClient, VoyagerMessage,
    FATAL_JSONRPC_ERROR_CODE,
};
use voyager_vm::{call, conc, data, pass::PassResult, promise, seq, BoxDynError, Op, Visit};

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

/// Representation of the client state of a state lens client.
///
/// For a state lens client A->B->C, where the state lens is running on A and tracking C, the terminology is as follows:
///
/// - B is L1
/// - C is L2
///
/// where C "settles" on B with the client `self.l2_client_id`, and B "settles" on A with `self.l1_client_id`.
// NOTE: Purposely DOESN'T use deny_unknown_fields
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateLensClientState /* <Extra> */ {
    /// L2 chain ID. This is the same as the ID of the chain being tracked by `self.l2_client_id`.
    ///
    /// ("C")
    pub l2_chain_id: ChainId,

    /// L1 client ID. This is the ID of the L1 client running on A that is used to check the L2 inclusion proof against.
    ///
    /// ("B" on "A")
    pub l1_client_id: u32,

    /// L2 client ID. This is the ID of the L2 client running on B (L1) tracking the C (L2).
    ///
    /// ("C" on "B")
    pub l2_client_id: u32,

    /// L2 latest height
    pub l2_latest_height: u64,
    // #[serde(flatten)]
    // pub extra: Extra,
}

#[derive(Debug, Clone)]
pub struct Module {
    /// The ID of the chain this plugin creates updates for.
    pub chain_id: ChainId,
    /// The state lens client type (state-lens/*/*) that this plugin creates updates for.
    pub state_lens_client_type: ClientType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,
    pub state_lens_client_type: ClientType,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        Ok(Self {
            chain_id: config.chain_id,
            state_lens_client_type: config.state_lens_client_type,
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id, &config.state_lens_client_type),
            interest_filter: UpdateHook::filter(&config.chain_id, &config.state_lens_client_type),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

fn plugin_name(chain_id: &ChainId, state_lens_client_type: &ClientType) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{chain_id}/{state_lens_client_type}")
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id, &self.state_lens_client_type)
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
                    UpdateHook::new(&self.chain_id, &self.state_lens_client_type, |fetch| {
                        Call::Plugin(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(FetchUpdate {
                                counterparty_chain_id: fetch.counterparty_chain_id.clone(),
                                client_id: fetch
                                    .client_id
                                    .clone()
                                    .decode_spec::<IbcUnion>()
                                    .unwrap(),
                                // .map_err(|e| {
                                //     ErrorObject::owned(
                                //         FATAL_JSONRPC_ERROR_CODE,
                                //         format!("invalid client id `{}`", fetch.client_id),
                                //         None::<()>,
                                //     )
                                // })?,
                                update_from: fetch.update_from,
                                update_to: fetch.update_to,
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
    async fn call(&self, ext: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::FetchUpdate(FetchUpdate {
                counterparty_chain_id,
                client_id,
                update_from,
                update_to,
            }) => {
                let voyager_client = ext.try_get::<VoyagerClient>()?;

                // state lens client running on the counterparty, tracking self.chain_id
                let raw_state_lens_client_state = voyager_client
                    .query_ibc_state(
                        counterparty_chain_id.clone(),
                        QueryHeight::Latest,
                        ClientStatePath { client_id },
                    )
                    .await?
                    .state
                    .ok_or_else(missing_state(
                        "state lens client state doesn't exist?",
                        None,
                    ))?;

                debug!(?raw_state_lens_client_state);

                let state_lens_client_state_info = voyager_client
                    .client_info::<IbcUnion>(counterparty_chain_id.clone(), client_id)
                    .await?;

                debug!(?state_lens_client_state_info);

                let state_lens_client_state_json = voyager_client
                    .decode_client_state::<IbcUnion>(
                        state_lens_client_state_info.client_type.clone(),
                        state_lens_client_state_info.ibc_interface,
                        raw_state_lens_client_state,
                    )
                    .await?;

                debug!(%state_lens_client_state_json);

                let state_lens_client_state =
                    serde_json::from_value::<StateLensClientState>(state_lens_client_state_json)
                        .map_err(|e| {
                            ErrorObject::owned(
                                FATAL_JSONRPC_ERROR_CODE,
                                format!(
                                    "unable to deserialize state lens client state: {}",
                                    ErrorReporter(e)
                                ),
                                None::<()>,
                            )
                        })?;

                debug!(?state_lens_client_state);

                let l1_client_meta = voyager_client
                    .client_meta::<IbcUnion>(
                        counterparty_chain_id.clone(),
                        QueryHeight::Latest,
                        state_lens_client_state.l1_client_id,
                    )
                    .await?;

                debug!(?l1_client_meta);

                let l1_latest_height = voyager_client
                    .query_latest_height(state_lens_client_state.l2_chain_id.clone(), true)
                    .await?;

                debug!(%l1_latest_height);

                assert_eq!(state_lens_client_state.l2_chain_id, self.chain_id);

                let l2_consensus_state_proof = voyager_client
                    .query_ibc_state(
                        l1_client_meta.chain_id.clone(),
                        QueryHeight::Specific(l1_latest_height),
                        ConsensusStatePath {
                            client_id: state_lens_client_state.l2_client_id,
                            height: update_to.height(),
                        },
                    )
                    .await?
                    .state;

                debug!(%l1_latest_height);

                let continuation = call(PluginMessage::new(
                    self.plugin_name(),
                    ModuleCall::from(FetchUpdateAfterL1Update {
                        counterparty_chain_id,
                        state_lens_client_state: state_lens_client_state.clone(),
                        client_id,
                        update_from,
                        update_to,
                    }),
                ));

                let l2_client_type = voyager_client
                    .client_info::<IbcUnion>(
                        l1_client_meta.chain_id.clone(),
                        state_lens_client_state.l2_client_id,
                    )
                    .await?
                    .client_type;

                // if the L2 consensus state exists on the L1, we don't have to update the L2 on the L1.
                match l2_consensus_state_proof {
                    Some(_) => Ok(continuation),
                    None => Ok(conc([
                        // update the L2 client on L1 and then dispatch the continuation
                        promise(
                            [call(FetchUpdateHeaders {
                                client_type: l2_client_type,
                                chain_id: self.chain_id.clone(),
                                counterparty_chain_id: l1_client_meta.chain_id.clone(),
                                client_id: RawClientId::new(state_lens_client_state.l2_client_id),
                                update_from,
                                update_to,
                            })],
                            [],
                            AggregateMsgUpdateClientsFromOrderedHeaders {
                                ibc_spec_id: IbcUnion::ID,
                                chain_id: l1_client_meta.chain_id.clone(),
                                client_id: RawClientId::new(state_lens_client_state.l2_client_id),
                            },
                        ),
                        seq([
                            call(WaitForTrustedHeight {
                                chain_id: l1_client_meta.chain_id.clone(),
                                ibc_spec_id: IbcUnion::ID,
                                client_id: RawClientId::new(state_lens_client_state.l2_client_id),
                                height: update_to,
                                finalized: true,
                            }),
                            continuation,
                        ]),
                    ])),
                }
            }
            ModuleCall::FetchUpdateAfterL1Update(FetchUpdateAfterL1Update {
                counterparty_chain_id,
                state_lens_client_state,
                client_id,
                update_from: _,
                update_to,
            }) => {
                let voyager_client = ext.try_get::<VoyagerClient>()?;

                // the client on the counterparty that is tracking the L1
                let l1_client_meta = voyager_client
                    .client_meta::<IbcUnion>(
                        counterparty_chain_id.clone(),
                        QueryHeight::Latest,
                        state_lens_client_state.l1_client_id,
                    )
                    .await?;
                debug!(?l1_client_meta);

                // the client on the counterparty that is tracking the L1
                let l1_client_info = voyager_client
                    .client_info::<IbcUnion>(
                        counterparty_chain_id.clone(),
                        state_lens_client_state.l1_client_id,
                    )
                    .await?;
                debug!(?l1_client_info);

                // the client on the L1 that is tracking the L2
                let l2_client_meta = voyager_client
                    .client_meta::<IbcUnion>(
                        l1_client_meta.chain_id.clone(),
                        QueryHeight::Latest,
                        state_lens_client_state.l2_client_id,
                    )
                    .await?;
                debug!(?l1_client_meta);

                let l1_latest_height = voyager_client
                    .query_latest_height(l1_client_meta.chain_id.clone(), false)
                    .await?;
                debug!(
                    "l1 ({}) latest height {}",
                    l1_client_meta.chain_id, l1_latest_height
                );

                // client meta of the state lens client on the counterparty
                let state_lens_client_meta = voyager_client
                    .client_meta::<IbcUnion>(
                        counterparty_chain_id.clone(),
                        QueryHeight::Latest,
                        client_id,
                    )
                    .await?;
                debug!(?state_lens_client_meta);

                let state_lens_client_info = voyager_client
                    .client_info::<IbcUnion>(counterparty_chain_id.clone(), client_id)
                    .await?;
                debug!(?state_lens_client_info);

                // ensure that the l2 client on the l1 has been updated to at least update_to
                if l2_client_meta.counterparty_height < update_to {
                    return Err(ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!(
                            "l2_client_meta.counterparty_height is {} but update_to \
                            request is {update_to}",
                            l2_client_meta.counterparty_height
                        ),
                        None::<()>,
                    ));
                }

                let update_to = l2_client_meta.counterparty_height;

                let l2_consensus_state_path = ConsensusStatePath {
                    client_id: state_lens_client_state.l2_client_id,
                    height: update_to.height(),
                };

                let l2_consensus_state = voyager_client
                    .query_ibc_state(
                        l1_client_meta.chain_id.clone(),
                        QueryHeight::Specific(l1_latest_height),
                        l2_consensus_state_path.clone(),
                    )
                    .await?
                    .state
                    .ok_or_else(missing_state("l2 consensus on l1 doesn't exist", None))?;

                debug!(?l2_consensus_state);

                let l2_consensus_state_proof = voyager_client
                    .query_ibc_proof(
                        l1_client_meta.chain_id.clone(),
                        QueryHeight::Specific(l1_latest_height),
                        l2_consensus_state_path,
                    )
                    .await?
                    .proof;
                debug!(?l2_consensus_state_proof);

                let l2_consensus_state_proof_bytes = voyager_client
                    .encode_proof::<IbcUnion>(
                        l1_client_info.client_type.clone(),
                        state_lens_client_info.ibc_interface,
                        l2_consensus_state_proof,
                    )
                    .await?;

                // dispatch an update for the L1 on the destination, then dispatch the L2 update on the destination
                Ok(conc([
                    promise(
                        [call(FetchUpdateHeaders {
                            client_type: l1_client_info.client_type,
                            chain_id: l1_client_meta.chain_id.clone(),
                            counterparty_chain_id: counterparty_chain_id.clone(),
                            client_id: RawClientId::new(state_lens_client_state.l1_client_id),
                            update_from: l1_client_meta.counterparty_height,
                            update_to: l1_latest_height,
                        })],
                        [],
                        AggregateMsgUpdateClientsFromOrderedHeaders {
                            ibc_spec_id: IbcUnion::ID,
                            chain_id: counterparty_chain_id.clone(),
                            client_id: RawClientId::new(state_lens_client_state.l1_client_id),
                        },
                    ),
                    seq([
                        call(WaitForTrustedHeight {
                            chain_id: counterparty_chain_id,
                            ibc_spec_id: IbcUnion::ID,
                            client_id: RawClientId::new(state_lens_client_state.l1_client_id),
                            height: l1_latest_height,
                            finalized: false,
                        }),
                        data(OrderedHeaders {
                            headers: vec![(
                                DecodedHeaderMeta { height: update_to },
                                into_value(Header {
                                    l1_height: l1_latest_height,
                                    l2_height: update_to,
                                    l2_consensus_state_proof: l2_consensus_state_proof_bytes,
                                    l2_consensus_state,
                                }),
                            )],
                        }),
                    ]),
                ]))
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn callback(
        &self,
        _: &Extensions,
        callback: ModuleCallback,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match callback {}
    }
}
