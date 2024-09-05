#![allow(unused_imports)] // TODO: Remove

use enumorph::Enumorph;
use jsonrpsee::{
    core::RpcResult,
    types::{ErrorObject, ErrorObjectOwned},
};
use macros::apply;
use queue_msg::{
    call, conc, data, defer, now, promise, queue_msg, seq, HandleCall, Op, QueueError,
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
        client::{height::Height, msg_create_client::MsgCreateClient},
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
    error_object_to_queue_error, json_rpc_error_to_queue_error,
    plugin::{ChainModuleClient, ClientModuleClient, ConsensusModuleClient, PluginModuleClient},
    rpc::{json_rpc_error_to_rpc_error, VoyagerRpcServer},
    top_level_identifiable_enum, ChainId, Context, IbcInterface, PluginMessage, VoyagerMessage,
    FATAL_JSONRPC_ERROR_CODE,
};

#[apply(top_level_identifiable_enum)]
#[queue_msg]
#[derive(Enumorph)]
pub enum Call<C = serde_json::Value> {
    FetchBlock(FetchBlock),
    FetchBlockRange(FetchBlockRange),

    UnfinalizedTrustedClientState(FetchUnfinalizedTrustedClientState),

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

    WaitForHeight(WaitForHeight),
    WaitForHeightRelative(WaitForHeightRelative),
    WaitForTimestamp(WaitForTimestamp),
    WaitForTrustedHeight(WaitForTrustedHeight),

    Plugin(PluginMessage<C>),
}

#[queue_msg]
pub struct FetchBlockRange {
    pub chain_id: ChainId<'static>,
    pub from_height: Height,
    pub to_height: Height,
}

#[queue_msg]
pub struct FetchBlock {
    pub chain_id: ChainId<'static>,
    pub height: Height,
}

#[queue_msg]
pub struct FetchSelfClientState {
    pub chain_id: ChainId<'static>,
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
    pub chain_id: ChainId<'static>,
    pub at: QueryHeight,
    /// The counterparty IBC interface that the state must be encoded for.
    pub ibc_interface: IbcInterface<'static>,
}

// TODO: This should have a height field
#[queue_msg]
pub struct FetchClientInfo {
    pub chain_id: ChainId<'static>,
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
    pub chain_id: ChainId<'static>,
    pub at: Height,
    pub path: Path,
}

#[queue_msg]
pub struct FetchState {
    pub chain_id: ChainId<'static>,
    pub at: QueryHeight,
    pub path: Path,
}

#[queue_msg]
pub struct FetchUpdateHeaders {
    pub chain_id: ChainId<'static>,
    pub counterparty_chain_id: ChainId<'static>,
    pub update_from: Height,
    pub update_to: Height,
}

#[queue_msg]
pub struct FetchLatestHeight {
    pub chain_id: ChainId<'static>,
}

#[queue_msg]
pub struct FetchUnfinalizedTrustedClientState {
    pub chain_id: ChainId<'static>,
    pub client_id: ClientId,
}

/// Build a [`MsgCreateClient`] [`IbcMessage`].
#[queue_msg]
pub struct MakeMsgCreateClient {
    /// The chain to create the client on.
    pub chain_id: ChainId<'static>,
    /// The height of the counterparty that the client will trust. The
    /// [`SelfClientState`] and [`SelfConsensusState`] will be queried at this
    /// height.
    pub height: QueryHeight,
    #[serde(default, skip_serializing_if = "Value::is_null")]
    /// Additional metadata that will be passed to
    /// [`ClientModulePlugin::encode_client_state`]. This field is analogous to
    /// [`ClientInfo::metadata`].
    pub metadata: Value,
    /// The chain to create a client of.
    pub counterparty_chain_id: ChainId<'static>,
    /// The IBC interface to create the client on.
    pub ibc_interface: IbcInterface<'static>,
}

#[queue_msg]
pub struct MakeMsgConnectionOpenTry {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: ChainId<'static>,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: ChainId<'static>,
    /// The original event that was emitted on the origin chain.
    pub connection_open_init_event: crate::data::ConnectionOpenInit,
}

#[queue_msg]
pub struct MakeMsgConnectionOpenAck {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: ChainId<'static>,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: ChainId<'static>,
    /// The original event that was emitted on the origin chain.
    pub connection_open_try_event: crate::data::ConnectionOpenTry,
}

#[queue_msg]
pub struct MakeMsgConnectionOpenConfirm {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: ChainId<'static>,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: ChainId<'static>,
    /// The original event that was emitted on the origin chain.
    pub connection_open_ack_event: crate::data::ConnectionOpenAck,
}

#[queue_msg]
pub struct MakeMsgChannelOpenTry {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: ChainId<'static>,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: ChainId<'static>,
    /// The original event that was emitted on the origin chain.
    pub channel_open_init_event: crate::data::ChannelOpenInit,
}

#[queue_msg]
pub struct MakeMsgChannelOpenAck {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: ChainId<'static>,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: ChainId<'static>,
    /// The original event that was emitted on the origin chain.
    pub channel_open_try_event: crate::data::ChannelOpenTry,
}

#[queue_msg]
pub struct MakeMsgChannelOpenConfirm {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: ChainId<'static>,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: ChainId<'static>,
    /// The original event that was emitted on the origin chain.
    pub channel_open_ack_event: crate::data::ChannelOpenAck,
}

#[queue_msg]
pub struct MakeMsgRecvPacket {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: ChainId<'static>,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: ChainId<'static>,
    /// The original event that was emitted on the origin chain.
    pub send_packet_event: crate::data::SendPacket,
}

#[queue_msg]
pub struct MakeMsgAcknowledgement {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: ChainId<'static>,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: ChainId<'static>,
    /// The original event that was emitted on the origin chain.
    pub write_acknowledgement_event: crate::data::WriteAcknowledgement,
}

#[queue_msg]
pub struct WaitForHeight {
    pub chain_id: ChainId<'static>,
    pub height: Height,
}

#[queue_msg]
pub struct WaitForHeightRelative {
    pub chain_id: ChainId<'static>,
    pub height: u64,
}

#[queue_msg]
pub struct WaitForTimestamp {
    pub chain_id: ChainId<'static>,
    pub timestamp: i64,
}

/// Wait for the client `.client_id` on `Hc` to trust a height >= `.height`,
/// returning the counterparty's client state at that height when it's reached.
#[queue_msg]
pub struct WaitForTrustedHeight {
    pub chain_id: ChainId<'static>,
    /// The id of the client on `Hc` who's [`ClientState::height()`] we're
    /// waiting to be >= `.height`.
    pub client_id: ClientId,
    /// The id of the counterparty client on `Tr`, who's state will be fetched
    /// at [`ClientState::height()`] when `.client_id` on `Hc` trusts a height
    /// >= `.height`.
    pub counterparty_client_id: ClientId,
    pub counterparty_chain_id: ChainId<'static>,
    pub height: Height,
}

impl<D: Member, C: Member, Cb: Member> HandleCall<VoyagerMessage<D, C, Cb>> for Call<C> {
    // #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn handle(self, ctx: &Context) -> Result<Op<VoyagerMessage<D, C, Cb>>, QueueError> {
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
                    ctx.chain_module(&chain_id)
                        .map_err(error_object_to_queue_error)?
                        .fetch_block_range(from_height, to_height)
                        .await
                        .map_err(json_rpc_error_to_queue_error)?,
                    call(FetchBlock {
                        chain_id,
                        height: to_height,
                    }),
                ]))
            }

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
                .await
                .map_err(error_object_to_queue_error)?;

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
                .await
                .map_err(error_object_to_queue_error)?;

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
                // info of the client on the target chain that will verify the storage
                // proofs
                let target_client_info = ctx
                    .rpc_server
                    // counterparty_client_id from open_try is the client on the target chain
                    .client_info(
                        &target_chain_id,
                        connection_open_ack_event.counterparty_client_id.clone(),
                    )
                    .await
                    .map_err(error_object_to_queue_error)?;

                // proof of connection_state, encoded for the client on the target chain
                // this is encoded via the client module for the client on the origin chain
                // (the chain the event was emitted on)
                let connection_proof = ctx
                    .rpc_server
                    .encode_proof(
                        &target_client_info.client_type,
                        &target_client_info.ibc_interface,
                        ctx.rpc_server
                            .query_ibc_proof(
                                &origin_chain_id,
                                origin_chain_proof_height,
                                ConnectionPath {
                                    connection_id: connection_open_ack_event.connection_id.clone(),
                                }
                                .into(),
                            )
                            .await
                            .map_err(error_object_to_queue_error)?
                            .proof,
                    )
                    .await
                    .map_err(error_object_to_queue_error)?;

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
                let origin_channel_path = ChannelEndPath {
                    port_id: event.port_id.clone(),
                    channel_id: event.channel_id.clone(),
                };

                let origin_channel = ctx
                    .rpc_server
                    .query_ibc_state_typed(
                        &origin_chain_id,
                        origin_chain_proof_height,
                        origin_channel_path.clone(),
                    )
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                let proof_init = ctx
                    .rpc_server
                    .query_ibc_proof(
                        &origin_chain_id,
                        origin_chain_proof_height,
                        origin_channel_path.into(),
                    )
                    .await
                    .map_err(error_object_to_queue_error)?;

                let client_info = ctx
                    .rpc_server
                    .client_info(&target_chain_id, event.connection.counterparty.client_id)
                    .await
                    .map_err(error_object_to_queue_error)?;

                let encoded_proof_init = ctx
                    .rpc_server
                    .encode_proof(
                        &client_info.client_type,
                        &client_info.ibc_interface,
                        proof_init.proof,
                    )
                    .await
                    .map_err(error_object_to_queue_error)?;

                Ok(data(IbcMessage::from(MsgChannelOpenTry {
                    port_id: event.counterparty_port_id,
                    channel: Channel {
                        state: channel::state::State::Tryopen,
                        ordering: origin_channel.state.ordering,
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
                channel_open_try_event,
            }) => {
                let origin_channel_path = ChannelEndPath {
                    port_id: channel_open_try_event.port_id.clone(),
                    channel_id: channel_open_try_event.channel_id.clone(),
                };

                let proof_try = ctx
                    .rpc_server
                    .query_ibc_proof(
                        &origin_chain_id,
                        origin_chain_proof_height,
                        origin_channel_path.into(),
                    )
                    .await
                    .map_err(error_object_to_queue_error)?;

                let client_info = ctx
                    .rpc_server
                    .client_info(
                        &target_chain_id,
                        channel_open_try_event.connection.counterparty.client_id,
                    )
                    .await
                    .map_err(error_object_to_queue_error)?;

                let encoded_proof_try = ctx
                    .rpc_server
                    .encode_proof(
                        &client_info.client_type,
                        &client_info.ibc_interface,
                        proof_try.proof,
                    )
                    .await
                    .map_err(error_object_to_queue_error)?;

                Ok(data(IbcMessage::from(MsgChannelOpenAck {
                    port_id: channel_open_try_event.counterparty_port_id,
                    channel_id: channel_open_try_event.counterparty_channel_id,
                    counterparty_channel_id: channel_open_try_event.channel_id,
                    counterparty_version: channel_open_try_event.version,
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
                let origin_channel_path = ChannelEndPath {
                    port_id: channel_open_ack_event.port_id.clone(),
                    channel_id: channel_open_ack_event.channel_id.clone(),
                };

                let proof_ack = ctx
                    .rpc_server
                    .query_ibc_proof(
                        &origin_chain_id,
                        origin_chain_proof_height,
                        origin_channel_path.into(),
                    )
                    .await
                    .map_err(error_object_to_queue_error)?;

                let client_info = ctx
                    .rpc_server
                    .client_info(
                        &target_chain_id,
                        channel_open_ack_event.connection.counterparty.client_id,
                    )
                    .await
                    .map_err(error_object_to_queue_error)?;

                let encoded_proof_ack = ctx
                    .rpc_server
                    .encode_proof(
                        &client_info.client_type,
                        &client_info.ibc_interface,
                        proof_ack.proof,
                    )
                    .await
                    .map_err(error_object_to_queue_error)?;

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
                let proof_commitment = ctx
                    .rpc_server
                    .query_ibc_proof(
                        &origin_chain_id,
                        origin_chain_proof_height,
                        CommitmentPath {
                            port_id: send_packet_event.packet.source_channel.port_id.clone(),
                            channel_id: send_packet_event.packet.source_channel.channel_id.clone(),
                            sequence: send_packet_event.packet.sequence,
                        }
                        .into(),
                    )
                    .await
                    .map_err(error_object_to_queue_error)?
                    .proof;

                let client_info = ctx
                    .rpc_server
                    .client_info(
                        &target_chain_id,
                        send_packet_event
                            .packet
                            .destination_channel
                            .connection
                            .client_id,
                    )
                    .await
                    .map_err(error_object_to_queue_error)?;

                let encoded_proof_commitment = ctx
                    .rpc_server
                    .encode_proof(
                        &client_info.client_type,
                        &client_info.ibc_interface,
                        proof_commitment,
                    )
                    .await
                    .map_err(error_object_to_queue_error)?;

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
                    let proof_acked = ctx
                        .rpc_server
                        .query_ibc_proof(
                            &origin_chain_id,
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
                        .map_err(error_object_to_queue_error)?
                        .proof;

                    let client_info = ctx
                        .rpc_server
                        .client_info(
                            &target_chain_id,
                            write_acknowledgement_event
                                .packet
                                .source_channel
                                .connection
                                .client_id,
                        )
                        .await
                        .map_err(error_object_to_queue_error)?;

                    let encoded_proof_acked = ctx
                        .rpc_server
                        .encode_proof(
                            &client_info.client_type,
                            &client_info.ibc_interface,
                            proof_acked,
                        )
                        .await
                        .map_err(error_object_to_queue_error)?;

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
                counterparty_chain_id,
                update_from,
                update_to,
            }) => ctx
                .consensus_module(&chain_id)
                .map_err(error_object_to_queue_error)?
                .fetch_update_headers(update_from, update_to, counterparty_chain_id)
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
            Call::WaitForHeight(WaitForHeight { chain_id, height }) => {
                let chain_height = ctx
                    .rpc_server
                    .query_latest_height(&chain_id)
                    .await
                    .map_err(error_object_to_queue_error)?;

                if chain_height.revision_number != height.revision_number {
                    return Err(QueueError::Fatal(
                        format!(
                            "revision number mismatch, \
                            chain_height: {chain_height}, height: {height}"
                        )
                        .into(),
                    ));
                }

                debug!("latest height is {chain_height}, waiting for {height}");

                if chain_height.revision_height >= height.revision_height {
                    Ok(data(LatestHeight {
                        chain_id,
                        height: chain_height,
                    }))
                } else {
                    Ok(seq([
                        defer(now() + 1),
                        call(WaitForHeight { chain_id, height }),
                    ]))
                }
            }
            // REVIEW: Perhaps remove, unused
            Call::WaitForHeightRelative(WaitForHeightRelative { chain_id, height }) => {
                let chain_height = ctx
                    .rpc_server
                    .query_latest_height(&chain_id)
                    .await
                    .map_err(error_object_to_queue_error)?;

                Ok(call(WaitForHeight {
                    chain_id,
                    height: Height {
                        revision_number: chain_height.revision_number,
                        revision_height: chain_height.revision_height + height,
                    },
                }))
            }

            Call::WaitForTimestamp(WaitForTimestamp {
                chain_id,
                timestamp,
            }) => {
                let chain_ts = ctx
                    .rpc_server
                    .query_latest_timestamp(&chain_id)
                    .await
                    .map_err(error_object_to_queue_error)?;

                if chain_ts >= timestamp {
                    // TODO: Figure out a way to fetch a height at a specific timestamp
                    Ok(data(LatestHeight {
                        height: ctx
                            .rpc_server
                            .query_latest_height(&chain_id)
                            .await
                            .map_err(error_object_to_queue_error)?,
                        chain_id,
                    }))
                } else {
                    Ok(seq([
                        // REVIEW: Defer until `now + chain.block_time()`? Would require a new
                        // method on chain
                        defer(now() + 1),
                        call(WaitForTimestamp {
                            chain_id,
                            timestamp,
                        }),
                    ]))
                }
            }

            Call::WaitForTrustedHeight(WaitForTrustedHeight {
                chain_id,
                client_id,
                counterparty_client_id,
                counterparty_chain_id,
                height,
            }) => {
                let client_state = ctx
                    .chain_module::<D, C, Cb>(&chain_id)
                    .map_err(error_object_to_queue_error)?
                    .query_raw_unfinalized_trusted_client_state(client_id.clone())
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                let trusted_client_state_meta = ctx
                    .rpc_server
                    .decode_client_state_meta(
                        &client_state.client_type,
                        &client_state.ibc_interface,
                        client_state.bytes.into(),
                    )
                    .await
                    .map_err(error_object_to_queue_error)?;

                if trusted_client_state_meta.height.revision_height >= height.revision_height {
                    debug!(
                        "client height reached ({} >= {})",
                        trusted_client_state_meta.height, height
                    );

                    // the height has been reached, fetch the counterparty client state on `Tr` at
                    // the trusted height
                    let crate::rpc::IbcState {
                        chain_id,
                        path,
                        height,
                        state,
                    } = ctx
                        .rpc_server
                        .query_ibc_state_typed(
                            &counterparty_chain_id,
                            trusted_client_state_meta.height,
                            ClientStatePath {
                                client_id: counterparty_client_id.clone(),
                            },
                        )
                        .await
                        .map_err(json_rpc_error_to_queue_error)?;

                    Ok(data(IbcState {
                        chain_id,
                        path,
                        height,
                        state,
                    }))
                } else {
                    Ok(seq([
                        // REVIEW: Defer until `now + counterparty_chain.block_time()`? Would
                        // require a new method on chain
                        defer(now() + 1),
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
                .call(message)
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
async fn make_msg_create_client<D: Member, C: Member, Cb: Member>(
    ctx: &Context,
    counterparty_chain_id: ChainId<'static>,
    height: QueryHeight,
    chain_id: ChainId<'static>,
    ibc_interface: IbcInterface<'_>,
    metadata: Value,
) -> Result<Op<VoyagerMessage<D, C, Cb>>, QueueError> {
    let height = ctx
        .rpc_server
        .query_latest_height(&counterparty_chain_id)
        .await
        .map_err(error_object_to_queue_error)?;

    let counterparty_consensus_module = ctx
        .consensus_module::<Value, Value, Value>(&counterparty_chain_id)
        .map_err(error_object_to_queue_error)?;

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

    let client_module = ctx
        .client_module::<Value, Value, Value>(&client_type, &ibc_interface)
        .map_err(error_object_to_queue_error)?;

    Ok(data(WithChainId {
        chain_id,
        message: IbcMessage::from(MsgCreateClientData {
            msg: MsgCreateClient {
                client_state: client_module
                    .encode_client_state(self_client_state, metadata)
                    .await
                    .map_err(json_rpc_error_to_queue_error)?
                    .0,
                consensus_state: client_module
                    .encode_consensus_state(self_consensus_state)
                    .await
                    .map_err(json_rpc_error_to_queue_error)?
                    .0,
            },
            client_type,
        }),
    }))
}

/// Used to fetch and construct the state and proofs for
/// MsgConnectionOpenTry/Ack.
#[instrument(
    skip_all,
    fields(
        %origin_chain_id,
        %target_chain_id,
        %client_id,
        %counterparty_client_id,
        %connection_id,
        %origin_chain_proof_height,
    )
)]
async fn mk_connection_handshake_state_and_proofs(
    ctx: &Context,
    origin_chain_id: ChainId<'static>,
    target_chain_id: ChainId<'static>,
    client_id: ClientId,
    counterparty_client_id: ClientId,
    connection_id: ConnectionId,
    origin_chain_proof_height: Height,
) -> RpcResult<ConnectionHandshakeStateAndProofs> {
    // info of the client on the target chain that will verify the storage
    // proofs
    let target_client_info = ctx
        .rpc_server
        // counterparty_client_id from open_init/try is the client on the target chain
        .client_info(&target_chain_id, counterparty_client_id.clone())
        .await?;

    debug!(
        %counterparty_client_id,
        %target_client_info.client_type,
        %target_client_info.ibc_interface,
        %target_client_info.metadata,
    );

    // info of the client on the origin chain, this is used to decode the stored
    // client state
    let origin_client_info = ctx
        .rpc_server
        // client_id from open_init/try is the client on the origin chain
        .client_info(&origin_chain_id, client_id.clone())
        .await?;

    debug!(
        %client_id,
        %origin_client_info.client_type,
        %origin_client_info.ibc_interface,
        %origin_client_info.metadata,
    );

    // client state of the destination on the source
    let client_state_path = ClientStatePath {
        client_id: client_id.clone(),
    };
    let client_state = ctx
        .rpc_server
        .query_ibc_state_typed(
            &origin_chain_id,
            origin_chain_proof_height,
            client_state_path,
        )
        .await
        .map_err(json_rpc_error_to_rpc_error)?
        .state;

    debug!(%client_state);

    // the client state meta of the target chain on the origin chain, that
    // contains a trusted height of the destination TODO: maybe assert the
    // chain_id is as expected?
    let client_meta = ctx
        .rpc_server
        .decode_client_state_meta(
            &origin_client_info.client_type,
            &origin_client_info.ibc_interface,
            client_state.0.clone(),
        )
        .await?;

    debug!(
        %client_meta.height,
        %client_meta.chain_id,
    );

    let reencoded_client_state = ctx
        .client_module::<Value, Value, Value>(
            &target_client_info.client_type,
            &target_client_info.ibc_interface,
        )?
        .reencode_counterparty_client_state(client_state.clone(), origin_client_info.client_type)
        .await
        .map_err(json_rpc_error_to_rpc_error)?;

    debug!(%reencoded_client_state);

    // the connection end as stored by the origin chain after open_init/try
    let connection_state = ctx
        .rpc_server
        .query_ibc_state_typed(
            &origin_chain_id,
            origin_chain_proof_height,
            ConnectionPath {
                connection_id: connection_id.clone(),
            },
        )
        .await
        .map_err(json_rpc_error_to_rpc_error)?
        .state;
    debug!(
        connection_state = %serde_json::to_string(&connection_state).unwrap(),
    );

    // proof of connection_state, encoded for the client on the target chain
    let connection_proof = ctx
        .rpc_server
        .query_ibc_proof(
            &origin_chain_id,
            origin_chain_proof_height,
            ConnectionPath {
                connection_id: connection_id.clone(),
            }
            .into(),
        )
        .await?
        .proof;
    debug!(%connection_proof);

    let encoded_connection_state_proof = ctx
        .rpc_server
        .encode_proof(
            &target_client_info.client_type,
            &target_client_info.ibc_interface,
            connection_proof,
        )
        .await?;
    debug!(encoded_connection_state_proof = %Hex(&encoded_connection_state_proof));

    let client_state_proof = ctx
        .rpc_server
        .query_ibc_proof(
            &origin_chain_id,
            origin_chain_proof_height,
            ClientStatePath {
                client_id: client_id.clone(),
            }
            .into(),
        )
        .await?
        .proof;
    debug!(%client_state_proof);

    let encoded_client_state_proof = ctx
        .rpc_server
        .encode_proof(
            &target_client_info.client_type,
            &target_client_info.ibc_interface,
            client_state_proof,
        )
        .await?;
    debug!(encoded_client_state_proof = %Hex(&encoded_client_state_proof));

    let consensus_state_proof = ctx
        .rpc_server
        .query_ibc_proof(
            &origin_chain_id,
            origin_chain_proof_height,
            ClientConsensusStatePath {
                client_id: client_id.clone(),
                height: client_meta.height,
            }
            .into(),
        )
        .await?
        .proof;
    debug!(%consensus_state_proof);

    let encoded_consensus_state_proof = ctx
        .rpc_server
        .encode_proof(
            &target_client_info.client_type,
            &target_client_info.ibc_interface,
            consensus_state_proof,
        )
        .await?;
    debug!(encoded_consensus_state_proof = %Hex(&encoded_consensus_state_proof));

    Ok(ConnectionHandshakeStateAndProofs {
        connection_state,
        encoded_client_state: reencoded_client_state.0,
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
