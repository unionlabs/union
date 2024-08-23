use enumorph::Enumorph;
use macros::apply;
use queue_msg::{
    call, conc, data, defer_absolute, now, promise, queue_msg, seq, HandleCall, Op, QueueError,
};
use serde_json::Value;
use serde_utils::Hex;
use tracing::{debug, info, info_span, instrument, trace, Instrument};
use unionlabs::{
    ibc::core::{
        channel::{
            self, channel::Channel, msg_acknowledgement::MsgAcknowledgement,
            msg_channel_open_ack::MsgChannelOpenAck,
            msg_channel_open_confirm::MsgChannelOpenConfirm,
            msg_channel_open_try::MsgChannelOpenTry, msg_recv_packet::MsgRecvPacket,
        },
        client::{
            height::{Height, IsHeight},
            msg_create_client::MsgCreateClient,
        },
        commitment::merkle_prefix::MerklePrefix,
        connection::{
            self, connection_end::ConnectionEnd, msg_connection_open_ack::MsgConnectionOpenAck,
            msg_connection_open_confirm::MsgConnectionOpenConfirm,
            msg_connection_open_try::MsgConnectionOpenTry,
        },
    },
    ics24::{
        AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath, Path,
    },
    id::{ClientId, ConnectionId},
    traits::Member,
    QueryHeight, DELAY_PERIOD,
};

use crate::{
    callback::AggregateFetchBlockRange,
    data::{
        ClientInfo, DecodedClientStateMeta, DecodedConsensusStateMeta, EncodedClientState,
        EncodedConsensusState, EncodedHeader, IbcMessage, IbcProof, IbcState, LatestHeight,
        MsgCreateClientData, RawIbcProof, SelfClientState, SelfConsensusState, WithChainId,
    },
    json_rpc_error_to_queue_error,
    plugin::{
        ChainModuleClient, ChainModuleClientExt, ClientModuleClient, ConsensusModuleClient,
        PluginModuleClient,
    },
    top_level_identifiable_enum, Context, IbcInterface, PluginMessage, VoyagerMessage,
};

#[apply(top_level_identifiable_enum)]
#[queue_msg]
#[derive(Enumorph)]
pub enum Call<C = serde_json::Value> {
    FetchBlock(FetchBlock),
    FetchBlockRange(FetchBlockRange),

    State(FetchState),
    RawProof(FetchRawProof),

    LatestHeight(FetchLatestHeight),

    ClientInfo(FetchClientInfo),

    UnfinalizedTrustedClientState(FetchUnfinalizedTrustedClientState),

    SelfClientState(FetchSelfClientState),
    SelfConsensusState(FetchSelfConsensusState),

    DecodeClientStateMeta(DecodeClientStateMeta),
    DecodeConsensusStateMeta(DecodeConsensusStateMeta),

    EncodeClientState(EncodeClientState),
    EncodeConsensusState(EncodeConsensusState),
    EncodeHeader(EncodeHeader),

    EncodeProof(EncodeProof),

    UpdateHeaders(FetchUpdateHeaders),

    MakeMsgCreateClient(MakeMsgCreateClient),

    MakeMsgConnectionOpenTry(MakeMsgConnectionOpenTry),
    MakeMsgConnectionOpenAck(MakeMsgConnectionOpenAck),
    MakeMsgConnectionOpenConfirm(MakeMsgConnectionOpenConfirm),

    MakeMsgChannelOpenTry(MakeMsgChannelOpenTry),
    MakeMsgChannelOpenAck(MakeMsgChannelOpenAck),
    MakeMsgChannelOpenConfirm(MakeMsgChannelOpenConfirm),

    MakeMsgAcknowledgement(MakeMsgAcknowledgement),
    MakeMsgRecvPacket(MakeMsgRecvPacket),

    Height(WaitForHeight),
    HeightRelative(WaitForHeightRelative),
    Timestamp(WaitForTimestamp),
    TrustedHeight(WaitForTrustedHeight),

    Plugin(PluginMessage<C>),
}

#[queue_msg]
pub struct FetchBlockRange {
    pub chain_id: String,
    pub from_height: Height,
    pub to_height: Height,
}

#[queue_msg]
pub struct FetchBlock {
    pub chain_id: String,
    pub height: Height,
}

#[queue_msg]
pub struct FetchSelfClientState {
    pub chain_id: String,
    pub at: QueryHeight,
    /// The counterparty IBC interface that the state must be encoded for.
    pub ibc_interface: IbcInterface<'static>,
    #[serde(default, skip_serializing_if = "Value::is_null")]
    /// Additional metadata that will be passed to
    /// [`ClientModulePlugin::encode_client_state`]. This field is analogous to
    /// [`ClientInfo::metadata`].
    pub metadata: Value,
}

#[queue_msg]
pub struct FetchSelfConsensusState {
    pub chain_id: String,
    pub at: QueryHeight,
    /// The counterparty IBC interface that the state must be encoded for.
    pub ibc_interface: IbcInterface<'static>,
}

// TODO: This should have a height field
#[queue_msg]
pub struct FetchClientInfo {
    pub chain_id: String,
    pub client_id: ClientId,
}

#[queue_msg]
pub struct DecodeClientStateMeta {
    pub ibc_state: IbcState<ClientStatePath>,
    pub client_info: ClientInfo,
}

#[queue_msg]
pub struct DecodeConsensusStateMeta {
    pub ibc_state: IbcState<ClientConsensusStatePath>,
    pub client_info: ClientInfo,
}

#[queue_msg]
pub struct EncodeClientState {
    pub client_state: Value,
    pub client_info: ClientInfo,
}

#[queue_msg]
pub struct EncodeConsensusState {
    pub consensus_state: Value,
    pub client_info: ClientInfo,
}

#[queue_msg]
pub struct EncodeHeader {
    pub header: Value,
    pub client_info: ClientInfo,
}

#[queue_msg]
pub struct EncodeProof {
    pub raw_proof: RawIbcProof,
    pub client_info: ClientInfo,
}

/// Fetches a raw, unenccoded IBC proof from the specified chain.
#[queue_msg]
pub struct FetchRawProof {
    pub chain_id: String,
    pub at: Height,
    pub path: Path,
}

#[queue_msg]
pub struct FetchState {
    pub chain_id: String,
    pub at: QueryHeight,
    pub path: Path,
}

#[queue_msg]
pub struct FetchUpdateHeaders {
    pub chain_id: String,
    pub update_from: Height,
    pub update_to: Height,
}

#[queue_msg]
pub struct FetchLatestHeight {
    pub chain_id: String,
}

#[queue_msg]
pub struct FetchUnfinalizedTrustedClientState {
    pub chain_id: String,
    pub client_id: ClientId,
}

/// Build a [`MsgCreateClient`] [`IbcMessage`].
#[queue_msg]
pub struct MakeMsgCreateClient {
    /// The chain to create the client on.
    pub chain_id: String,
    /// The height of the counterparty that the client will trust. The
    /// [`SelfClientState`] and [`SelfConsensusState`] will be queried at this
    /// height.
    pub height: Height,
    #[serde(default, skip_serializing_if = "Value::is_null")]
    /// Additional metadata that will be passed to
    /// [`ClientModulePlugin::encode_client_state`]. This field is analogous to
    /// [`ClientInfo::metadata`].
    pub metadata: Value,
    /// The chain to create a client of.
    pub counterparty_chain_id: String,
    /// The IBC interface to create the client on.
    pub ibc_interface: IbcInterface<'static>,
}

#[queue_msg]
pub struct MakeMsgConnectionOpenTry {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: String,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: String,
    /// The original event that was emitted on the origin chain.
    pub connection_open_init_event: crate::data::ConnectionOpenInit,
}

#[queue_msg]
pub struct MakeMsgConnectionOpenAck {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: String,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: String,
    /// The original event that was emitted on the origin chain.
    pub connection_open_try_event: crate::data::ConnectionOpenTry,
}

#[queue_msg]
pub struct MakeMsgConnectionOpenConfirm {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: String,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: String,
    /// The original event that was emitted on the origin chain.
    pub connection_open_ack_event: crate::data::ConnectionOpenAck,
}

#[queue_msg]
pub struct MakeMsgChannelOpenTry {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: String,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: String,
    /// The original event that was emitted on the origin chain.
    pub channel_open_init_event: crate::data::ChannelOpenInit,
}

#[queue_msg]
pub struct MakeMsgChannelOpenAck {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: String,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: String,
    /// The original event that was emitted on the origin chain.
    pub channel_open_try_event: crate::data::ChannelOpenTry,
}

#[queue_msg]
pub struct MakeMsgChannelOpenConfirm {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: String,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: String,
    /// The original event that was emitted on the origin chain.
    pub channel_open_ack_event: crate::data::ChannelOpenAck,
}

#[queue_msg]
pub struct MakeMsgRecvPacket {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: String,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: String,
    /// The original event that was emitted on the origin chain.
    pub send_packet_event: crate::data::SendPacket,
}

#[queue_msg]
pub struct MakeMsgAcknowledgement {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: String,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: String,
    /// The original event that was emitted on the origin chain.
    pub write_acknowledgement_event: crate::data::WriteAcknowledgement,
}

#[queue_msg]
pub struct WaitForHeight {
    pub chain_id: String,
    pub height: Height,
}

#[queue_msg]
pub struct WaitForHeightRelative {
    pub chain_id: String,
    pub height: u64,
}

#[queue_msg]
pub struct WaitForTimestamp {
    pub chain_id: String,
    pub timestamp: i64,
}

/// Wait for the client `.client_id` on `Hc` to trust a height >= `.height`,
/// returning the counterparty's client state at that height when it's reached.
#[queue_msg]
pub struct WaitForTrustedHeight {
    pub chain_id: String,
    /// The id of the client on `Hc` who's [`ClientState::height()`] we're
    /// waiting to be >= `.height`.
    pub client_id: ClientId,
    /// The id of the counterparty client on `Tr`, who's state will be fetched
    /// at [`ClientState::height()`] when `.client_id` on `Hc` trusts a height
    /// >= `.height`.
    pub counterparty_client_id: ClientId,
    pub counterparty_chain_id: String,
    pub height: Height,
}

impl<D: Member, F: Member, A: Member> HandleCall<VoyagerMessage<D, F, A>> for Call<F> {
    // #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn handle(self, ctx: &Context) -> Result<Op<VoyagerMessage<D, F, A>>, QueueError> {
        match self {
            Call::FetchBlock(FetchBlock { height, chain_id }) => {
                info!(%height, "fetch_block");

                Ok(promise(
                    [call(WaitForHeight {
                        chain_id,
                        height: height.increment(),
                    })],
                    [],
                    AggregateFetchBlockRange {
                        from_height: height,
                    },
                ))
            }

            Call::FetchBlockRange(FetchBlockRange {
                chain_id,
                from_height,
                to_height,
            }) => {
                info!(%from_height, %to_height, "fetch_block_range");

                Ok(conc([
                    ctx.chain_module(&chain_id)?
                        .fetch_block_range(from_height, to_height)
                        .await
                        .map_err(json_rpc_error_to_queue_error)?,
                    call(FetchBlock {
                        chain_id,
                        height: to_height,
                    }),
                ]))
            }

            Call::RawProof(FetchRawProof { chain_id, at, path }) => Ok(data(RawIbcProof {
                path: path.clone(),
                height: at,
                proof: ctx
                    .chain_module::<Value, Value, Value>(&chain_id)?
                    .query_ibc_proof(at, path)
                    .await
                    .map_err(json_rpc_error_to_queue_error)?,
            })),

            Call::State(FetchState { chain_id, at, path }) => match at {
                QueryHeight::Latest => Ok(call(FetchState {
                    at: QueryHeight::Specific(
                        ctx.chain_module::<D, F, A>(&chain_id)?
                            .query_latest_height()
                            .await
                            .map_err(json_rpc_error_to_queue_error)?,
                    ),
                    chain_id,
                    path,
                })),
                QueryHeight::Specific(at) => {
                    let state = ctx
                        .chain_module::<Value, Value, Value>(&chain_id)?
                        .query_ibc_state(at, path.clone())
                        .await
                        .map_err(json_rpc_error_to_queue_error)?;

                    Ok(match path {
                        Path::ClientState(path) => data(IbcState {
                            chain_id,
                            path,
                            height: at,
                            state: serde_json::from_value(state).expect("valid state value"),
                        }),
                        Path::ClientConsensusState(path) => data(IbcState {
                            chain_id,
                            path,
                            height: at,
                            state: serde_json::from_value(state).expect("valid state value"),
                        }),
                        Path::Connection(path) => data(IbcState {
                            chain_id,
                            path,
                            height: at,
                            state: serde_json::from_value(state).expect("valid state value"),
                        }),
                        Path::ChannelEnd(path) => data(IbcState {
                            chain_id,
                            path,
                            height: at,
                            state: serde_json::from_value(state).expect("valid state value"),
                        }),
                        Path::Commitment(path) => data(IbcState {
                            chain_id,
                            path,
                            height: at,
                            state: serde_json::from_value(state).expect("valid state value"),
                        }),
                        Path::Acknowledgement(path) => data(IbcState {
                            chain_id,
                            path,
                            height: at,
                            state: serde_json::from_value(state).expect("valid state value"),
                        }),
                        Path::Receipt(path) => data(IbcState {
                            chain_id,
                            path,
                            height: at,
                            state: serde_json::from_value(state).expect("valid state value"),
                        }),
                        Path::NextSequenceSend(path) => data(IbcState {
                            chain_id,
                            path,
                            height: at,
                            state: serde_json::from_value(state).expect("valid state value"),
                        }),
                        Path::NextSequenceRecv(path) => data(IbcState {
                            chain_id,
                            path,
                            height: at,
                            state: serde_json::from_value(state).expect("valid state value"),
                        }),
                        Path::NextSequenceAck(path) => data(IbcState {
                            chain_id,
                            path,
                            height: at,
                            state: serde_json::from_value(state).expect("valid state value"),
                        }),
                        Path::NextConnectionSequence(path) => data(IbcState {
                            chain_id,
                            path,
                            height: at,
                            state: serde_json::from_value(state).expect("valid state value"),
                        }),
                        Path::NextClientSequence(path) => data(IbcState {
                            chain_id,
                            path,
                            height: at,
                            state: serde_json::from_value(state).expect("valid state value"),
                        }),
                    })
                }
            },

            Call::LatestHeight(FetchLatestHeight { chain_id }) => Ok(data(LatestHeight {
                height: ctx
                    .chain_module::<D, F, A>(&chain_id)?
                    .query_latest_height()
                    .await
                    .map_err(json_rpc_error_to_queue_error)?,
                chain_id,
            })),

            Call::ClientInfo(FetchClientInfo {
                chain_id,
                client_id,
            }) => Ok(data(
                ctx.chain_module::<D, F, A>(&chain_id)?
                    .client_info(client_id)
                    .await
                    .map_err(json_rpc_error_to_queue_error)?,
            )),

            // TODO: This is currently unused, but there is a wait that uses this call that needs to
            // be refactored to be an aggregation using this
            Call::UnfinalizedTrustedClientState(FetchUnfinalizedTrustedClientState {
                chain_id: _,
                client_id: _,
            }) => {
                // let client_state = ctx
                //     .chain_module(&chain_id)
                //     ?
                //     .query_raw_unfinalized_trusted_client_state(client_id)
                //     .await
                //     .map_err(json_rpc_error_to_queue_error)?;

                // let decoded = ctx
                //     .client_module(&client_state.client_type, &client_state.ibc_interface)
                //     ?
                //     .decode_client_state_meta(client_state.bytes, client_state.ibc_interface)
                //     .await
                //     .map_err(json_rpc_error_to_queue_error)?;

                // Ok(data(id(
                //     self.chain_id,
                //     UnfinalizedTrustedClientState {
                //         height,
                //         client_state,
                //     },
                // )))
                todo!()
            }

            Call::SelfClientState(FetchSelfClientState {
                chain_id,
                at: height,
                ibc_interface,
                metadata,
            }) => {
                // TODO: Split this into a separate query and aggregate
                let height = match height {
                    QueryHeight::Latest => ctx
                        .chain_module::<D, F, A>(&chain_id)?
                        .query_latest_height()
                        .await
                        .map_err(json_rpc_error_to_queue_error)?,
                    QueryHeight::Specific(h) => h,
                };

                info!(%height, "querying self client state");

                let self_client_state = ctx
                    .consensus_module::<D, F, A>(&chain_id)?
                    .self_client_state(height)
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                // REVIEW: Add an assert here that the returned chain_id is the same as the
                // passed in one?
                let client_type = ctx
                    .consensus_module::<D, F, A>(&chain_id)?
                    .consensus_info()
                    .await
                    .map_err(json_rpc_error_to_queue_error)?
                    .client_type;

                let self_client_state = ctx
                    .client_module::<D, F, A>(&client_type, &ibc_interface)?
                    .encode_client_state(self_client_state, metadata)
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                Ok(data(SelfClientState { self_client_state }))
            }

            Call::SelfConsensusState(FetchSelfConsensusState {
                chain_id,
                at: height,
                ibc_interface,
            }) => {
                // TODO: Split this into a separate query and aggregate?
                let height = match height {
                    QueryHeight::Latest => ctx
                        .chain_module::<D, F, A>(&chain_id)?
                        .query_latest_height()
                        .await
                        .map_err(json_rpc_error_to_queue_error)?,
                    QueryHeight::Specific(h) => h,
                };

                let self_consensus_state = ctx
                    .consensus_module::<D, F, A>(&chain_id)?
                    .self_consensus_state(height)
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                let client_type = ctx
                    .consensus_module::<D, F, A>(&chain_id)?
                    .consensus_info()
                    .await
                    .map_err(json_rpc_error_to_queue_error)?
                    .client_type;

                let self_consensus_state = ctx
                    .client_module::<D, F, A>(&client_type, &ibc_interface)?
                    .encode_consensus_state(self_consensus_state)
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                Ok(data(SelfConsensusState {
                    self_consensus_state,
                }))
            }

            Call::DecodeClientStateMeta(DecodeClientStateMeta {
                ibc_state,
                client_info,
            }) => {
                debug!(
                    %client_info.client_type,
                    %client_info.ibc_interface,
                    "decode client state meta"
                );

                Ok(data(DecodedClientStateMeta {
                    path: ibc_state.path,
                    height: ibc_state.height,
                    state: ctx
                        .client_module::<D, F, A>(
                            &client_info.client_type,
                            &client_info.ibc_interface,
                        )?
                        .decode_client_state_meta(ibc_state.state.0.into())
                        .await
                        .map_err(json_rpc_error_to_queue_error)?,
                }))
            }

            Call::DecodeConsensusStateMeta(DecodeConsensusStateMeta {
                ibc_state,
                client_info,
            }) => Ok(data(DecodedConsensusStateMeta {
                path: ibc_state.path,
                height: ibc_state.height,
                state: ctx
                    .client_module::<D, F, A>(&client_info.client_type, &client_info.ibc_interface)?
                    .decode_consensus_state_meta(ibc_state.state.0.into())
                    .await
                    .map_err(json_rpc_error_to_queue_error)?,
            })),

            Call::EncodeClientState(EncodeClientState {
                client_state,
                client_info,
            }) => Ok(data(EncodedClientState {
                encoded_client_state: ctx
                    .client_module::<D, F, A>(&client_info.client_type, &client_info.ibc_interface)?
                    .encode_client_state(client_state, client_info.metadata)
                    .await
                    .map_err(json_rpc_error_to_queue_error)?,
            })),

            Call::EncodeConsensusState(EncodeConsensusState {
                consensus_state,
                client_info,
            }) => Ok(data(EncodedConsensusState {
                encoded_consensus_state: ctx
                    .client_module::<D, F, A>(&client_info.client_type, &client_info.ibc_interface)?
                    .encode_consensus_state(consensus_state)
                    .await
                    .map_err(json_rpc_error_to_queue_error)?,
            })),

            Call::EncodeHeader(EncodeHeader {
                header,
                client_info,
            }) => Ok(data(EncodedHeader {
                encoded_header: ctx
                    .client_module::<D, F, A>(&client_info.client_type, &client_info.ibc_interface)?
                    .encode_header(header)
                    .await
                    .map_err(json_rpc_error_to_queue_error)?,
            })),

            Call::EncodeProof(EncodeProof {
                raw_proof:
                    RawIbcProof {
                        path,
                        height,
                        proof,
                    },
                client_info,
            }) => {
                let proof = ctx
                    .client_module::<D, F, A>(&client_info.client_type, &client_info.ibc_interface)?
                    .encode_proof(proof)
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                // assign to a value so we can apply the attribute
                #[rustfmt::skip]
                let r = match path {
                    Path::ClientState(path)            => Ok(data(IbcProof { path, height, proof, })),
                    Path::ClientConsensusState(path)   => Ok(data(IbcProof { path, height, proof, })),
                    Path::Connection(path)             => Ok(data(IbcProof { path, height, proof, })),
                    Path::ChannelEnd(path)             => Ok(data(IbcProof { path, height, proof, })),
                    Path::Commitment(path)             => Ok(data(IbcProof { path, height, proof, })),
                    Path::Acknowledgement(path)        => Ok(data(IbcProof { path, height, proof, })),
                    Path::Receipt(path)                => Ok(data(IbcProof { path, height, proof, })),
                    Path::NextSequenceSend(path)       => Ok(data(IbcProof { path, height, proof, })),
                    Path::NextSequenceRecv(path)       => Ok(data(IbcProof { path, height, proof, })),
                    Path::NextSequenceAck(path)        => Ok(data(IbcProof { path, height, proof, })),
                    Path::NextConnectionSequence(path) => Ok(data(IbcProof { path, height, proof, })),
                    Path::NextClientSequence(path)     => Ok(data(IbcProof { path, height, proof, })),
                };

                r
            }

            Call::MakeMsgConnectionOpenTry(MakeMsgConnectionOpenTry {
                origin_chain_id,
                origin_chain_proof_height,
                target_chain_id,
                connection_open_init_event,
            }) => {
                let ConnectionHandshakeStateAndProofs {
                    connection_state,
                    encoded_client_state,
                    encoded_client_state_proof,
                    encoded_consensus_state_proof,
                    encoded_connection_state_proof,
                    consensus_height,
                } = mk_connection_handshake_state_and_proofs(
                    ctx,
                    origin_chain_id,
                    target_chain_id,
                    connection_open_init_event.client_id.clone(),
                    connection_open_init_event.counterparty_client_id.clone(),
                    connection_open_init_event.connection_id.clone(),
                    origin_chain_proof_height,
                )
                .await?;

                Ok(data(IbcMessage::from(MsgConnectionOpenTry {
                    client_id: connection_open_init_event.counterparty_client_id,
                    client_state: encoded_client_state,
                    counterparty: connection::counterparty::Counterparty {
                        client_id: connection_open_init_event.client_id,
                        connection_id: Some(connection_open_init_event.connection_id),
                        prefix: MerklePrefix {
                            // TODO: Make configurable
                            key_prefix: b"ibc".to_vec(),
                        },
                    },
                    // TODO: Make configurable
                    delay_period: DELAY_PERIOD,
                    counterparty_versions: connection_state.versions,
                    proof_height: origin_chain_proof_height,
                    proof_init: encoded_connection_state_proof,
                    proof_client: encoded_client_state_proof,
                    proof_consensus: encoded_consensus_state_proof,
                    consensus_height,
                })))
            }

            Call::MakeMsgConnectionOpenAck(MakeMsgConnectionOpenAck {
                origin_chain_id,
                origin_chain_proof_height,
                target_chain_id,
                connection_open_try_event,
            }) => {
                let ConnectionHandshakeStateAndProofs {
                    connection_state,
                    encoded_client_state,
                    encoded_client_state_proof,
                    encoded_consensus_state_proof,
                    encoded_connection_state_proof,
                    consensus_height,
                } = mk_connection_handshake_state_and_proofs(
                    ctx,
                    origin_chain_id,
                    target_chain_id,
                    connection_open_try_event.client_id,
                    connection_open_try_event.counterparty_client_id,
                    connection_open_try_event.connection_id.clone(),
                    origin_chain_proof_height,
                )
                .await?;

                Ok(queue_msg::data(IbcMessage::from(MsgConnectionOpenAck {
                    connection_id: connection_open_try_event.counterparty_connection_id,
                    counterparty_connection_id: connection_open_try_event.connection_id,
                    client_state: encoded_client_state,
                    version: connection_state.versions[0].clone(),
                    proof_height: origin_chain_proof_height,
                    proof_try: encoded_connection_state_proof,
                    proof_client: encoded_client_state_proof,
                    proof_consensus: encoded_consensus_state_proof,
                    consensus_height,
                })))
            }

            Call::MakeMsgConnectionOpenConfirm(MakeMsgConnectionOpenConfirm {
                origin_chain_id,
                origin_chain_proof_height,
                target_chain_id,
                connection_open_ack_event,
            }) => {
                let origin_chain_module =
                    ctx.chain_module::<Value, Value, Value>(origin_chain_id)?;

                let target_chain_module =
                    ctx.chain_module::<Value, Value, Value>(target_chain_id)?;

                // info of the client on the target chain that will verify the storage
                // proofs
                let target_client_info = target_chain_module
                    // counterparty_client_id from open_try is the client on the target chain
                    .client_info(connection_open_ack_event.counterparty_client_id.clone())
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                // client module for the client on the origin chain (the chain the event was
                // emitted on)
                let target_client_module = ctx.client_module::<Value, Value, Value>(
                    &target_client_info.client_type,
                    &target_client_info.ibc_interface,
                )?;

                // proof of connection_state, encoded for the client on the target chain
                let connection_proof = target_client_module
                    .encode_proof(
                        origin_chain_module
                            .query_ibc_proof(
                                origin_chain_proof_height,
                                ConnectionPath {
                                    connection_id: connection_open_ack_event.connection_id.clone(),
                                }
                                .into(),
                            )
                            .await
                            .map_err(json_rpc_error_to_queue_error)?,
                    )
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                Ok(queue_msg::data(IbcMessage::from(
                    MsgConnectionOpenConfirm {
                        connection_id: connection_open_ack_event.counterparty_connection_id,
                        proof_height: origin_chain_proof_height,
                        proof_ack: connection_proof,
                    },
                )))
            }

            Call::MakeMsgChannelOpenTry(MakeMsgChannelOpenTry {
                origin_chain_id,
                origin_chain_proof_height,
                target_chain_id,
                channel_open_init_event: event,
            }) => {
                let origin_chain_module =
                    ctx.chain_module::<Value, Value, Value>(origin_chain_id)?;

                let target_chain_module =
                    ctx.chain_module::<Value, Value, Value>(target_chain_id)?;

                let origin_channel_path = ChannelEndPath {
                    port_id: event.port_id.clone(),
                    channel_id: event.channel_id.clone(),
                };

                let origin_channel = origin_chain_module
                    .query_ibc_state_typed(origin_chain_proof_height, origin_channel_path.clone())
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                let proof_init = origin_chain_module
                    .query_ibc_proof(origin_chain_proof_height, origin_channel_path.into())
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                let client_info = target_chain_module
                    .client_info(event.connection.counterparty.client_id)
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                let encoded_proof_init = ctx
                    .client_module::<Value, Value, Value>(
                        &client_info.client_type,
                        &client_info.ibc_interface,
                    )?
                    .encode_proof(proof_init)
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                Ok(data(IbcMessage::from(MsgChannelOpenTry {
                    port_id: event.counterparty_port_id,
                    channel: Channel {
                        state: channel::state::State::Tryopen,
                        ordering: origin_channel.ordering,
                        counterparty: channel::counterparty::Counterparty {
                            port_id: event.port_id,
                            channel_id: event.channel_id.to_string(),
                        },
                        connection_hops: vec![event.connection.counterparty.connection_id.unwrap()],
                        version: event.version.clone(),
                    },
                    counterparty_version: event.version,
                    proof_init: encoded_proof_init,
                    proof_height: origin_chain_proof_height,
                })))
            }

            Call::MakeMsgChannelOpenAck(MakeMsgChannelOpenAck {
                origin_chain_id,
                origin_chain_proof_height,
                target_chain_id,
                channel_open_try_event: event,
            }) => {
                let origin_chain_module =
                    ctx.chain_module::<Value, Value, Value>(origin_chain_id)?;

                let target_chain_module =
                    ctx.chain_module::<Value, Value, Value>(target_chain_id)?;

                let origin_channel_path = ChannelEndPath {
                    port_id: event.port_id,
                    channel_id: event.channel_id.clone(),
                };

                let proof_try = origin_chain_module
                    .query_ibc_proof(origin_chain_proof_height, origin_channel_path.into())
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                let client_info = target_chain_module
                    .client_info(event.connection.counterparty.client_id)
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                let encoded_proof_try = ctx
                    .client_module::<Value, Value, Value>(
                        &client_info.client_type,
                        &client_info.ibc_interface,
                    )?
                    .encode_proof(proof_try)
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                Ok(data(IbcMessage::from(MsgChannelOpenAck {
                    port_id: event.counterparty_port_id,
                    channel_id: event.counterparty_channel_id,
                    counterparty_channel_id: event.channel_id,
                    counterparty_version: event.version,
                    proof_try: encoded_proof_try,
                    proof_height: origin_chain_proof_height,
                })))
            }

            Call::MakeMsgChannelOpenConfirm(MakeMsgChannelOpenConfirm {
                origin_chain_id,
                origin_chain_proof_height,
                target_chain_id,
                channel_open_ack_event,
            }) => {
                let origin_chain_module =
                    ctx.chain_module::<Value, Value, Value>(origin_chain_id)?;

                let target_chain_module =
                    ctx.chain_module::<Value, Value, Value>(target_chain_id)?;

                let origin_channel_path = ChannelEndPath {
                    port_id: channel_open_ack_event.port_id,
                    channel_id: channel_open_ack_event.channel_id,
                };

                let proof_ack = origin_chain_module
                    .query_ibc_proof(origin_chain_proof_height, origin_channel_path.into())
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                let client_info = target_chain_module
                    .client_info(channel_open_ack_event.connection.counterparty.client_id)
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                let encoded_proof_ack = ctx
                    .client_module::<Value, Value, Value>(
                        &client_info.client_type,
                        &client_info.ibc_interface,
                    )?
                    .encode_proof(proof_ack)
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                Ok(queue_msg::data(IbcMessage::from(MsgChannelOpenConfirm {
                    port_id: channel_open_ack_event.counterparty_port_id,
                    channel_id: channel_open_ack_event.counterparty_channel_id,
                    proof_ack: encoded_proof_ack,
                    proof_height: origin_chain_proof_height,
                })))
            }

            Call::MakeMsgRecvPacket(MakeMsgRecvPacket {
                origin_chain_id,
                origin_chain_proof_height,
                target_chain_id,
                send_packet_event,
            }) => {
                let origin_chain_module =
                    ctx.chain_module::<Value, Value, Value>(origin_chain_id)?;

                let target_chain_module =
                    ctx.chain_module::<Value, Value, Value>(target_chain_id)?;

                let proof_commitment = origin_chain_module
                    .query_ibc_proof(
                        origin_chain_proof_height,
                        CommitmentPath {
                            port_id: send_packet_event.packet.source_channel.port_id.clone(),
                            channel_id: send_packet_event.packet.source_channel.channel_id.clone(),
                            sequence: send_packet_event.packet.sequence,
                        }
                        .into(),
                    )
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                let client_info = target_chain_module
                    .client_info(
                        send_packet_event
                            .packet
                            .destination_channel
                            .connection
                            .client_id,
                    )
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                let encoded_proof_commitment = ctx
                    .client_module::<Value, Value, Value>(
                        &client_info.client_type,
                        &client_info.ibc_interface,
                    )?
                    .encode_proof(proof_commitment)
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                Ok(queue_msg::data(IbcMessage::from(MsgRecvPacket {
                    packet: channel::packet::Packet {
                        sequence: send_packet_event.packet.sequence,
                        source_port: send_packet_event.packet.source_channel.port_id,
                        source_channel: send_packet_event.packet.source_channel.channel_id,
                        destination_port: send_packet_event.packet.destination_channel.port_id,
                        destination_channel: send_packet_event
                            .packet
                            .destination_channel
                            .channel_id,
                        data: send_packet_event.packet_data,
                        timeout_height: send_packet_event.packet.timeout_height,
                        timeout_timestamp: send_packet_event.packet.timeout_timestamp,
                    },
                    proof_commitment: encoded_proof_commitment,
                    proof_height: origin_chain_proof_height,
                })))
            }

            Call::MakeMsgAcknowledgement(MakeMsgAcknowledgement {
                origin_chain_id,
                origin_chain_proof_height,
                target_chain_id,
                write_acknowledgement_event,
            }) => {
                async {
                    let origin_chain_module =
                        ctx.chain_module::<Value, Value, Value>(&origin_chain_id)?;

                    let target_chain_module =
                        ctx.chain_module::<Value, Value, Value>(&target_chain_id)?;

                    let proof_acked = origin_chain_module
                        .query_ibc_proof(
                            origin_chain_proof_height,
                            AcknowledgementPath {
                                port_id: write_acknowledgement_event
                                    .packet
                                    .destination_channel
                                    .port_id
                                    .clone(),
                                channel_id: write_acknowledgement_event
                                    .packet
                                    .destination_channel
                                    .channel_id
                                    .clone(),
                                sequence: write_acknowledgement_event.packet.sequence,
                            }
                            .into(),
                        )
                        .await
                        .map_err(json_rpc_error_to_queue_error)?;

                    let client_info = target_chain_module
                        .client_info(
                            write_acknowledgement_event
                                .packet
                                .source_channel
                                .connection
                                .client_id,
                        )
                        .await
                        .map_err(json_rpc_error_to_queue_error)?;

                    let encoded_proof_acked = ctx
                        .client_module::<Value, Value, Value>(
                            &client_info.client_type,
                            &client_info.ibc_interface,
                        )?
                        .encode_proof(proof_acked)
                        .await
                        .map_err(json_rpc_error_to_queue_error)?;

                    Ok(queue_msg::data(IbcMessage::from(MsgAcknowledgement {
                        packet: channel::packet::Packet {
                            sequence: write_acknowledgement_event.packet.sequence,
                            source_port: write_acknowledgement_event.packet.source_channel.port_id,
                            source_channel: write_acknowledgement_event
                                .packet
                                .source_channel
                                .channel_id,
                            destination_port: write_acknowledgement_event
                                .packet
                                .destination_channel
                                .port_id,
                            destination_channel: write_acknowledgement_event
                                .packet
                                .destination_channel
                                .channel_id,
                            data: write_acknowledgement_event.packet_data,
                            timeout_height: write_acknowledgement_event.packet.timeout_height,
                            timeout_timestamp: write_acknowledgement_event.packet.timeout_timestamp,
                        },
                        acknowledgement: write_acknowledgement_event.packet_ack,
                        proof_acked: encoded_proof_acked,
                        proof_height: origin_chain_proof_height,
                    })))
                }
                .instrument(info_span!(
                    "make_msg_acknowledgement",
                    %origin_chain_id,
                    %origin_chain_proof_height,
                    %target_chain_id,
                ))
                .await
            }

            // Fetch::UpdateHeaders(FetchUpdateHeaders {
            //     chain_id,
            //     counterparty_chain_id,
            //     counterparty_client_id,
            //     update_from,
            //     update_to,
            // }) => Ok(aggregate(
            //     [
            //         ctx.consensus_module(&chain_id)?
            //             .fetch_update_headers(update_from, update_to)
            //             .await
            //             .map_err(json_rpc_error_to_queue_error)?,
            //         // REVIEW: If we notice that this causes too much latency, it can be
            // pre-fetched and put into the data section. I would prefer to keep all fetch messages
            // "single-purpose" if possible though.         fetch(FetchClientInfo {
            //             chain_id: counterparty_chain_id.clone(),
            //             client_id: counterparty_client_id.clone(),
            //         }),
            //     ],
            //     [],
            //     AggregateMsgUpdateClientsFromOrderedHeaders {
            //         counterparty_chain_id,
            //         counterparty_client_id,
            //     },
            // )),
            Call::UpdateHeaders(FetchUpdateHeaders {
                chain_id,
                update_from,
                update_to,
            }) => ctx
                .consensus_module(&chain_id)?
                .fetch_update_headers(update_from, update_to)
                .await
                .map_err(json_rpc_error_to_queue_error),

            Call::MakeMsgCreateClient(MakeMsgCreateClient {
                chain_id,
                height,
                metadata,
                counterparty_chain_id,
                ibc_interface,
            }) => {
                make_msg_create_client(
                    ctx,
                    counterparty_chain_id,
                    height,
                    chain_id,
                    ibc_interface,
                    metadata,
                )
                .await
            }

            // TODO: Replace this with an aggregation
            Call::Height(WaitForHeight { chain_id, height }) => {
                let chain_height = ctx
                    .chain_module::<D, F, A>(&chain_id)?
                    .query_latest_height()
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                assert_eq!(
                    chain_height.revision_number, height.revision_number,
                    "chain_height: {chain_height}, height: {height}",
                );

                debug!("latest height is {chain_height}, waiting for {height}");

                if chain_height.revision_height >= height.revision_height {
                    Ok(data(LatestHeight {
                        chain_id,
                        height: chain_height,
                    }))
                } else {
                    Ok(seq([
                        defer_absolute(now() + 1),
                        call(WaitForHeight { chain_id, height }),
                    ]))
                }
            }
            // REVIEW: Perhaps remove, unused
            Call::HeightRelative(WaitForHeightRelative { chain_id, height }) => {
                let chain_height = ctx
                    .chain_module::<D, F, A>(&chain_id)?
                    .query_latest_height()
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                Ok(call(WaitForHeight {
                    chain_id,
                    height: Height {
                        revision_number: chain_height.revision_number,
                        revision_height: chain_height.revision_height + height,
                    },
                }))
            }

            Call::Timestamp(WaitForTimestamp {
                chain_id,
                timestamp,
            }) => {
                let chain_ts = ctx
                    .chain_module::<D, F, A>(&chain_id)?
                    .query_latest_timestamp()
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                if chain_ts >= timestamp {
                    // TODO: Figure out a way to fetch a height at a specific timestamp
                    Ok(data(LatestHeight {
                        height: ctx
                            .chain_module::<D, F, A>(&chain_id)?
                            .query_latest_height()
                            .await
                            .map_err(json_rpc_error_to_queue_error)?,
                        chain_id,
                    }))
                } else {
                    Ok(seq([
                        // REVIEW: Defer until `now + chain.block_time()`? Would require a new
                        // method on chain
                        defer_absolute(now() + 1),
                        call(WaitForTimestamp {
                            chain_id,
                            timestamp,
                        }),
                    ]))
                }
            }

            Call::TrustedHeight(WaitForTrustedHeight {
                chain_id,
                client_id,
                counterparty_client_id,
                counterparty_chain_id,
                height,
            }) => {
                let client_state = ctx
                    .chain_module::<D, F, A>(&chain_id)?
                    .query_raw_unfinalized_trusted_client_state(client_id.clone())
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                let trusted_client_state_meta = ctx
                    .client_module::<D, F, A>(
                        &client_state.client_type,
                        &client_state.ibc_interface,
                    )?
                    .decode_client_state_meta(client_state.bytes)
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                if trusted_client_state_meta.height.revision_height >= height.revision_height {
                    debug!(
                        "client height reached ({} >= {})",
                        trusted_client_state_meta.height, height
                    );

                    // the height has been reached, fetch the counterparty client state on `Tr` at
                    // the trusted height
                    Ok(call(FetchState {
                        chain_id: counterparty_chain_id,
                        at: QueryHeight::Specific(trusted_client_state_meta.height),
                        path: ClientStatePath {
                            client_id: counterparty_client_id.clone(),
                        }
                        .into(),
                    }))
                } else {
                    Ok(seq([
                        // REVIEW: Defer until `now + counterparty_chain.block_time()`? Would
                        // require a new method on chain
                        defer_absolute(now() + 1),
                        call(WaitForTrustedHeight {
                            chain_id,
                            client_id,
                            height,
                            counterparty_client_id,
                            counterparty_chain_id,
                        }),
                    ]))
                }
            }

            Call::Plugin(PluginMessage { plugin, message }) => Ok(ctx
                .plugin(plugin)?
                .handle_fetch(message)
                .await
                .map_err(json_rpc_error_to_queue_error)?),
        }
    }
}

#[instrument(
    skip_all,
    fields(
        %counterparty_chain_id,
        %height,
        %chain_id,
        %ibc_interface,
        %metadata,
    )
)]
async fn make_msg_create_client<D: Member, F: Member, A: Member>(
    ctx: &Context,
    counterparty_chain_id: String,
    height: Height,
    chain_id: String,
    ibc_interface: IbcInterface<'_>,
    metadata: Value,
) -> Result<Op<VoyagerMessage<D, F, A>>, QueueError> {
    let counterparty_consensus_module =
        ctx.consensus_module::<Value, Value, Value>(counterparty_chain_id)?;

    let self_client_state = counterparty_consensus_module
        .self_client_state(height)
        .await
        .map_err(json_rpc_error_to_queue_error)?;
    trace!(%self_client_state);

    let self_consensus_state = counterparty_consensus_module
        .self_consensus_state(height)
        .await
        .map_err(json_rpc_error_to_queue_error)?;
    trace!(%self_consensus_state);

    let client_type = counterparty_consensus_module
        .consensus_info()
        .await
        .map_err(json_rpc_error_to_queue_error)?
        .client_type;

    let client_module = ctx.client_module::<Value, Value, Value>(&client_type, &ibc_interface)?;

    Ok(data(WithChainId {
        chain_id,
        message: IbcMessage::from(MsgCreateClientData {
            msg: MsgCreateClient {
                client_state: client_module
                    .encode_client_state(self_client_state, metadata)
                    .await
                    .map_err(json_rpc_error_to_queue_error)?,
                consensus_state: client_module
                    .encode_consensus_state(self_consensus_state)
                    .await
                    .map_err(json_rpc_error_to_queue_error)?,
            },
            client_type,
        }),
    }))
}

pub mod compound {
    use queue_msg::{call, promise, Op};
    use unionlabs::{
        ics24::{ChannelEndPath, ClientStatePath},
        id::{ChannelId, ClientId, PortId},
        traits::Member,
        QueryHeight,
    };

    use crate::{
        call::{FetchClientInfo, FetchState},
        callback::{
            AggregateDecodeClientStateMeta, AggregateDecodeClientStateMetaFromConnection,
            AggregateFetchConnectionFromChannel,
        },
        VoyagerMessage,
    };

    /// Fetches the underlying connection of the provided channel ids on the
    /// specified chain.
    ///
    /// This expands to [`AggregateFetchConnectionFromChannel`].
    pub fn fetch_connection_from_channel_info<D: Member, F: Member, A: Member>(
        chain_id: String,
        at: QueryHeight,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> Op<VoyagerMessage<D, F, A>> {
        promise(
            [call(FetchState {
                chain_id,
                at,
                path: ChannelEndPath {
                    port_id,
                    channel_id,
                }
                .into(),
            })],
            [],
            AggregateFetchConnectionFromChannel {},
        )
    }

    /// Fetches the underlying connection of the provided channel ids on the
    /// specified chain.
    ///
    /// This expands to [`AggregateFetchConnectionFromConnection`].
    pub fn fetch_client_state_meta_from_connection<D: Member, F: Member, A: Member>(
        chain_id: String,
        at: QueryHeight,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> Op<VoyagerMessage<D, F, A>> {
        promise(
            [fetch_connection_from_channel_info(
                chain_id, at, port_id, channel_id,
            )],
            [],
            AggregateDecodeClientStateMetaFromConnection {},
        )
    }

    /// Fetches the underlying connection of the provided channel ids on the
    /// specified chain.
    ///
    /// This expands to [`AggregateFetchConnectionFromConnection`].
    pub fn fetch_client_state_meta<D: Member, F: Member, A: Member>(
        chain_id: String,
        client_id: ClientId,
        at: QueryHeight,
    ) -> Op<VoyagerMessage<D, F, A>> {
        promise(
            [
                call(FetchClientInfo {
                    chain_id: chain_id.clone(),
                    client_id: client_id.clone(),
                }),
                call(FetchState {
                    chain_id,
                    at,
                    path: ClientStatePath { client_id }.into(),
                }),
            ],
            [],
            AggregateDecodeClientStateMeta {},
        )
    }
}

/// Used to fetch and construct the state and proofs for
/// MsgConnectionOpenTry/Ack.
async fn mk_connection_handshake_state_and_proofs(
    ctx: &Context,
    origin_chain_id: String,
    target_chain_id: String,
    client_id: ClientId,
    counterparty_client_id: ClientId,
    connection_id: ConnectionId,
    origin_chain_proof_height: Height,
) -> Result<ConnectionHandshakeStateAndProofs, QueueError> {
    let origin_chain_module = ctx.chain_module::<Value, Value, Value>(origin_chain_id)?;

    let target_chain_module = ctx.chain_module::<Value, Value, Value>(target_chain_id)?;

    // info of the client on the target chain that will verify the storage
    // proofs
    let target_client_info = target_chain_module
        // counterparty_client_id from open_init/try is the client on the target chain
        .client_info(counterparty_client_id.clone())
        .await
        .map_err(json_rpc_error_to_queue_error)?;

    debug!(
        %counterparty_client_id,
        %target_client_info.client_type,
        %target_client_info.ibc_interface,
        %target_client_info.metadata,
    );

    // client module for the client on the target chain
    let target_client_module = ctx.client_module::<Value, Value, Value>(
        &target_client_info.client_type,
        &target_client_info.ibc_interface,
    )?;

    // info of the client on the origin chain, this is used to decode the stored
    // client state
    let origin_client_info = origin_chain_module
        // client_id from open_init/try is the client on the origin chain
        .client_info(client_id.clone())
        .await
        .map_err(json_rpc_error_to_queue_error)?;

    debug!(
        %client_id,
        %origin_client_info.client_type,
        %origin_client_info.ibc_interface,
        %origin_client_info.metadata,
    );

    // client module for the client on the origin chain (the chain the event was
    // emitted on)
    let origin_client_module = ctx.client_module::<Value, Value, Value>(
        &origin_client_info.client_type,
        &origin_client_info.ibc_interface,
    )?;

    // client state of the destination on the source
    let client_state = origin_chain_module
        .query_ibc_state_typed(
            origin_chain_proof_height,
            ClientStatePath {
                client_id: client_id.clone(),
            },
        )
        .await
        .map_err(json_rpc_error_to_queue_error)?;

    debug!(%client_state);

    // the client state meta of the target chain on the origin chain, that
    // contains a trusted height of the destination TODO: maybe assert the
    // chain_id is as expected?
    let client_meta = origin_client_module
        .decode_client_state_meta(client_state.0.clone().into())
        .await
        .map_err(json_rpc_error_to_queue_error)?;

    debug!(
        %client_meta.height,
        %client_meta.chain_id,
    );

    let reencoded_client_state = target_client_module
        .reencode_counterparty_client_state(
            client_state.0.clone().into(),
            origin_client_info.client_type,
        )
        .await
        .map_err(json_rpc_error_to_queue_error)?;

    debug!(reencoded_client_state = %Hex(&reencoded_client_state));

    // the connection end as stored by the origin chain after open_init/try
    let connection_state = origin_chain_module
        .query_ibc_state_typed(
            origin_chain_proof_height,
            ConnectionPath {
                connection_id: connection_id.clone(),
            },
        )
        .await
        .map_err(json_rpc_error_to_queue_error)?;
    debug!(
        connection_state = %serde_json::to_string(&connection_state).unwrap(),
    );

    // proof of connection_state, encoded for the client on the target chain
    let connection_proof = origin_chain_module
        .query_ibc_proof(
            origin_chain_proof_height,
            ConnectionPath {
                connection_id: connection_id.clone(),
            }
            .into(),
        )
        .await
        .map_err(json_rpc_error_to_queue_error)?;
    debug!(%connection_proof);

    let encoded_connection_state_proof = target_client_module
        .encode_proof(connection_proof)
        .await
        .map_err(json_rpc_error_to_queue_error)?;
    debug!(encoded_connection_state_proof = %Hex(&encoded_connection_state_proof));

    let client_state_proof = origin_chain_module
        .query_ibc_proof(
            origin_chain_proof_height,
            ClientStatePath {
                client_id: client_id.clone(),
            }
            .into(),
        )
        .await
        .map_err(json_rpc_error_to_queue_error)?;
    debug!(%client_state_proof);

    let encoded_client_state_proof = target_client_module
        .encode_proof(client_state_proof)
        .await
        .map_err(json_rpc_error_to_queue_error)?;
    debug!(encoded_client_state_proof = %Hex(&encoded_client_state_proof));

    let consensus_state_proof = origin_chain_module
        .query_ibc_proof(
            origin_chain_proof_height,
            ClientConsensusStatePath {
                client_id: client_id.clone(),
                height: client_meta.height,
            }
            .into(),
        )
        .await
        .map_err(json_rpc_error_to_queue_error)?;
    debug!(%consensus_state_proof);

    let encoded_consensus_state_proof = target_client_module
        .encode_proof(consensus_state_proof)
        .await
        .map_err(json_rpc_error_to_queue_error)?;
    debug!(encoded_consensus_state_proof = %Hex(&encoded_consensus_state_proof));

    Ok(ConnectionHandshakeStateAndProofs {
        connection_state,
        encoded_client_state: reencoded_client_state,
        encoded_client_state_proof,
        encoded_consensus_state_proof,
        encoded_connection_state_proof,
        consensus_height: client_meta.height,
    })
}

struct ConnectionHandshakeStateAndProofs {
    connection_state: ConnectionEnd,
    /// The raw client state, exactly as stored in the counterparty's state.
    encoded_client_state: Vec<u8>,
    encoded_client_state_proof: Vec<u8>,
    encoded_consensus_state_proof: Vec<u8>,
    encoded_connection_state_proof: Vec<u8>,
    consensus_height: Height,
}
