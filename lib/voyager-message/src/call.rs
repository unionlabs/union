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
    DELAY_PERIOD,
};
use voyager_core::{ClientType, IbcVersionId, QueryHeight};
use voyager_vm::{call, data, defer, noop, now, seq, CallT, Op, QueueError};

#[cfg(doc)]
use crate::core::ClientInfo;
use crate::{
    core::{ChainId, IbcInterface},
    data::{IbcMessage, MsgCreateClientData, WithChainId},
    error_object_to_queue_error, json_rpc_error_to_queue_error,
    module::{ChainModuleClient, ClientModuleClient, ConsensusModuleClient, PluginClient},
    rpc::{json_rpc_error_to_error_object, VoyagerRpcServer},
    Context, IbcSpec, PluginMessage, VoyagerMessage, FATAL_JSONRPC_ERROR_CODE,
};

#[model]
#[derive(Enumorph)]
pub enum Call {
    FetchBlocks(FetchBlocks),

    FetchUpdateHeaders(FetchUpdateHeaders),

    MakeMsgCreateClient(MakeMsgCreateClient),

    Version(VersionMessage),

    WaitForHeight(WaitForHeight),
    WaitForTimestamp(WaitForTimestamp),
    WaitForTrustedHeight(WaitForTrustedHeight),

    Plugin(PluginMessage),
}

#[model]
pub struct VersionMessage {
    pub ibc_version_id: IbcVersionId<'static>,
    pub data: Value,
}

impl VersionMessage {
    pub fn new<V: IbcSpec>(data: Value) -> Self {
        Self {
            ibc_version_id: V::ID,
            data,
        }
    }
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
    pub chain_id: ChainId,
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
    pub chain_id: ChainId,
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
    pub chain_id: ChainId,
    pub counterparty_chain_id: ChainId,
    pub update_from: Height,
    pub update_to: Height,
}

#[model]
pub struct WaitForHeight {
    pub chain_id: ChainId,
    pub height: Height,
    pub finalized: bool,
}

#[model]
pub struct WaitForTimestamp {
    pub chain_id: ChainId,
    /// THIS IS NANOSECONDS
    pub timestamp: i64,
    pub finalized: bool,
}

/// Wait for the client `.client_id` on `.chain_id` to trust a height >=
/// `.height`.
#[model]
pub struct WaitForTrustedHeight {
    pub chain_id: ChainId,
    pub client_id: ClientId,
    pub height: Height,
}

impl CallT<VoyagerMessage> for Call {
    // #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn process(self, ctx: &Context) -> Result<Op<VoyagerMessage>, QueueError> {
        match self {
            Call::Version(VersionMessage {
                ibc_version_id,
                data,
            }) => {
                (ctx.ibc_spec_handlers.get(&ibc_version_id).unwrap().call)(&ctx.rpc_server, data)
                    .await
            }
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
            Call::WaitForHeight(WaitForHeight {
                chain_id,
                height,
                finalized,
            }) => {
                let chain_height = ctx
                    .rpc_server
                    .query_latest_height(&chain_id, finalized)
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
                        call(WaitForHeight {
                            chain_id,
                            height,
                            finalized,
                        }),
                    ]))
                }
            }

            Call::WaitForTimestamp(WaitForTimestamp {
                chain_id,
                timestamp,
                finalized,
            }) => {
                let chain_timestamp = ctx
                    .rpc_server
                    .query_latest_timestamp(&chain_id, finalized)
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
                            finalized,
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
