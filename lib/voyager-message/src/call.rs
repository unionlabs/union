use enumorph::Enumorph;
use jsonrpsee::{core::RpcResult, types::ErrorObject};
use macros::model;
use serde::de::DeserializeOwned;
use serde_json::Value;
use serde_utils::Hex;
use tracing::{debug, error, info, instrument, trace};
use unionlabs::{
    bytes::Bytes,
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
        CommitmentPath, ConnectionPath,
    },
    id::{ClientId, ConnectionId},
    traits::Member,
    QueryHeight, DELAY_PERIOD,
};
use voyager_core::ClientType;
use voyager_vm::{call, data, defer, noop, now, seq, CallT, Op, QueueError};

#[cfg(doc)]
use crate::core::ClientInfo;
use crate::{
    core::{ChainId, IbcInterface},
    data::{IbcMessage, MsgCreateClientData, WithChainId},
    error_object_to_queue_error, json_rpc_error_to_queue_error,
    module::{ChainModuleClient, ClientModuleClient, ConsensusModuleClient, PluginClient},
    rpc::{json_rpc_error_to_error_object, VoyagerRpcServer},
    Context, PluginMessage, VoyagerMessage, FATAL_JSONRPC_ERROR_CODE,
};

#[model]
#[derive(Enumorph)]
pub enum Call {
    FetchBlocks(FetchBlocks),

    FetchUpdateHeaders(FetchUpdateHeaders),

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

    Plugin(PluginMessage),
}

impl Call {
    #[allow(clippy::result_large_err)]
    pub fn as_plugin<T: DeserializeOwned>(self, plugin_name: impl AsRef<str>) -> Result<T, Self> {
        match self {
            Self::Plugin(plugin_message) => {
                plugin_message.downcast(plugin_name).map_err(Self::Plugin)
            }
            this => Err(this),
        }
    }
}

#[model]
pub struct FetchBlockRange {
    pub chain_id: ChainId<'static>,
    pub from_height: Height,
    pub to_height: Height,
}

/// Fetch blocks on a chain, starting at height `start_height`.
///
/// This represents a request for IBC events on a chain and must be
/// picked up by a plugin. If it is not handled by a plugin, this will
/// return with a fatal error.
///
/// # Implementor's Note
///
/// This message is intended to act as a "seed" to an infinite stream of
/// unfolding messages. For example, if this is queued with height 10,
/// the plugin message this is replaced with should fetch all events in
/// block 10 and then wait for block 11 (which would then wait for block
/// 12, etc). Due to differing behaviours between chains, this may not
/// be the exact implementation, but the semantics of the unfold should
/// still hold.
#[model]
pub struct FetchBlocks {
    pub chain_id: ChainId<'static>,
    pub start_height: Height,
}

/// Generate a client update for this module's client type.
///
/// This represents a request for a client update and must be picked up
/// by a plugin. If it is not handled by a plugin, this will return with
/// a fatal error.
///
/// # Implementor's Note
///
/// The returned [`Op`] ***MUST*** resolve to an [`OrderedHeaders`] data.
/// This is the entrypoint called when a client update is requested, and
/// is intended to be called in the queue of an
/// [`AggregateMsgUpdateClientsFromOrderedHeaders`] message, which will
/// be used to build the actual [`MsgUpdateClient`]s.
#[model]
pub struct FetchUpdateHeaders {
    pub chain_id: ChainId<'static>,
    pub counterparty_chain_id: ChainId<'static>,
    pub update_from: Height,
    pub update_to: Height,
}

/// Build a [`MsgCreateClient`] [`IbcMessage`].
#[model]
pub struct MakeMsgCreateClient {
    /// The chain to create the client on.
    pub chain_id: ChainId<'static>,
    /// The height of the counterparty that the client will trust. The
    /// `SelfClientState` and `SelfConsensusState` will be queried at this
    /// height.
    pub height: QueryHeight,
    #[serde(default, skip_serializing_if = "Value::is_null")]
    /// Additional metadata that will be passed to
    /// [`ClientModuleClient::encode_client_state`]. This field is analogous to
    /// [`ClientInfo::metadata`].
    pub metadata: Value,
    /// The chain to create a client of.
    pub counterparty_chain_id: ChainId<'static>,
    /// The IBC interface to create the client on.
    pub ibc_interface: IbcInterface<'static>,
    /// The type of client to create.
    pub client_type: ClientType<'static>,
}

#[model]
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

#[model]
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

#[model]
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

#[model]
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

#[model]
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

#[model]
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

#[model]
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

#[model]
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

#[model]
pub struct WaitForHeight {
    pub chain_id: ChainId<'static>,
    pub height: Height,
}

#[model]
pub struct WaitForHeightRelative {
    pub chain_id: ChainId<'static>,
    pub height: u64,
}

#[model]
pub struct WaitForTimestamp {
    pub chain_id: ChainId<'static>,
    /// THIS IS NANOSECONDS
    pub timestamp: i64,
}

/// Wait for the client `.client_id` on `.chain_id` to trust a height >=
/// `.height`.
#[model]
pub struct WaitForTrustedHeight {
    pub chain_id: ChainId<'static>,
    pub client_id: ClientId,
    pub height: Height,
}

impl CallT<VoyagerMessage> for Call {
    // #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn process(self, ctx: &Context) -> Result<Op<VoyagerMessage>, QueueError> {
        match self {
            Call::FetchBlocks(FetchBlocks {
                start_height,
                chain_id,
            }) => {
                let message = format!(
                    "fetch blocks request received for chain `{chain_id}` at height \
                    {start_height} but it was not picked up by a plugin"
                );

                error!(%message);

                Err(QueueError::Fatal(message.into()))
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

                Ok(voyager_vm::data(IbcMessage::from(MsgConnectionOpenAck {
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

                Ok(voyager_vm::data(IbcMessage::from(
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
                let origin_channel = ctx
                    .rpc_server
                    .query_channel(
                        origin_chain_id.clone(),
                        QueryHeight::Specific(origin_chain_proof_height),
                        event.port_id.clone(),
                        event.channel_id.clone(),
                    )
                    .await
                    .map_err(error_object_to_queue_error)?;

                let proof_init = ctx
                    .rpc_server
                    .query_ibc_proof(
                        &origin_chain_id,
                        origin_chain_proof_height,
                        ChannelEndPath {
                            port_id: event.port_id.clone(),
                            channel_id: event.channel_id.clone(),
                        }
                        .into(),
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
                        ordering: origin_channel
                            .state
                            .ok_or(QueueError::Fatal("channel must exist".into()))?
                            .ordering,
                        counterparty: channel::counterparty::Counterparty {
                            port_id: event.port_id,
                            channel_id: Some(event.channel_id),
                        },
                        connection_hops: vec![event.connection.counterparty.connection_id.unwrap()],
                        version: event.version.clone(),
                        upgrade_sequence: 0,
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

                Ok(voyager_vm::data(IbcMessage::from(MsgChannelOpenConfirm {
                    port_id: channel_open_ack_event.counterparty_port_id,
                    channel_id: channel_open_ack_event.counterparty_channel_id,
                    proof_ack: encoded_proof_ack,
                    proof_height: origin_chain_proof_height,
                })))
            }

            Call::MakeMsgRecvPacket(msg) => make_msg_recv_packet(ctx, msg).await,

            Call::MakeMsgAcknowledgement(msg) => make_msg_acknowledgement(ctx, msg).await,

            Call::FetchUpdateHeaders(FetchUpdateHeaders {
                chain_id,
                counterparty_chain_id,
                update_from,
                update_to,
            }) => {
                let message = format!(
                    "client update request received for a client on {counterparty_chain_id} \
                    tracking {chain_id} from height {update_from} to {update_to} but it was \
                    not picked up by a plugin"
                );

                error!(%message);

                Err(QueueError::Fatal(message.into()))
            }

            Call::MakeMsgCreateClient(MakeMsgCreateClient {
                chain_id,
                height,
                metadata,
                counterparty_chain_id,
                client_type,
                ibc_interface,
            }) => {
                make_msg_create_client(
                    ctx,
                    counterparty_chain_id,
                    height,
                    chain_id,
                    client_type,
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

                if chain_height.revision() != height.revision() {
                    return Err(QueueError::Fatal(
                        format!(
                            "revision number mismatch, \
                            chain_height: {chain_height}, height: {height}"
                        )
                        .into(),
                    ));
                }

                debug!("latest height is {chain_height}, waiting for {height}");

                if chain_height.height() >= height.height() {
                    Ok(noop())
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
                    height: Height::new_with_revision(
                        chain_height.revision(),
                        chain_height.height() + height,
                    ),
                }))
            }

            Call::WaitForTimestamp(WaitForTimestamp {
                chain_id,
                timestamp,
            }) => {
                let chain_timestamp = ctx
                    .rpc_server
                    .query_latest_timestamp(&chain_id)
                    .await
                    .map_err(error_object_to_queue_error)?;

                if chain_timestamp >= timestamp {
                    info!(%chain_id, %timestamp, %chain_timestamp, "timestamp reached");
                    Ok(noop())
                } else {
                    debug!(%chain_id, %timestamp, %chain_timestamp, "timestamp not yet reached");
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
                height,
            }) => {
                let client_state = ctx
                    .rpc_server
                    .modules()
                    .map_err(error_object_to_queue_error)?
                    .chain_module(&chain_id)?
                    .query_raw_unfinalized_trusted_client_state(client_id.clone())
                    .await
                    .map_err(json_rpc_error_to_queue_error)?;

                let trusted_client_state_meta = ctx
                    .rpc_server
                    .decode_client_state_meta(
                        &client_state.client_type,
                        &client_state.ibc_interface,
                        client_state.bytes,
                    )
                    .await
                    .map_err(error_object_to_queue_error)?;

                if trusted_client_state_meta.height.height() >= height.height() {
                    debug!(
                        "client height reached ({} >= {})",
                        trusted_client_state_meta.height, height
                    );

                    Ok(noop())
                } else {
                    Ok(seq([
                        // REVIEW: Defer until `now + counterparty_chain.block_time()`? Would
                        // require a new method on chain
                        defer(now() + 1),
                        call(WaitForTrustedHeight {
                            chain_id,
                            client_id,
                            height,
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
        %origin_chain_id,
        %origin_chain_proof_height,
        %target_chain_id,
        %send_packet_event.packet.sequence,
        %send_packet_event.packet.source_channel.port_id,
        %send_packet_event.packet.source_channel.channel_id,
        %send_packet_event.packet.destination_channel.port_id,
        %send_packet_event.packet.destination_channel.channel_id,
        %send_packet_event.packet.channel_ordering,
        %send_packet_event.packet.timeout_height,
        %send_packet_event.packet.timeout_timestamp,
    )
)]
async fn make_msg_recv_packet(
    ctx: &Context,
    MakeMsgRecvPacket {
        origin_chain_id,
        origin_chain_proof_height,
        target_chain_id,
        send_packet_event,
    }: MakeMsgRecvPacket,
) -> Result<Op<VoyagerMessage>, QueueError> {
    let target_chain_latest_height = ctx
        .rpc_server
        .query_latest_height(&target_chain_id)
        .await
        .map_err(error_object_to_queue_error)?;

    let commitment = ctx
        .rpc_server
        .query_receipt(
            target_chain_id.clone(),
            QueryHeight::Specific(target_chain_latest_height),
            send_packet_event.packet.destination_channel.port_id.clone(),
            send_packet_event
                .packet
                .destination_channel
                .channel_id
                .clone(),
            send_packet_event.packet.sequence,
        )
        .await
        .map_err(error_object_to_queue_error)?
        .state;

    if commitment {
        info!("packet already received on the target chain");
        return Ok(noop());
    }

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

    Ok(voyager_vm::data(IbcMessage::from(MsgRecvPacket {
        packet: channel::packet::Packet {
            sequence: send_packet_event.packet.sequence,
            source_port: send_packet_event.packet.source_channel.port_id,
            source_channel: send_packet_event.packet.source_channel.channel_id,
            destination_port: send_packet_event.packet.destination_channel.port_id,
            destination_channel: send_packet_event.packet.destination_channel.channel_id,
            data: send_packet_event.packet_data,
            timeout_height: send_packet_event.packet.timeout_height,
            timeout_timestamp: send_packet_event.packet.timeout_timestamp,
        },
        proof_commitment: encoded_proof_commitment,
        proof_height: origin_chain_proof_height,
    })))
}

#[instrument(
    skip_all,
    fields(
        %origin_chain_id,
        %origin_chain_proof_height,
        %target_chain_id,
        %write_acknowledgement_event.packet.sequence,
        %write_acknowledgement_event.packet.source_channel.port_id,
        %write_acknowledgement_event.packet.source_channel.channel_id,
        %write_acknowledgement_event.packet.destination_channel.port_id,
        %write_acknowledgement_event.packet.destination_channel.channel_id,
        %write_acknowledgement_event.packet.channel_ordering,
        %write_acknowledgement_event.packet.timeout_height,
        %write_acknowledgement_event.packet.timeout_timestamp,
    )
)]
async fn make_msg_acknowledgement(
    ctx: &Context,
    MakeMsgAcknowledgement {
        origin_chain_id,
        origin_chain_proof_height,
        target_chain_id,
        write_acknowledgement_event,
    }: MakeMsgAcknowledgement,
) -> Result<Op<VoyagerMessage>, QueueError> {
    let target_chain_latest_height = ctx
        .rpc_server
        .query_latest_height(&target_chain_id)
        .await
        .map_err(error_object_to_queue_error)?;

    let commitment = ctx
        .rpc_server
        .query_commitment(
            target_chain_id.clone(),
            QueryHeight::Specific(target_chain_latest_height),
            write_acknowledgement_event
                .packet
                .source_channel
                .port_id
                .clone(),
            write_acknowledgement_event
                .packet
                .source_channel
                .channel_id
                .clone(),
            write_acknowledgement_event.packet.sequence,
        )
        .await
        .map_err(error_object_to_queue_error)?
        .state;

    if commitment.is_none() {
        info!("packet already acknowledged on the target chain");
        return Ok(noop());
    }

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

    Ok(voyager_vm::data(IbcMessage::from(MsgAcknowledgement {
        packet: channel::packet::Packet {
            sequence: write_acknowledgement_event.packet.sequence,
            source_port: write_acknowledgement_event.packet.source_channel.port_id,
            source_channel: write_acknowledgement_event.packet.source_channel.channel_id,
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
async fn make_msg_create_client(
    ctx: &Context,
    counterparty_chain_id: ChainId<'static>,
    height: QueryHeight,
    chain_id: ChainId<'static>,
    client_type: ClientType<'static>,
    ibc_interface: IbcInterface<'_>,
    metadata: Value,
) -> Result<Op<VoyagerMessage>, QueueError> {
    let height = ctx
        .rpc_server
        .query_latest_height(&counterparty_chain_id)
        .await
        .map_err(error_object_to_queue_error)?;

    let counterparty_consensus_module = ctx
        .rpc_server
        .modules()
        .map_err(error_object_to_queue_error)?
        .consensus_module(&counterparty_chain_id)?;

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

    let consensus_type = ctx
        .rpc_server
        .modules()
        .map_err(error_object_to_queue_error)?
        .chain_consensus_type(&counterparty_chain_id)?;

    let client_consensus_type = ctx
        .rpc_server
        .modules()
        .map_err(error_object_to_queue_error)?
        .client_consensus_type(&client_type)?;

    if client_consensus_type != consensus_type {
        return Err(QueueError::Fatal(
            format!(
                "attempted to create a {client_type} client on \
                {chain_id} tracking {counterparty_chain_id}, but \
                the consensus of that chain ({consensus_type}) is \
                not verifiable by a client of type {client_type} \
                (which instead verifies {client_consensus_type})."
            )
            .into(),
        ));
    }

    let client_module = ctx
        .rpc_server
        .modules()
        .map_err(error_object_to_queue_error)?
        .client_module(&client_type, &ibc_interface)?;

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
            client_type: client_type.clone(),
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
    let client_state = ctx
        .rpc_server
        .query_client_state(
            origin_chain_id.clone(),
            origin_chain_proof_height.into(),
            client_id.clone(),
        )
        .await?
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
            client_state.clone(),
        )
        .await?;

    debug!(
        %client_meta.height,
        %client_meta.chain_id,
    );

    let reencoded_client_state = ctx
        .rpc_server
        .modules()?
        .client_module(
            &target_client_info.client_type,
            &target_client_info.ibc_interface,
        )?
        .reencode_counterparty_client_state(client_state.clone(), origin_client_info.client_type)
        .await
        .map_err(json_rpc_error_to_error_object)?;

    debug!(%reencoded_client_state);

    // the connection end as stored by the origin chain after open_init/try
    let connection_state = ctx
        .rpc_server
        .query_connection(
            origin_chain_id.clone(),
            origin_chain_proof_height.into(),
            connection_id.clone(),
        )
        .await?
        .state
        .ok_or(ErrorObject::owned(
            FATAL_JSONRPC_ERROR_CODE,
            "connection must exist",
            None::<()>,
        ))?;
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
    encoded_client_state: Bytes,
    encoded_client_state_proof: Bytes,
    encoded_consensus_state_proof: Bytes,
    encoded_connection_state_proof: Bytes,
    consensus_height: Height,
}
