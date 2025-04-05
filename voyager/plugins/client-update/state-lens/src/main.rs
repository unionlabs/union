use std::{collections::VecDeque, fmt::Debug};

use call::FetchUpdateAfterL1Update;
use ibc_union_spec::{
    path::{ClientStatePath, ConsensusStatePath},
    ClientId, IbcUnion,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use state_lens_light_client_types::Header;
use tracing::{debug, info, instrument};
use unionlabs::ibc::core::client::height::Height;
use voyager_message::{
    call::{Call, FetchUpdateHeaders, WaitForTrustedHeight},
    callback::AggregateSubmitTxFromOrderedHeaders,
    data::{Data, DecodedHeaderMeta, OrderedHeaders},
    into_value,
    module::{PluginInfo, PluginServer},
    primitives::{ChainId, ClientType, IbcSpec, QueryHeight},
    rpc::ProofType,
    DefaultCmd, ExtensionsExt, Plugin, PluginMessage, RawClientId, VoyagerClient, VoyagerMessage,
    FATAL_JSONRPC_ERROR_CODE, MISSING_STATE_ERROR_CODE,
};
use voyager_vm::{call, conc, data, pass::PassResult, promise, seq, BoxDynError, Op, Visit};

use crate::{
    call::{FetchUpdate, ModuleCall},
    callback::ModuleCallback,
};

pub mod call;
pub mod callback;

pub type StateLensClientState =
    state_lens_light_client_types::ClientState<serde_json::Map<String, serde_json::Value>>;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    /// The state lens client type (state-lens/*/*) that this plugin creates updates for.
    pub state_lens_client_type: ClientType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub state_lens_client_type: ClientType,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        Ok(Self {
            state_lens_client_type: config.state_lens_client_type,
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.state_lens_client_type),
            // inlined UpdateHook::filter, since we only care about client_type here
            interest_filter: format!(
                r#"[.. | ."@type"? == "fetch_update_headers" and ."@value".client_type == "{}"] | any"#,
                config.state_lens_client_type
            ),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

fn plugin_name(state_lens_client_type: &ClientType) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{state_lens_client_type}")
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.state_lens_client_type)
    }

    #[instrument(
        skip_all,
        fields(
            %chain_id,
            %counterparty_chain_id,
            %client_id,
            %update_from,
            %update_to,
        )
    )]
    async fn fetch_update(
        &self,
        voyager_client: &VoyagerClient,
        chain_id: ChainId,
        counterparty_chain_id: ChainId,
        client_id: ClientId,
        update_from: Height,
        update_to: Height,
    ) -> RpcResult<Op<VoyagerMessage>> {
        let counterparty_latest_height = voyager_client
            .query_latest_height(counterparty_chain_id.clone(), false)
            .await?;

        // state lens client running on the counterparty, tracking self.chain_id
        let raw_state_lens_client_state = voyager_client
            .query_ibc_state(
                counterparty_chain_id.clone(),
                counterparty_latest_height,
                ClientStatePath { client_id },
            )
            .await?;

        debug!(?raw_state_lens_client_state);

        let state_lens_client_state_info = voyager_client
            .client_info::<IbcUnion>(counterparty_chain_id.clone(), client_id)
            .await?;

        debug!(?state_lens_client_state_info);

        let state_lens_client_state = voyager_client
            .decode_client_state::<IbcUnion, StateLensClientState>(
                state_lens_client_state_info.client_type.clone(),
                state_lens_client_state_info.ibc_interface,
                raw_state_lens_client_state,
            )
            .await?;

        debug!(?state_lens_client_state);

        info!(
            l2_chain_id = state_lens_client_state.l2_chain_id,
            l1_client_id = state_lens_client_state.l1_client_id.raw(),
            l2_client_id = state_lens_client_state.l2_client_id.raw(),
            l2_latest_height = state_lens_client_state.l2_latest_height,
            "state lens client state"
        );

        let l1_client_state_meta = voyager_client
            .client_state_meta::<IbcUnion>(
                counterparty_chain_id.clone(),
                QueryHeight::Latest,
                state_lens_client_state.l1_client_id,
            )
            .await?;

        debug!(?l1_client_state_meta);

        let l1_latest_height = voyager_client
            .query_latest_height(l1_client_state_meta.counterparty_chain_id.clone(), true)
            .await?;

        debug!(%l1_latest_height);

        // assert_eq!(state_lens_client_state.l2_chain_id, self.chain_id.as_str());

        let l2_consensus_state = voyager_client
            .maybe_query_ibc_state(
                l1_client_state_meta.counterparty_chain_id.clone(),
                QueryHeight::Specific(l1_latest_height),
                ConsensusStatePath {
                    client_id: state_lens_client_state.l2_client_id,
                    height: update_to.height(),
                },
            )
            .await?
            .state;

        debug!(?l2_consensus_state);

        let continuation = call(PluginMessage::new(
            self.plugin_name(),
            ModuleCall::from(FetchUpdateAfterL1Update {
                counterparty_chain_id: counterparty_chain_id.clone(),
                state_lens_client_state: state_lens_client_state.clone(),
                client_id,
                update_from,
                update_to,
            }),
        ));

        let l2_client_type = voyager_client
            .client_info::<IbcUnion>(
                l1_client_state_meta.counterparty_chain_id.clone(),
                state_lens_client_state.l2_client_id,
            )
            .await?
            .client_type;

        // if the L2 consensus state exists on the L1, we don't have to update the L2 on the L1.
        match l2_consensus_state {
            Some(_) => {
                info!("consensus state already exists");
                Ok(continuation)
            }
            None => {
                info!(
                    "consensus state does not exist, queuing update for l2 client {} (client type {}, tracking {}) on {}",
                    state_lens_client_state.l2_client_id,
                    l2_client_type,
                    chain_id,
                    l1_client_state_meta.counterparty_chain_id,
                );
                Ok(conc([
                    // update the L2 client on L1 and then dispatch the continuation
                    promise(
                        [call(FetchUpdateHeaders {
                            client_type: l2_client_type,
                            chain_id: chain_id.clone(),
                            counterparty_chain_id: l1_client_state_meta
                                .counterparty_chain_id
                                .clone(),
                            client_id: RawClientId::new(state_lens_client_state.l2_client_id),
                            update_from,
                            update_to,
                        })],
                        [],
                        AggregateSubmitTxFromOrderedHeaders {
                            ibc_spec_id: IbcUnion::ID,
                            chain_id: l1_client_state_meta.counterparty_chain_id.clone(),
                            client_id: RawClientId::new(state_lens_client_state.l2_client_id),
                        },
                    ),
                    seq([
                        call(WaitForTrustedHeight {
                            chain_id: l1_client_state_meta.counterparty_chain_id.clone(),
                            ibc_spec_id: IbcUnion::ID,
                            client_id: RawClientId::new(state_lens_client_state.l2_client_id),
                            height: update_to,
                            finalized: true,
                        }),
                        continuation,
                    ]),
                ]))
            }
        }
    }
}

#[async_trait]
impl PluginServer<ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all, fields())]
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
                    struct StateLensClientUpdateHook<'a>(&'a Module);

                    impl Visit<VoyagerMessage> for StateLensClientUpdateHook<'_> {
                        fn visit_call(&mut self, c: &mut Call) {
                            match c {
                                Call::FetchUpdateHeaders(fetch)
                                    if fetch.client_type == self.0.state_lens_client_type =>
                                {
                                    info!(
                                        %fetch.client_type,
                                        %fetch.chain_id,
                                        %fetch.counterparty_chain_id,
                                        %fetch.client_id,
                                        %fetch.update_from,
                                        %fetch.update_to,
                                        "hooking for state lens update (`{}` on `{}` tracking `{}`, id {}, {} to {})",
                                        fetch.client_type,
                                        fetch.chain_id,
                                        fetch.counterparty_chain_id,
                                        fetch.client_id,
                                        fetch.update_from,
                                        fetch.update_to
                                    );

                                    *c = Call::Plugin(PluginMessage::new(
                                        self.0.plugin_name(),
                                        ModuleCall::from(FetchUpdate {
                                            chain_id: fetch.chain_id.clone(),
                                            counterparty_chain_id: fetch
                                                .counterparty_chain_id
                                                .clone(),
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
                                    ));
                                }
                                _ => {}
                            }
                        }
                    }

                    StateLensClientUpdateHook(self).visit_op(&mut op);

                    op
                })
                .enumerate()
                .map(|(i, op)| (vec![i], op))
                .collect(),
        })
    }

    #[instrument(skip_all, fields())]
    async fn call(&self, ext: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::FetchUpdate(FetchUpdate {
                chain_id,
                counterparty_chain_id,
                client_id,
                update_from,
                update_to,
            }) => {
                self.fetch_update(
                    ext.try_get::<VoyagerClient>()?,
                    chain_id,
                    counterparty_chain_id,
                    client_id,
                    update_from,
                    update_to,
                )
                .await
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
                let l1_client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        counterparty_chain_id.clone(),
                        QueryHeight::Latest,
                        state_lens_client_state.l1_client_id,
                    )
                    .await?;
                debug!(?l1_client_state_meta);

                // the client on the counterparty that is tracking the L1
                let l1_client_info = voyager_client
                    .client_info::<IbcUnion>(
                        counterparty_chain_id.clone(),
                        state_lens_client_state.l1_client_id,
                    )
                    .await?;
                debug!(?l1_client_info);

                // the client on the L1 that is tracking the L2
                let l2_client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        l1_client_state_meta.counterparty_chain_id.clone(),
                        QueryHeight::Latest,
                        state_lens_client_state.l2_client_id,
                    )
                    .await?;
                debug!(?l1_client_state_meta);

                let l1_latest_height = voyager_client
                    .query_latest_height(l1_client_state_meta.counterparty_chain_id.clone(), false)
                    .await?;
                debug!(
                    "l1 ({}) latest height {}",
                    l1_client_state_meta.counterparty_chain_id, l1_latest_height
                );

                // client state meta of the state lens client on the counterparty
                let state_lens_client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        counterparty_chain_id.clone(),
                        QueryHeight::Latest,
                        client_id,
                    )
                    .await?;
                debug!(?state_lens_client_state_meta);

                let state_lens_client_info = voyager_client
                    .client_info::<IbcUnion>(counterparty_chain_id.clone(), client_id)
                    .await?;
                debug!(?state_lens_client_info);

                // ensure that the l2 client on the l1 has been updated to at least update_to
                if l2_client_state_meta.counterparty_height < update_to {
                    return Err(ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!(
                            "l2_client_state_meta.counterparty_height is {} but update_to \
                            request is {update_to}",
                            l2_client_state_meta.counterparty_height
                        ),
                        None::<()>,
                    ));
                }

                let update_to = l2_client_state_meta.counterparty_height;

                let l2_consensus_state_path = ConsensusStatePath {
                    client_id: state_lens_client_state.l2_client_id,
                    height: update_to.height(),
                };

                let l2_consensus_state = voyager_client
                    .query_ibc_state(
                        l1_client_state_meta.counterparty_chain_id.clone(),
                        l1_latest_height,
                        l2_consensus_state_path.clone(),
                    )
                    .await?;

                debug!(?l2_consensus_state);

                let l2_consensus_state_proof = voyager_client
                    .query_ibc_proof(
                        l1_client_state_meta.counterparty_chain_id.clone(),
                        QueryHeight::Specific(l1_latest_height),
                        l2_consensus_state_path,
                    )
                    .await?;

                if l2_consensus_state_proof.proof_type != ProofType::Membership {
                    return Err(ErrorObject::owned(
                        MISSING_STATE_ERROR_CODE,
                        "proof of the l2 consensus state must be a membership proof",
                        None::<()>,
                    ));
                }

                debug!(?l2_consensus_state_proof);

                let l2_consensus_state_proof_bytes = voyager_client
                    .encode_proof::<IbcUnion>(
                        l1_client_info.client_type.clone(),
                        state_lens_client_info.ibc_interface,
                        l2_consensus_state_proof.proof,
                    )
                    .await?;

                // dispatch an update for the L1 on the destination, then dispatch the L2 update on the destination
                Ok(conc([
                    promise(
                        [call(FetchUpdateHeaders {
                            client_type: l1_client_info.client_type,
                            chain_id: l1_client_state_meta.counterparty_chain_id.clone(),
                            counterparty_chain_id: counterparty_chain_id.clone(),
                            client_id: RawClientId::new(state_lens_client_state.l1_client_id),
                            update_from: l1_client_state_meta.counterparty_height,
                            update_to: l1_latest_height,
                        })],
                        [],
                        AggregateSubmitTxFromOrderedHeaders {
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

    #[instrument(skip_all, fields())]
    async fn callback(
        &self,
        _: &Extensions,
        callback: ModuleCallback,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match callback {}
    }
}

#[test]
fn json() {
    serde_json::from_str::<StateLensClientState>(r#"{"l2_chain_id":"elgafar-1","l1_client_id":3,"l2_client_id":14,"l2_latest_height":14268576,"contract_address":"0x83cd1201e1dfd6605349a902146daf50f2b8f254b152b96b882e3dfc47c583bc"}"#).unwrap();
}
