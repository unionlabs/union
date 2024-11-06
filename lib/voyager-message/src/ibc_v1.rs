pub mod data {
    use std::num::NonZeroU64;

    use enumorph::Enumorph;
    use macros::model;
    use subset_of::SubsetOf;
    use tracing::info;
    use unionlabs::{
        bytes::Bytes,
        ibc::core::{
            channel::{
                msg_acknowledgement::MsgAcknowledgement, msg_channel_open_ack::MsgChannelOpenAck,
                msg_channel_open_confirm::MsgChannelOpenConfirm,
                msg_channel_open_init::MsgChannelOpenInit, msg_channel_open_try::MsgChannelOpenTry,
                msg_recv_packet::MsgRecvPacket, msg_timeout::MsgTimeout, order::Order,
            },
            client::{
                height::Height, msg_create_client::MsgCreateClient,
                msg_update_client::MsgUpdateClient,
            },
            connection::{
                connection_end::ConnectionEnd, msg_connection_open_ack::MsgConnectionOpenAck,
                msg_connection_open_confirm::MsgConnectionOpenConfirm,
                msg_connection_open_init::MsgConnectionOpenInit,
                msg_connection_open_try::MsgConnectionOpenTry,
            },
        },
        id::{ChannelId, ClientId, ConnectionId, PortId},
    };
    use voyager_core::ClientType;

    pub enum VersionData {}

    #[model]
    #[derive(Enumorph)]
    pub enum IbcMessage {
        CreateClient(MsgCreateClientData),
        UpdateClient(MsgUpdateClient),

        ConnectionOpenInit(MsgConnectionOpenInit),
        ConnectionOpenTry(MsgConnectionOpenTry),
        ConnectionOpenAck(MsgConnectionOpenAck),
        ConnectionOpenConfirm(MsgConnectionOpenConfirm),

        ChannelOpenInit(MsgChannelOpenInit),
        ChannelOpenTry(MsgChannelOpenTry),
        ChannelOpenAck(MsgChannelOpenAck),
        ChannelOpenConfirm(MsgChannelOpenConfirm),

        RecvPacket(MsgRecvPacket),
        AcknowledgePacket(MsgAcknowledgement),
        TimeoutPacket(MsgTimeout),
    }

    impl IbcMessage {
        /// Returns the proof height of the IBC message, if it has one.
        /// (ConnectionOpenInit does not contain a proof, for example)
        pub fn proof_height(&self) -> Option<Height> {
            match self {
                IbcMessage::CreateClient(_) => None,
                IbcMessage::UpdateClient(_) => None,
                IbcMessage::ConnectionOpenInit(_) => None,
                IbcMessage::ConnectionOpenTry(msg) => Some(msg.proof_height),
                IbcMessage::ConnectionOpenAck(msg) => Some(msg.proof_height),
                IbcMessage::ConnectionOpenConfirm(msg) => Some(msg.proof_height),
                IbcMessage::ChannelOpenInit(_) => None,
                IbcMessage::ChannelOpenTry(msg) => Some(msg.proof_height),
                IbcMessage::ChannelOpenAck(msg) => Some(msg.proof_height),
                IbcMessage::ChannelOpenConfirm(msg) => Some(msg.proof_height),
                IbcMessage::RecvPacket(msg) => Some(msg.proof_height),
                IbcMessage::AcknowledgePacket(msg) => Some(msg.proof_height),
                IbcMessage::TimeoutPacket(msg) => Some(msg.proof_height),
            }
        }

        pub fn name(&self) -> &'static str {
            match self {
                IbcMessage::CreateClient(_) => "create_client",
                IbcMessage::UpdateClient(_) => "update_client",
                IbcMessage::ConnectionOpenInit(_) => "connection_open_init",
                IbcMessage::ConnectionOpenTry(_) => "connection_open_try",
                IbcMessage::ConnectionOpenAck(_) => "connection_open_ack",
                IbcMessage::ConnectionOpenConfirm(_) => "connection_open_confirm",
                IbcMessage::ChannelOpenInit(_) => "channel_open_init",
                IbcMessage::ChannelOpenTry(_) => "channel_open_try",
                IbcMessage::ChannelOpenAck(_) => "channel_open_ack",
                IbcMessage::ChannelOpenConfirm(_) => "channel_open_confirm",
                IbcMessage::RecvPacket(_) => "recv_packet",
                IbcMessage::AcknowledgePacket(_) => "acknowledgement",
                IbcMessage::TimeoutPacket(_) => "timeout",
            }
        }
    }

    #[model]
    pub struct CreateClient {
        pub client_id: ClientId,
        pub client_type: ClientType<'static>,
        pub consensus_height: Height,
    }

    #[model]
    pub struct UpdateClient {
        pub client_id: ClientId,
        pub client_type: ClientType<'static>,
        pub consensus_heights: Vec<Height>,
    }

    #[model]
    pub struct ConnectionOpenInit {
        pub connection_id: ConnectionId,
        pub client_id: ClientId,
        pub counterparty_client_id: ClientId,
    }

    #[model]
    pub struct ConnectionOpenTry {
        pub connection_id: ConnectionId,
        pub client_id: ClientId,
        pub counterparty_client_id: ClientId,
        pub counterparty_connection_id: ConnectionId,
    }

    #[model]
    pub struct ConnectionOpenAck {
        pub connection_id: ConnectionId,
        pub client_id: ClientId,
        pub counterparty_client_id: ClientId,
        pub counterparty_connection_id: ConnectionId,
    }

    #[model]
    pub struct ConnectionOpenConfirm {
        pub connection_id: ConnectionId,
        pub client_id: ClientId,
        pub counterparty_client_id: ClientId,
        pub counterparty_connection_id: ConnectionId,
    }

    #[model]
    pub struct ChannelOpenInit {
        pub port_id: PortId,
        pub channel_id: ChannelId,

        pub counterparty_port_id: PortId,

        pub connection: ConnectionEnd,

        pub version: String,
    }

    #[model]
    pub struct ChannelOpenTry {
        pub port_id: PortId,
        pub channel_id: ChannelId,

        pub counterparty_port_id: PortId,
        pub counterparty_channel_id: ChannelId,

        pub connection: ConnectionEnd,

        pub version: String,
    }

    #[model]
    pub struct ChannelOpenAck {
        pub port_id: PortId,
        pub channel_id: ChannelId,

        pub counterparty_port_id: PortId,
        pub counterparty_channel_id: ChannelId,

        pub connection: ConnectionEnd,

        pub version: String,
    }

    #[model]
    pub struct ChannelOpenConfirm {
        pub port_id: PortId,
        pub channel_id: ChannelId,

        pub counterparty_port_id: PortId,
        pub counterparty_channel_id: ChannelId,

        pub connection: ConnectionEnd,

        pub version: String,
    }

    #[model]
    pub struct WriteAcknowledgement {
        pub packet_data: Bytes,
        pub packet_ack: Bytes,
        pub packet: PacketMetadata,
    }

    #[model]
    pub struct RecvPacket {
        pub packet_data: Bytes,
        pub packet: PacketMetadata,
    }

    #[model]
    pub struct SendPacket {
        pub packet_data: Bytes,

        pub packet: PacketMetadata,
    }

    #[model]
    pub struct AcknowledgePacket {
        pub packet: PacketMetadata,
    }

    #[model]
    pub struct TimeoutPacket {
        pub packet: PacketMetadata,
    }

    #[model]
    pub struct PacketMetadata {
        pub sequence: NonZeroU64,

        pub source_channel: ChannelMetadata,
        pub destination_channel: ChannelMetadata,

        pub channel_ordering: Order,

        pub timeout_height: Height,
        pub timeout_timestamp: u64,
    }

    #[model]
    pub struct ChannelMetadata {
        pub port_id: PortId,
        pub channel_id: ChannelId,
        // REVIEW: Can this be different on either end of a channel?
        pub version: String,
        pub connection: ConnectionMetadata,
    }

    #[model]
    pub struct ConnectionMetadata {
        pub client_id: ClientId,
        // this is really `Either<ConnectionId, EmptyString>`
        // REVIEW: Is it?
        pub connection_id: ConnectionId,
    }

    /// Similar to `IbcEvent`, but contains more information (counterparty
    /// clients, channel version, etc)
    #[model]
    #[derive(Enumorph, SubsetOf)]
    pub enum FullIbcEvent {
        // TODO: Probably move create client and update client into the top level message enum
        CreateClient(CreateClient),
        UpdateClient(UpdateClient),

        ConnectionOpenInit(ConnectionOpenInit),
        ConnectionOpenTry(ConnectionOpenTry),
        ConnectionOpenAck(ConnectionOpenAck),
        ConnectionOpenConfirm(ConnectionOpenConfirm),

        ChannelOpenInit(ChannelOpenInit),
        ChannelOpenTry(ChannelOpenTry),
        ChannelOpenAck(ChannelOpenAck),
        ChannelOpenConfirm(ChannelOpenConfirm),

        SendPacket(SendPacket),
        RecvPacket(RecvPacket),
        WriteAcknowledgement(WriteAcknowledgement),
        AcknowledgePacket(AcknowledgePacket),
        TimeoutPacket(TimeoutPacket),
    }

    impl FullIbcEvent {
        pub fn client_id(&self) -> &ClientId {
            match self {
                Self::CreateClient(ref event) => &event.client_id,
                Self::UpdateClient(ref event) => &event.client_id,
                Self::ConnectionOpenInit(ref event) => &event.client_id,
                Self::ConnectionOpenTry(ref event) => &event.client_id,
                Self::ConnectionOpenAck(ref event) => &event.client_id,
                Self::ConnectionOpenConfirm(ref event) => &event.client_id,
                Self::ChannelOpenInit(ref event) => &event.connection.client_id,
                Self::ChannelOpenTry(ref event) => &event.connection.client_id,
                Self::ChannelOpenAck(ref event) => &event.connection.client_id,
                Self::ChannelOpenConfirm(ref event) => &event.connection.client_id,
                Self::SendPacket(ref event) => &event.packet.source_channel.connection.client_id,
                Self::RecvPacket(ref event) => &event.packet.source_channel.connection.client_id,
                Self::WriteAcknowledgement(ref event) => {
                    &event.packet.source_channel.connection.client_id
                }
                Self::AcknowledgePacket(ref event) => {
                    &event.packet.source_channel.connection.client_id
                }
                Self::TimeoutPacket(ref event) => &event.packet.source_channel.connection.client_id,
            }
        }

        /// Returns the counterparty client id of this ibc event, if there is a
        /// counterparty. This will return `None` for `UpdateClient` and
        /// `CreateClient`.
        pub fn counterparty_client_id(&self) -> Option<&ClientId> {
            match self {
                Self::ConnectionOpenInit(ref event) => Some(&event.counterparty_client_id),
                Self::ConnectionOpenTry(ref event) => Some(&event.counterparty_client_id),
                Self::ConnectionOpenAck(ref event) => Some(&event.counterparty_client_id),
                Self::ConnectionOpenConfirm(ref event) => Some(&event.counterparty_client_id),
                Self::ChannelOpenInit(ref event) => Some(&event.connection.counterparty.client_id),
                Self::ChannelOpenTry(ref event) => Some(&event.connection.counterparty.client_id),
                Self::ChannelOpenAck(ref event) => Some(&event.connection.counterparty.client_id),
                Self::ChannelOpenConfirm(ref event) => {
                    Some(&event.connection.counterparty.client_id)
                }
                Self::SendPacket(ref event) => {
                    Some(&event.packet.destination_channel.connection.client_id)
                }
                Self::RecvPacket(ref event) => {
                    Some(&event.packet.source_channel.connection.client_id)
                }
                Self::WriteAcknowledgement(ref event) => {
                    Some(&event.packet.source_channel.connection.client_id)
                }
                Self::AcknowledgePacket(ref event) => {
                    Some(&event.packet.destination_channel.connection.client_id)
                }
                Self::TimeoutPacket(ref event) => {
                    Some(&event.packet.destination_channel.connection.client_id)
                }
                _ => None,
            }
        }
    }

    #[model]
    pub struct MsgCreateClientData {
        pub msg: MsgCreateClient,
        pub client_type: ClientType<'static>,
    }

    pub fn log_msg(chain_id: &str, effect: &IbcMessage) {
        match effect.clone() {
            IbcMessage::ConnectionOpenInit(message) => {
                info!(
                    %chain_id,
                    %message.client_id,
                    %message.counterparty.client_id,
                    // TODO: Use Valuable here
                    ?message.counterparty.connection_id,
                    message.counterparty.prefix.key_prefix = %::serde_utils::to_hex(message.counterparty.prefix.key_prefix),
                    %message.version.identifier,
                    message.version.features = %message
                        .version
                        .features
                        .into_iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                    %message.delay_period,
                )
            }
            IbcMessage::ConnectionOpenTry(message) => {
                info!(
                    %chain_id,
                    %message.client_id,
                    %message.counterparty.client_id,
                    // TODO: Use Valuable here
                    ?message.counterparty.connection_id,
                    message.counterparty.prefix.key_prefix = %::serde_utils::to_hex(message.counterparty.prefix.key_prefix),
                    %message.delay_period,
                    // TODO: This needs `valuable`
                    // message.counterparty_versions = %message
                    //     .counterparty_versions
                    //     .into_iter()
                    //     .map(Into::into)
                    //     .collect(),
                    %message.proof_height,
                    %message.consensus_height,
                )
            }
            IbcMessage::ConnectionOpenAck(message) => {
                info!(
                    %chain_id,
                    // client_state.height = message.%data.message.client_state.height(),
                    %message.proof_height,
                    %message.consensus_height,
                    %message.connection_id,
                    %message.counterparty_connection_id,
                    %message.version.identifier,
                    message.version.features = %message
                        .version
                        .features
                        .into_iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                )
            }
            IbcMessage::ConnectionOpenConfirm(message) => {
                info!(
                    %chain_id,
                    %message.connection_id,
                    %message.proof_height,
                )
            }
            IbcMessage::ChannelOpenInit(message) => {
                info!(
                    %chain_id,
                    %message.port_id,
                    %message.channel.state,
                    %message.channel.ordering,
                    %message.channel.counterparty.port_id,
                    // TODO: Use Valuable here
                    ?message.channel.counterparty.channel_id,
                    message.channel.connection_hops = %message
                        .channel
                        .connection_hops
                        .into_iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                    %message.channel.version,
                )
            }
            IbcMessage::ChannelOpenTry(message) => {
                info!(
                    %chain_id,

                    %message.port_id,
                    %message.channel.state,
                    %message.channel.ordering,
                    %message.channel.counterparty.port_id,
                    // TODO: Use Valuable here
                    ?message.channel.counterparty.channel_id,
                    message.channel.connection_hops = %message
                        .channel
                        .connection_hops
                        .into_iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                    %message.channel.version,
                    %message.counterparty_version,
                    %message.proof_height,
                )
            }
            IbcMessage::ChannelOpenAck(message) => {
                info!(
                    %chain_id,
                    %message.port_id,
                    %message.channel_id,
                    %message.counterparty_version,
                    %message.counterparty_channel_id,
                    %message.proof_height,
                )
            }
            IbcMessage::ChannelOpenConfirm(message) => {
                info!(
                    %chain_id,
                    %message.port_id,
                    %message.channel_id,
                    %message.proof_height,
                )
            }
            IbcMessage::RecvPacket(message) => {
                info!(
                    %chain_id,
                    %message.packet.sequence,
                    %message.packet.source_port,
                    %message.packet.source_channel,
                    %message.packet.destination_port,
                    %message.packet.destination_channel,
                    message.data = %::serde_utils::to_hex(message.packet.data),
                    %message.packet.timeout_height,
                    %message.packet.timeout_timestamp,

                    %message.proof_height,
                )
            }
            IbcMessage::AcknowledgePacket(message) => {
                info!(
                    %chain_id,
                    %message.packet.sequence,
                    %message.packet.source_port,
                    %message.packet.source_channel,
                    %message.packet.destination_port,
                    %message.packet.destination_channel,
                    message.data = %::serde_utils::to_hex(message.packet.data),
                    %message.packet.timeout_height,
                    %message.packet.timeout_timestamp,

                    message.data = %::serde_utils::to_hex(message.acknowledgement),
                    %message.proof_height,
                )
            }
            IbcMessage::TimeoutPacket(message) => {
                info!(
                    %chain_id,
                    %message.packet.sequence,
                    %message.packet.source_port,
                    %message.packet.source_channel,
                    %message.packet.destination_port,
                    %message.packet.destination_channel,
                    message.data = %::serde_utils::to_hex(message.packet.data),
                    %message.packet.timeout_height,
                    %message.packet.timeout_timestamp,

                    %message.proof_height,
                    %message.next_sequence_recv,
                )
            }
            IbcMessage::CreateClient(message) => {
                info!(
                    %chain_id,
                    %message.client_type,
                )
            }
            IbcMessage::UpdateClient(message) => {
                info!(
                    %chain_id,
                    %message.client_id,
                )
            }
        }
    }
}
pub mod call {
    use futures::future::BoxFuture;
    use jsonrpsee::core::RpcResult;
    use macros::model;
    use serde_json::Value;
    use tracing::{info, instrument, trace};
    use unionlabs::{
        ibc::core::{
            channel::{self, msg_recv_packet::MsgRecvPacket},
            client::height::Height,
        },
        ics24::CommitmentPath,
        id::{ClientId, ConnectionId},
    };
    use voyager_core::{ChainId, ClientType, IbcInterface, IbcVersionId, QueryHeight};
    use voyager_vm::{noop, Op, QueueError};

    use crate::{
        call::VersionMessage, error_object_to_queue_error, ibc_v1::data::IbcMessage, into_value,
        json_rpc_error_to_queue_error, VoyagerMessage,
    };

    pub fn process(
        ctx: &crate::rpc::server::Server,
        c: Value,
    ) -> BoxFuture<Result<Op<VoyagerMessage>, QueueError>> {
        Box::pin(async move {
            let c = serde_json::from_value::<VersionCall>(c).map_err(QueueError::fatal)?;

            match c {
                VersionCall::MakeMsgConnectionOpenTry(make_msg_connection_open_try) => todo!(),
                VersionCall::MakeMsgConnectionOpenAck(make_msg_connection_open_ack) => todo!(),
                VersionCall::MakeMsgConnectionOpenConfirm(make_msg_connection_open_confirm) => {
                    todo!()
                }
                VersionCall::MakeMsgChannelOpenTry(make_msg_channel_open_try) => todo!(),
                VersionCall::MakeMsgChannelOpenAck(make_msg_channel_open_ack) => todo!(),
                VersionCall::MakeMsgChannelOpenConfirm(make_msg_channel_open_confirm) => todo!(),

                VersionCall::MakeMsgRecvPacket(msg) => make_msg_recv_packet(ctx, msg).await,
                VersionCall::MakeMsgAcknowledgement(msg) => {
                    make_msg_acknowledgement(ctx, msg).await
                }
            }
        })
    }

    #[model]
    pub enum VersionCall {
        MakeMsgConnectionOpenTry(MakeMsgConnectionOpenTry),
        MakeMsgConnectionOpenAck(MakeMsgConnectionOpenAck),
        MakeMsgConnectionOpenConfirm(MakeMsgConnectionOpenConfirm),

        MakeMsgChannelOpenTry(MakeMsgChannelOpenTry),
        MakeMsgChannelOpenAck(MakeMsgChannelOpenAck),
        MakeMsgChannelOpenConfirm(MakeMsgChannelOpenConfirm),

        MakeMsgAcknowledgement(MakeMsgAcknowledgement),
        MakeMsgRecvPacket(MakeMsgRecvPacket),
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
        pub connection_open_init_event: crate::ibc_v1::data::ConnectionOpenInit,
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
        pub connection_open_try_event: crate::ibc_v1::data::ConnectionOpenTry,
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
        pub connection_open_ack_event: crate::ibc_v1::data::ConnectionOpenAck,
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
        pub channel_open_init_event: crate::ibc_v1::data::ChannelOpenInit,
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
        pub channel_open_try_event: crate::ibc_v1::data::ChannelOpenTry,
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
        pub channel_open_ack_event: crate::ibc_v1::data::ChannelOpenAck,
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
        pub send_packet_event: crate::ibc_v1::data::SendPacket,
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
        pub write_acknowledgement_event: crate::ibc_v1::data::WriteAcknowledgement,
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
        ctx: &crate::rpc::server::Server,
        MakeMsgRecvPacket {
            origin_chain_id,
            origin_chain_proof_height,
            target_chain_id,
            send_packet_event,
        }: MakeMsgRecvPacket,
    ) -> Result<Op<VoyagerMessage>, QueueError> {
        let target_chain_latest_height = ctx
            .query_latest_height(&target_chain_id, true)
            .await
            .map_err(error_object_to_queue_error)?;

        let commitment = ctx
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
            .encode_proof(
                &client_info.client_type,
                &client_info.ibc_interface,
                proof_commitment,
            )
            .await
            .map_err(error_object_to_queue_error)?;

        Ok(voyager_vm::data(VersionMessage {
            ibc_version_id: IbcVersionId::new(IbcVersionId::V1_0_0),
            data: into_value(IbcMessage::from(MsgRecvPacket {
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
            })),
        }))
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
        ctx: &crate::rpc::server::Server,
        MakeMsgAcknowledgement {
            origin_chain_id,
            origin_chain_proof_height,
            target_chain_id,
            write_acknowledgement_event,
        }: MakeMsgAcknowledgement,
    ) -> Result<Op<VoyagerMessage>, QueueError> {
        let target_chain_latest_height = ctx
            .query_latest_height(&target_chain_id, true)
            .await
            .map_err(error_object_to_queue_error)?;

        let commitment = ctx
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
        ctx: &crate::rpc::server::Server,
        counterparty_chain_id: ChainId<'static>,
        height: QueryHeight,
        chain_id: ChainId<'static>,
        client_type: ClientType<'static>,
        ibc_interface: IbcInterface<'_>,
        metadata: Value,
    ) -> Result<Op<VoyagerMessage>, QueueError> {
        let height = ctx
            .query_latest_height(&counterparty_chain_id, true)
            .await
            .map_err(error_object_to_queue_error)?;

        let counterparty_consensus_module = ctx
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
            .modules()
            .map_err(error_object_to_queue_error)?
            .chain_consensus_type(&counterparty_chain_id)?;

        let client_consensus_type = ctx
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
        ctx: &crate::rpc::server::Server,
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
            .modules()?
            .client_module(
                &target_client_info.client_type,
                &target_client_info.ibc_interface,
            )?
            .reencode_counterparty_client_state(
                client_state.clone(),
                origin_client_info.client_type,
            )
            .await
            .map_err(json_rpc_error_to_error_object)?;

        debug!(%reencoded_client_state);

        // the connection end as stored by the origin chain after open_init/try
        let connection_state = ctx
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
            .encode_proof(
                &target_client_info.client_type,
                &target_client_info.ibc_interface,
                connection_proof,
            )
            .await?;
        debug!(encoded_connection_state_proof = %Hex(&encoded_connection_state_proof));

        let client_state_proof = ctx
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
            .encode_proof(
                &target_client_info.client_type,
                &target_client_info.ibc_interface,
                client_state_proof,
            )
            .await?;
        debug!(encoded_client_state_proof = %Hex(&encoded_client_state_proof));

        let consensus_state_proof = ctx
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
}
