use std::{collections::VecDeque, fmt::Debug};

use call::FetchUpdateAfterL1Update;
use ibc_union_spec::{ConsensusStatePath, IbcUnion};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use serde::{Deserialize, Serialize};
use state_lens_light_client_types::Header;
use tracing::{debug, instrument};
use unionlabs::ibc::core::commitment::merkle_proof::MerkleProof;
use voyager_message::{
    call::{Call, FetchUpdateHeaders, WaitForTrustedHeight},
    callback::AggregateMsgUpdateClientsFromOrderedHeaders,
    core::{ChainId, ClientType, IbcSpec, QueryHeight},
    data::{Data, DecodedHeaderMeta, OrderedHeaders},
    hook::UpdateHook,
    into_value,
    module::{PluginInfo, PluginServer},
    DefaultCmd, ExtensionsExt, Plugin, PluginMessage, RawClientId, VoyagerClient, VoyagerMessage,
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

#[derive(Debug, Clone)]
pub struct Module {
    pub l0_client_id: u32,
    pub l1_client_id: u32,
    pub l1_chain_id: ChainId,
    pub l2_chain_id: ChainId,
    pub l2_client_type: ClientType,
    pub state_lens_client_type: ClientType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub l0_client_id: u32,
    pub l1_client_id: u32,
    pub l1_chain_id: ChainId,
    pub l2_chain_id: ChainId,
    pub l2_client_type: ClientType,
    pub state_lens_client_type: ClientType,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        Ok(Self {
            l0_client_id: config.l0_client_id,
            l1_client_id: config.l1_client_id,
            l1_chain_id: config.l1_chain_id,
            l2_chain_id: config.l2_chain_id,
            l2_client_type: config.l2_client_type,
            state_lens_client_type: config.state_lens_client_type,
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(
                config.l0_client_id,
                config.l1_client_id,
                &config.l2_chain_id,
            ),
            interest_filter: UpdateHook::filter(
                &config.l2_chain_id,
                &config.state_lens_client_type,
            ),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

fn plugin_name(l0_client_id: u32, l1_client_id: u32, l2_chain_id: &ChainId) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{l0_client_id}/{l1_client_id}/{l2_chain_id}")
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(self.l0_client_id, self.l1_client_id, &self.l2_chain_id)
    }
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
                    UpdateHook::new(&self.l2_chain_id, &self.state_lens_client_type, |fetch| {
                        Call::Plugin(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(FetchUpdate {
                                counterparty_chain_id: fetch.counterparty_chain_id.clone(),
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

    #[instrument(skip_all, fields(chain_id = %self.l2_chain_id))]
    async fn call(&self, ext: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::FetchUpdate(FetchUpdate {
                counterparty_chain_id,
                update_from,
                update_to,
            }) => {
                let voy_client = ext.try_get::<VoyagerClient>()?;
                let l1_latest_height = voy_client
                    .query_latest_height(self.l1_chain_id.clone(), true)
                    .await?;
                let l2_consensus_state_proof = serde_json::from_value::<MerkleProof>(
                    voy_client
                        .query_ibc_proof(
                            self.l1_chain_id.clone(),
                            QueryHeight::Specific(l1_latest_height),
                            ConsensusStatePath {
                                client_id: self.l1_client_id,
                                height: update_to.height(),
                            },
                        )
                        .await
                        .expect("big trouble")
                        .proof,
                )
                .expect("impossible");
                let l2_merkle_proof = unionlabs::union::ics23::merkle_proof::MerkleProof::try_from(
                    protos::ibc::core::commitment::v1::MerkleProof::from(l2_consensus_state_proof),
                )
                .expect("impossible");
                let continuation = call(PluginMessage::new(
                    self.plugin_name(),
                    ModuleCall::from(FetchUpdateAfterL1Update {
                        counterparty_chain_id,
                        update_from,
                        update_to,
                    }),
                ));
                // If the L2 consensus proof exists on the L1, we don't have to update the L2 on the L1.
                match l2_merkle_proof {
                    unionlabs::union::ics23::merkle_proof::MerkleProof::Membership(_, _) => {
                        Ok(continuation)
                    }
                    _ => Ok(conc([
                        // Update the L2 (eth) client on L1 (union) and then dispatch the continuation
                        promise(
                            [call(FetchUpdateHeaders {
                                client_type: self.l2_client_type.clone(),
                                chain_id: self.l2_chain_id.clone(),
                                counterparty_chain_id: self.l1_chain_id.clone(),
                                update_from,
                                update_to,
                            })],
                            [],
                            AggregateMsgUpdateClientsFromOrderedHeaders {
                                ibc_spec_id: IbcUnion::ID,
                                chain_id: self.l1_chain_id.clone(),
                                client_id: RawClientId::new(self.l1_client_id),
                            },
                        ),
                        seq([
                            call(WaitForTrustedHeight {
                                chain_id: self.l1_chain_id.clone(),
                                ibc_spec_id: IbcUnion::ID,
                                client_id: RawClientId::new(self.l1_client_id),
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
                ..
            }) => {
                let voy_client = ext.try_get::<VoyagerClient>()?;
                let l1_latest_height = voy_client
                    .query_latest_height(self.l1_chain_id.clone(), false)
                    .await?;
                debug!("l1 latest height {}", l1_latest_height);
                let l0_client_meta = voy_client
                    .client_meta::<IbcUnion>(
                        counterparty_chain_id.clone(),
                        QueryHeight::Latest,
                        self.l0_client_id,
                    )
                    .await?;
                let l1_client_meta = voy_client
                    .client_meta::<IbcUnion>(
                        self.l1_chain_id.clone(),
                        QueryHeight::Specific(l1_latest_height),
                        self.l1_client_id,
                    )
                    .await?;
                // The client has been updated to at least update_to
                let update_to = l1_client_meta.counterparty_height;
                debug!("l0 client meta {:?}", l0_client_meta);

                let l2_consensus_state_path = ConsensusStatePath {
                    client_id: self.l1_client_id,
                    height: update_to.height(),
                };

                let l2_consensus_state = voy_client
                    .query_ibc_state(
                        self.l1_chain_id.clone(),
                        QueryHeight::Specific(l1_latest_height),
                        l2_consensus_state_path.clone(),
                    )
                    .await?
                    .state;

                debug!("l2 consensus state {:?}", l2_consensus_state);

                let l2_consensus_state_proof = voy_client
                    .query_ibc_proof(
                        self.l1_chain_id.clone(),
                        QueryHeight::Specific(l1_latest_height),
                        l2_consensus_state_path,
                    )
                    .await
                    .expect("big trouble")
                    .proof;

                let state_lens_client = voy_client
                    .client_info::<IbcUnion>(counterparty_chain_id.clone(), self.l0_client_id)
                    .await?;

                debug!("l2 consensus state proof {:?}", l2_consensus_state_proof);

                let l2_consensus_state_proof_bytes = voy_client
                    .encode_proof::<IbcUnion>(
                        ClientType::new(ClientType::COMETBLS),
                        state_lens_client.ibc_interface,
                        l2_consensus_state_proof,
                    )
                    .await?;

                // Dispatch an update for the L1 on the destination, then dispatch the L2 update on the destination
                Ok(conc([
                    promise(
                        [call(FetchUpdateHeaders {
                            client_type: ClientType::new(ClientType::COMETBLS),
                            chain_id: self.l1_chain_id.clone(),
                            counterparty_chain_id: counterparty_chain_id.clone(),
                            update_from: l0_client_meta.counterparty_height,
                            update_to: l1_latest_height,
                        })],
                        [],
                        AggregateMsgUpdateClientsFromOrderedHeaders {
                            ibc_spec_id: IbcUnion::ID,
                            chain_id: counterparty_chain_id.clone(),
                            client_id: RawClientId::new(self.l0_client_id),
                        },
                    ),
                    seq([
                        call(WaitForTrustedHeight {
                            chain_id: counterparty_chain_id,
                            ibc_spec_id: IbcUnion::ID,
                            client_id: RawClientId::new(self.l0_client_id),
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
