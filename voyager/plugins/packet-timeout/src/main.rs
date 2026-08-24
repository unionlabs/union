use std::collections::VecDeque;

use ibc_union_spec::{
    ClientId, IbcUnion,
    datagram::{Datagram, MsgCommitNonMembershipProof, MsgPacketTimeout},
    event::{FullEvent, PacketSend},
    path::{
        BatchPacketsPath, BatchReceiptsPath, COMMITMENT_MAGIC_ACK, ClientStatePath,
        ConsensusStatePath, NonMembershipProofPath,
    },
};
use jsonrpsee::{Extensions, core::async_trait};
use proof_lens_light_client_types::{
    ClientState as ProofLensClientState, ConsensusState as ProofLensConsensusState,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, info, instrument};
use unionlabs::{self, ibc::core::client::height::Height, never::Never};
use voyager_sdk::{
    DefaultCmd, ExtensionsExt, VoyagerClient, anyhow,
    message::{
        PluginMessage, VoyagerMessage,
        call::{FetchUpdateHeaders, SubmitTx, WaitForHeightRelative},
        callback::AggregateSubmitTxFromOrderedHeaders,
        data::{Data, IbcDatagram},
    },
    plugin::Plugin,
    primitives::{ChainId, ClientType, IbcSpec, QueryHeight},
    rpc::{PluginServer, RpcError, RpcErrorExt, RpcResult, types::PluginInfo},
    types::{ProofType, RawClientId},
    vm::{Op, call, defer, defer_relative, noop, pass::PassResult, promise, seq},
};

use crate::call::{
    CommitProofLensNonMembershipProof, MakeMsgTimeout, MakeMsgTimeoutFromTrustedHeight, ModuleCall,
    UpdateClientToHeightTimestamp, WaitForTimeoutOrReceipt,
};

pub mod call;

#[tokio::main]
async fn main() {
    Module::run().await
}

pub struct Module {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {}

/// Info needed to verify/commit proofs by proxy through a proof lens client's L1, resolved from
/// the lens client's `ClientState`. See `proof_lens_light_client_types::ClientState` for the
/// A->B->C (self->L1->L2) terminology.
struct ProofLensResolution {
    l1_chain_id: ChainId,
    l1_client_id: ClientId,
    l2_client_id: ClientId,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = Never;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> anyhow::Result<Self> {
        Ok(Module::new(config))
    }

    fn info(config: Self::Config) -> PluginInfo {
        let module = Module::new(config);

        PluginInfo {
            name: module.plugin_name(),
            interest_filter: format!(
                r#"
if ."@type" == "data"
    and ."@value"."@type" == "ibc_event"
    and ."@value"."@value".ibc_spec_id == "{ibc_union_id}"
    and ."@value"."@value".event."@type" == "packet_send"
then
    false # interest, but only copy
else
    null
end
"#,
                ibc_union_id = IbcUnion::ID,
            ),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

impl Module {
    fn plugin_name(&self) -> String {
        PLUGIN_NAME.to_string()
    }

    pub fn new(Config {}: Config) -> Self {
        Self {}
    }
}

#[async_trait]
impl PluginServer<ModuleCall, Never> for Module {
    #[instrument(skip_all, fields())]
    async fn run_pass(
        &self,
        _: &Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        let ready = msgs
            .into_iter()
            .enumerate()
            .map(|(idx, msg)| match msg {
                Op::Data(Data::IbcEvent(ref chain_event)) => match chain_event
                    .decode_event::<IbcUnion>()
                    .ok_or_else(|| {
                        RpcError::fatal_from_message("unexpected data message in queue").with_data(
                            json!({
                                "msg": msg.clone(),
                            }),
                        )
                    })?
                    .map_err(RpcError::fatal("unable to parse ibc datagram"))
                    .with_data(json!({
                        "msg": msg.clone(),
                    }))? {
                    FullEvent::PacketSend(packet_send) => Ok((
                        vec![idx],
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::WaitForTimeoutOrReceipt(WaitForTimeoutOrReceipt {
                                event: packet_send,
                                chain_id: chain_event.chain_id.clone(),
                                counterparty_chain_id: chain_event.counterparty_chain_id.clone(),
                            }),
                        )),
                    )),
                    datagram => Err(RpcError::fatal_from_message(format!(
                        "unexpected ibc datagram {}",
                        datagram.name()
                    ))
                    .with_data(json!({
                        "msg": msg,
                    }))),
                },
                _ => Err(
                    RpcError::fatal_from_message("unexpected message in queue").with_data(json!({
                        "msg": msg,
                    })),
                ),
            })
            .collect::<RpcResult<Vec<_>>>()?;

        Ok(PassResult {
            optimize_further: vec![],
            ready,
        })
    }

    #[instrument(skip_all, fields())]
    async fn call(&self, e: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        let voyager_client = e.voyager_client()?;

        match msg {
            ModuleCall::WaitForTimeoutOrReceipt(call) => {
                self.wait_for_timeout_or_receipt(voyager_client, call).await
            }
            ModuleCall::MakeMsgTimeout(MakeMsgTimeout {
                event,
                chain_id,
                counterparty_chain_id,
            }) => {
                let client_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        chain_id.clone(),
                        QueryHeight::Latest,
                        event.packet.source_channel.connection.client_id,
                    )
                    .await?;

                let timeout_sent = voyager_client
                    .query_ibc_state(
                        chain_id.clone(),
                        QueryHeight::Latest,
                        BatchPacketsPath::from_packet(&event.packet()),
                    )
                    .await?;

                if timeout_sent == COMMITMENT_MAGIC_ACK {
                    info!("packet timeout already received");

                    return Ok(noop());
                }

                let proof_unreceived = voyager_client
                    .query_ibc_proof(
                        counterparty_chain_id.clone(),
                        QueryHeight::Specific(client_meta.counterparty_height),
                        BatchReceiptsPath::from_packets(&[event.packet().clone()]),
                    )
                    .await?
                    .into_result()?;

                match proof_unreceived.proof_type {
                    ProofType::NonMembership => {
                        info!("packet not received yet");

                        Ok(seq([
                            // wait for the counterparty to finalize the timeout timestamp
                            call(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(UpdateClientToHeightTimestamp {
                                    chain_id: chain_id.clone(),
                                    counterparty_chain_id: counterparty_chain_id.clone(),
                                    client_id: event.packet.source_channel.connection.client_id,
                                    timestamp: event.packet.timeout_timestamp,
                                }),
                            )),
                            // build the timeout tx once the client is updated
                            call(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(MakeMsgTimeoutFromTrustedHeight {
                                    event,
                                    chain_id,
                                    counterparty_chain_id,
                                }),
                            )),
                        ]))
                    }
                    ProofType::Membership => {
                        info!(
                            packet_hash = %event.packet().hash(),
                            "packet already received",
                        );

                        Ok(noop())
                    }
                }
            }
            ModuleCall::UpdateClientToHeightTimestamp(UpdateClientToHeightTimestamp {
                chain_id,
                counterparty_chain_id,
                client_id,
                timestamp,
            }) => {
                let latest_timestamp = voyager_client
                    .query_latest_timestamp(chain_id.clone(), false)
                    .await?;

                if latest_timestamp < timestamp {
                    info!(
                        "timestamp not reached: latest_timestamp ({latest_timestamp}) < timestamp ({timestamp})"
                    );

                    Ok(seq([
                        // if the latest timestamp isn't high enough yet, wait a bit and try again
                        defer_relative(60),
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(UpdateClientToHeightTimestamp {
                                chain_id,
                                counterparty_chain_id,
                                client_id,
                                timestamp,
                            }),
                        )),
                    ]))
                } else {
                    info!(
                        "timestamp reached: latest_timestamp ({latest_timestamp}) >= timestamp ({timestamp})"
                    );

                    let latest_height = voyager_client
                        .query_latest_height(counterparty_chain_id.clone(), false)
                        .await?;

                    let client_info = voyager_client
                        .client_info::<IbcUnion>(chain_id.clone(), client_id)
                        .await?;

                    info!(
                        "updating client {client_id} on {counterparty_chain_id} to {latest_height}"
                    );

                    let client_meta = voyager_client
                        .client_state_meta::<IbcUnion>(
                            chain_id.clone(),
                            QueryHeight::Latest,
                            client_id,
                        )
                        .await?;

                    // update the counterparty client
                    Ok(promise(
                        [call(FetchUpdateHeaders {
                            client_type: client_info.client_type,
                            chain_id: counterparty_chain_id.clone(),
                            counterparty_chain_id: chain_id.clone(),
                            client_id: RawClientId::new(client_id),
                            update_from: client_meta.counterparty_height,
                            update_to: latest_height,
                        })],
                        [],
                        AggregateSubmitTxFromOrderedHeaders {
                            ibc_spec_id: IbcUnion::ID,
                            chain_id: chain_id.clone(),
                            client_id: RawClientId::new(client_id),
                        },
                    ))
                }
            }
            ModuleCall::MakeMsgTimeoutFromTrustedHeight(MakeMsgTimeoutFromTrustedHeight {
                event,
                chain_id,
                counterparty_chain_id,
            }) => {
                let client_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        chain_id.clone(),
                        QueryHeight::Latest,
                        event.packet.source_channel.connection.client_id,
                    )
                    .await?;

                let consensus_state_meta = voyager_client
                    .consensus_state_meta::<IbcUnion>(
                        chain_id.clone(),
                        QueryHeight::Latest,
                        event.packet.source_channel.connection.client_id,
                        client_meta.counterparty_height,
                    )
                    .await?;

                if consensus_state_meta.timestamp >= event.packet.timeout_timestamp {
                    info!(
                        "timestamp reached: consensus_state.timestamp ({}) >= event.timeout_timestamp ({})",
                        consensus_state_meta.timestamp, event.packet.timeout_timestamp,
                    );

                    let proof_unreceived = voyager_client
                        .query_ibc_proof(
                            counterparty_chain_id.clone(),
                            QueryHeight::Specific(client_meta.counterparty_height),
                            BatchReceiptsPath::from_packets(&[event.packet().clone()]),
                        )
                        .await?
                        .into_result()?;

                    match proof_unreceived.proof_type {
                        ProofType::NonMembership => {
                            match self
                                .resolve_proof_lens(
                                    voyager_client,
                                    &chain_id,
                                    event.packet.source_channel.connection.client_id,
                                )
                                .await?
                            {
                                // proof lens clients can't verify a raw counterparty proof
                                // directly - the fact that the packet hasn't been received must
                                // instead be committed onto the L1 first (see
                                // `commit_proof_lens_non_membership_proof`), which the lens client
                                // then verifies a proof of by proxy.
                                Some(lens) => {
                                    self.make_msg_timeout_via_proof_lens(
                                        voyager_client,
                                        event,
                                        chain_id,
                                        counterparty_chain_id,
                                        client_meta.counterparty_height,
                                        lens,
                                    )
                                    .await
                                }
                                None => {
                                    let client_info = voyager_client
                                        .client_info::<IbcUnion>(
                                            chain_id.clone(),
                                            event.packet.source_channel.connection.client_id,
                                        )
                                        .await?;

                                    let encoded_proof_commitment = voyager_client
                                        .encode_proof::<IbcUnion>(
                                            client_info.client_type,
                                            client_info.ibc_interface,
                                            proof_unreceived.proof,
                                        )
                                        .await?;

                                    Ok(call(SubmitTx {
                                        chain_id,
                                        datagrams: vec![IbcDatagram::new::<IbcUnion>(
                                            Datagram::from(MsgPacketTimeout {
                                                packet: event.packet(),
                                                proof: encoded_proof_commitment,
                                                proof_height: client_meta
                                                    .counterparty_height
                                                    .height(),
                                            }),
                                        )],
                                    }))
                                }
                            }
                        }
                        ProofType::Membership => {
                            info!(
                                packet_hash = %event.packet().hash(),
                                "packet already received",
                            );

                            Ok(noop())
                        }
                    }
                } else {
                    info!(
                        "timestamp not reached: consensus_state.timestamp ({}) < event.timeout_timestamp ({})",
                        consensus_state_meta.timestamp, event.packet.timeout_timestamp,
                    );

                    Ok(seq([
                        // if the latest trusted timestamp isn't high enough yet, wait a bit and try again
                        defer_relative(10),
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(UpdateClientToHeightTimestamp {
                                chain_id: chain_id.clone(),
                                counterparty_chain_id: counterparty_chain_id.clone(),
                                client_id: event.packet.source_channel.connection.client_id,
                                timestamp: event.packet.timeout_timestamp,
                            }),
                        )),
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(MakeMsgTimeoutFromTrustedHeight {
                                event,
                                chain_id,
                                counterparty_chain_id,
                            }),
                        )),
                    ]))
                }
            }
            ModuleCall::CommitProofLensNonMembershipProof(CommitProofLensNonMembershipProof {
                event,
                chain_id,
                counterparty_chain_id,
                counterparty_height,
            }) => {
                self.commit_proof_lens_non_membership_proof(
                    voyager_client,
                    event,
                    chain_id,
                    counterparty_chain_id,
                    counterparty_height,
                )
                .await
            }
        }
    }

    #[instrument(skip_all, fields())]
    async fn callback(
        &self,
        _: &Extensions,
        cb: Never,
        _datas: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }
}

impl Module {
    #[instrument(
        skip_all,
        fields(
            %chain_id,
            %counterparty_chain_id,
            packet_hash = %event.packet().hash()
        )
    )]
    async fn wait_for_timeout_or_receipt(
        &self,
        voyager_client: &VoyagerClient,
        WaitForTimeoutOrReceipt {
            event,
            chain_id,
            counterparty_chain_id,
        }: WaitForTimeoutOrReceipt,
    ) -> RpcResult<Op<VoyagerMessage>> {
        let receipt = voyager_client
            .maybe_query_ibc_state(
                counterparty_chain_id.clone(),
                QueryHeight::Latest,
                BatchReceiptsPath::from_packets(&[event.packet()]),
            )
            .await?;

        info!(
            height = receipt.height.height(),
            "counterparty latest height"
        );

        match receipt.state {
            Some(receipt) => {
                info!(%receipt, "packet received");
                Ok(noop())
            }
            None => {
                debug!("packet not received yet");

                if event.packet.timeout_timestamp.is_zero() {
                    Err(RpcError::fatal_from_message(
                        "packet has no timeout timestamp - should be impossible",
                    ))
                } else {
                    let counterparty_timestamp = voyager_client
                        .query_latest_timestamp(counterparty_chain_id.clone(), false)
                        .await?;

                    if event.packet.timeout_timestamp <= counterparty_timestamp {
                        info!(
                            "packet timed out (timestamp): {} <= {}",
                            event.packet.timeout_timestamp, counterparty_timestamp
                        );
                    }

                    Ok(seq([
                        // wait until the timestamp is hit
                        defer(event.packet.timeout_timestamp.as_secs()),
                        // then attempt to make the timeout message
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(MakeMsgTimeout {
                                event,
                                chain_id,
                                counterparty_chain_id,
                            }),
                        )),
                    ]))
                }
            }
        }
    }

    /// Determine whether `client_id` on `chain_id` is a proof lens client, resolving its L1/L2
    /// client ids if so.
    async fn resolve_proof_lens(
        &self,
        voyager_client: &VoyagerClient,
        chain_id: &ChainId,
        client_id: ClientId,
    ) -> RpcResult<Option<ProofLensResolution>> {
        let client_info = voyager_client
            .client_info::<IbcUnion>(chain_id.clone(), client_id)
            .await?;

        if client_info.client_type.as_str() != ClientType::PROOF_LENS {
            return Ok(None);
        }

        let proof_lens_client_state = voyager_client
            .decode_client_state::<IbcUnion, ProofLensClientState>(
                client_info.client_type.clone(),
                client_info.ibc_interface.clone(),
                voyager_client
                    .query_ibc_state(
                        chain_id.clone(),
                        QueryHeight::Latest,
                        ClientStatePath { client_id },
                    )
                    .await?,
            )
            .await?;

        let l1_chain_id = voyager_client
            .client_state_meta::<IbcUnion>(
                chain_id.clone(),
                QueryHeight::Latest,
                proof_lens_client_state.l1_client_id,
            )
            .await?
            .counterparty_chain_id;

        Ok(Some(ProofLensResolution {
            l1_chain_id,
            l1_client_id: proof_lens_client_state.l1_client_id,
            l2_client_id: proof_lens_client_state.l2_client_id,
        }))
    }

    /// Build the timeout message for a packet whose source client is a proof lens client.
    ///
    /// Proof lens clients don't have their own proof format (`client/proof-lens`'s
    /// `encode_proof`/`decode_proof` always error) - instead, the non-membership of the packet
    /// must first be committed onto the client's L1 (see
    /// `commit_proof_lens_non_membership_proof`), which the lens client then verifies a proof of
    /// by proxy.
    async fn make_msg_timeout_via_proof_lens(
        &self,
        voyager_client: &VoyagerClient,
        event: PacketSend,
        chain_id: ChainId,
        counterparty_chain_id: ChainId,
        counterparty_height: Height,
        lens: ProofLensResolution,
    ) -> RpcResult<Op<VoyagerMessage>> {
        let client_id = event.packet.source_channel.connection.client_id;

        // `verifyNonMembership` on the lens client reads the L1 height to check out of the lens
        // client's own consensus state at `counterparty_height` (not `counterparty_height`
        // itself, which is the L2/counterparty height) - so the proof we submit must be queried
        // against that same L1 height, not the L2 height.
        let lens_client_info = voyager_client
            .client_info::<IbcUnion>(chain_id.clone(), client_id)
            .await?;

        let proof_lens_consensus_state = voyager_client
            .decode_consensus_state::<IbcUnion, ProofLensConsensusState>(
                lens_client_info.client_type,
                lens_client_info.ibc_interface,
                voyager_client
                    .query_ibc_state(
                        chain_id.clone(),
                        QueryHeight::Latest,
                        ConsensusStatePath {
                            client_id,
                            height: counterparty_height.height(),
                        },
                    )
                    .await?,
            )
            .await?;

        let l1_proof_height = Height::new(proof_lens_consensus_state.l1_height);

        let receipt_path_key = BatchReceiptsPath::from_packets(&[event.packet()]).key();

        let commitment_proof = voyager_client
            .query_ibc_proof(
                lens.l1_chain_id.clone(),
                QueryHeight::Specific(l1_proof_height),
                NonMembershipProofPath {
                    client_id: lens.l2_client_id,
                    proof_height: counterparty_height.height(),
                    path: receipt_path_key.into(),
                },
            )
            .await?
            .into_result()?;

        match commitment_proof.proof_type {
            ProofType::Membership => {
                let l1_client_info = voyager_client
                    .client_info::<IbcUnion>(chain_id.clone(), lens.l1_client_id)
                    .await?;

                let encoded_proof_commitment = voyager_client
                    .encode_proof::<IbcUnion>(
                        l1_client_info.client_type,
                        l1_client_info.ibc_interface,
                        commitment_proof.proof,
                    )
                    .await?;

                Ok(call(SubmitTx {
                    chain_id,
                    datagrams: vec![IbcDatagram::new::<IbcUnion>(Datagram::from(
                        MsgPacketTimeout {
                            packet: event.packet(),
                            proof: encoded_proof_commitment,
                            proof_height: counterparty_height.height(),
                        },
                    ))],
                }))
            }
            ProofType::NonMembership => {
                info!("proof lens non-membership commitment not yet posted on L1, committing");

                Ok(call(PluginMessage::new(
                    self.plugin_name(),
                    ModuleCall::from(CommitProofLensNonMembershipProof {
                        event,
                        chain_id,
                        counterparty_chain_id,
                        counterparty_height,
                    }),
                )))
            }
        }
    }

    /// Commit a non-membership proof (proving that `event.packet` has not been received on
    /// `counterparty_chain_id` as of `counterparty_height`) onto the L1 anchoring
    /// `event.packet`'s source client, then refresh the source client's consensus state *at
    /// `counterparty_height`* (not a newer height - the commitment is keyed by this exact
    /// height) before retrying the timeout message.
    async fn commit_proof_lens_non_membership_proof(
        &self,
        voyager_client: &VoyagerClient,
        event: PacketSend,
        chain_id: ChainId,
        counterparty_chain_id: ChainId,
        counterparty_height: Height,
    ) -> RpcResult<Op<VoyagerMessage>> {
        let client_id = event.packet.source_channel.connection.client_id;

        let raw_proof = voyager_client
            .query_ibc_proof(
                counterparty_chain_id.clone(),
                QueryHeight::Specific(counterparty_height),
                BatchReceiptsPath::from_packets(&[event.packet()]),
            )
            .await?
            .into_result()?;

        if raw_proof.proof_type != ProofType::NonMembership {
            info!("packet was received in the meantime, no longer need to commit a timeout proof");

            return Ok(noop());
        }

        let lens = self
            .resolve_proof_lens(voyager_client, &chain_id, client_id)
            .await?
            .ok_or_else(|| RpcError::fatal_from_message("client is not a proof lens client"))?;

        let l2_client_info = voyager_client
            .client_info::<IbcUnion>(lens.l1_chain_id.clone(), lens.l2_client_id)
            .await?;

        let encoded_l2_proof = voyager_client
            .encode_proof::<IbcUnion>(
                l2_client_info.client_type,
                l2_client_info.ibc_interface,
                raw_proof.proof,
            )
            .await?;

        let commit_msg = MsgCommitNonMembershipProof {
            client_id: lens.l2_client_id,
            proof_height: counterparty_height.height(),
            proof: encoded_l2_proof,
            path: BatchReceiptsPath::from_packets(&[event.packet()])
                .key()
                .into(),
        };

        let lens_client_info = voyager_client
            .client_info::<IbcUnion>(chain_id.clone(), client_id)
            .await?;

        Ok(seq([
            call(SubmitTx {
                chain_id: lens.l1_chain_id.clone(),
                datagrams: vec![IbcDatagram::new::<IbcUnion>(Datagram::from(commit_msg))],
            }),
            call(WaitForHeightRelative {
                chain_id: lens.l1_chain_id,
                height_diff: 1,
                finalized: false,
            }),
            // refresh the lens client's consensus state *at counterparty_height* (not a newer
            // height, e.g. via `UpdateClientToHeightTimestamp`) - `FetchUpdateAfterL1Update`
            // always builds a header off the L1's current latest height regardless of whether a
            // consensus state already exists for this L2 height, and `updateClient` overwrites
            // the L1 height for a given L2 height, so this refreshes it to (the now-post-commit)
            // latest without changing the L2 height the client trusts, which the just-submitted
            // commitment's key is bound to.
            promise(
                [call(FetchUpdateHeaders {
                    client_type: lens_client_info.client_type,
                    chain_id: counterparty_chain_id.clone(),
                    counterparty_chain_id: chain_id.clone(),
                    client_id: RawClientId::new(client_id),
                    update_from: counterparty_height,
                    update_to: counterparty_height,
                })],
                [],
                AggregateSubmitTxFromOrderedHeaders {
                    ibc_spec_id: IbcUnion::ID,
                    chain_id: chain_id.clone(),
                    client_id: RawClientId::new(client_id),
                },
            ),
            call(PluginMessage::new(
                self.plugin_name(),
                ModuleCall::from(MakeMsgTimeoutFromTrustedHeight {
                    event,
                    chain_id,
                    counterparty_chain_id,
                }),
            )),
        ]))
    }
}
