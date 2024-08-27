use contracts::{
    ibc_channel_handshake::{
        ChannelOpenAckFilter, ChannelOpenConfirmFilter, ChannelOpenInitFilter, ChannelOpenTryFilter,
    },
    ibc_client::{ClientCreatedFilter, ClientUpdatedFilter},
    ibc_connection::{
        ConnectionOpenAckFilter, ConnectionOpenConfirmFilter, ConnectionOpenInitFilter,
        ConnectionOpenTryFilter,
    },
    ibc_packet::{
        AcknowledgePacketFilter, RecvPacketFilter, SendPacketFilter, TimeoutPacketFilter,
        WriteAcknowledgementFilter,
    },
};
use enumorph::Enumorph;
use frunk::{hlist, hlist_pat, HList};
use queue_msg::{aggregation::DoCallback, data, queue_msg, Op, SubsetOf};
use unionlabs::{
    hash::H256,
    ibc::core::client::height::Height,
    ics24::{ChannelEndPath, ConnectionPath},
};
use voyager_message::{
    callback::mk_packet_metadata_from_hlist,
    data::{
        AcknowledgePacket, ChainEvent, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit,
        ChannelOpenTry, ClientInfo, ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit,
        ConnectionOpenTry, CreateClient, DecodedClientStateMeta, IbcState, PacketMetadata,
        RecvPacket, SendPacket, TimeoutPacket, UpdateClient, WriteAcknowledgement,
    },
    ChainId, VoyagerMessage,
};

use crate::{call::ModuleCall, data::ModuleData};

#[queue_msg]
#[derive(Enumorph, SubsetOf)]
pub enum ModuleCallback {
    CreateClient(EventInfo<ClientCreatedFilter>),
    UpdateClient(EventInfo<ClientUpdatedFilter>),

    ConnectionOpenInit(EventInfo<ConnectionOpenInitFilter>),
    ConnectionOpenTry(EventInfo<ConnectionOpenTryFilter>),
    ConnectionOpenAck(EventInfo<ConnectionOpenAckFilter>),
    ConnectionOpenConfirm(EventInfo<ConnectionOpenConfirmFilter>),

    ChannelOpenInit(EventInfo<ChannelOpenInitFilter>),
    ChannelOpenTry(EventInfo<ChannelOpenTryFilter>),
    ChannelOpenAck(EventInfo<ChannelOpenAckFilter>),
    ChannelOpenConfirm(EventInfo<ChannelOpenConfirmFilter>),

    SendPacket(EventInfo<SendPacketFilter>),
    RecvPacket(EventInfo<RecvPacketFilter>),

    WriteAcknowledgement(EventInfo<WriteAcknowledgementFilter>),
    PacketAcknowledgement(EventInfo<AcknowledgePacketFilter>),

    PacketTimeout(EventInfo<TimeoutPacketFilter>),
}

#[queue_msg]
pub struct EventInfo<T> {
    pub chain_id: ChainId<'static>,
    pub height: Height,
    pub tx_hash: H256,
    pub raw_event: T,
}

impl DoCallback<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>
    for EventInfo<ClientCreatedFilter>
{
    type Params = HList![ClientInfo, DecodedClientStateMeta];

    fn call(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![client_info, client_meta]: Self::Params,
    ) -> Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>> {
        data(ChainEvent {
            chain_id,
            client_info: client_info.clone(),
            counterparty_chain_id: client_meta.state.chain_id,
            tx_hash,
            provable_height: height,
            event: CreateClient {
                client_id: raw_event.client_id.parse().unwrap(),
                client_type: client_info.client_type,
                consensus_height: client_meta.state.height,
            }
            .into(),
        })
    }
}

impl DoCallback<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>
    for EventInfo<ClientUpdatedFilter>
{
    type Params = HList![ClientInfo, DecodedClientStateMeta];

    fn call(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![client_info, client_meta]: Self::Params,
    ) -> Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>> {
        data(ChainEvent {
            chain_id,
            client_info: client_info.clone(),
            counterparty_chain_id: client_meta.state.chain_id,
            tx_hash,
            provable_height: height,
            event: UpdateClient {
                client_id: raw_event.client_id.parse().unwrap(),
                client_type: client_info.client_type.to_string(),
                consensus_heights: vec![raw_event.height.into()],
            }
            .into(),
        })
    }
}

impl DoCallback<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>
    for EventInfo<ChannelOpenInitFilter>
{
    type Params = HList![ClientInfo, DecodedClientStateMeta, IbcState<ConnectionPath>];

    fn call(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![client_info, client_meta, connection]: Self::Params,
    ) -> Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>> {
        data(ChainEvent {
            chain_id,
            client_info,
            counterparty_chain_id: client_meta.state.chain_id,
            tx_hash,
            provable_height: height,
            event: ChannelOpenInit {
                port_id: raw_event.port_id.parse().unwrap(),
                channel_id: raw_event.channel_id.parse().unwrap(),
                counterparty_port_id: raw_event.counterparty_port_id.parse().unwrap(),
                connection: connection.state,
                version: raw_event.version,
            }
            .into(),
        })
    }
}

impl DoCallback<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>
    for EventInfo<ChannelOpenTryFilter>
{
    type Params = HList![ClientInfo, DecodedClientStateMeta, IbcState<ConnectionPath>];

    fn call(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![client_info, client_meta, connection]: Self::Params,
    ) -> Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>> {
        data(ChainEvent {
            chain_id,
            client_info,
            counterparty_chain_id: client_meta.state.chain_id,
            tx_hash,
            provable_height: height,
            event: ChannelOpenTry {
                port_id: raw_event.port_id.parse().unwrap(),
                channel_id: raw_event.channel_id.parse().unwrap(),
                counterparty_channel_id: raw_event.counterparty_channel_id.parse().unwrap(),
                counterparty_port_id: raw_event.counterparty_port_id.parse().unwrap(),
                connection: connection.state,
                version: raw_event.version,
            }
            .into(),
        })
    }
}

impl DoCallback<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>
    for EventInfo<ChannelOpenAckFilter>
{
    type Params = HList![
        ClientInfo,
        DecodedClientStateMeta,
        IbcState<ConnectionPath>,
        IbcState<ChannelEndPath>,
    ];

    fn call(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![client_info, client_meta, connection, channel]: Self::Params,
    ) -> Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>> {
        data(ChainEvent {
            chain_id,
            client_info,
            counterparty_chain_id: client_meta.state.chain_id,
            tx_hash,
            provable_height: height,
            event: ChannelOpenAck {
                port_id: raw_event.port_id.parse().unwrap(),
                channel_id: raw_event.channel_id.parse().unwrap(),
                counterparty_channel_id: raw_event.counterparty_channel_id.parse().unwrap(),
                counterparty_port_id: raw_event.counterparty_port_id.parse().unwrap(),
                connection: connection.state,
                version: channel.state.version,
            }
            .into(),
        })
    }
}

impl DoCallback<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>
    for EventInfo<ChannelOpenConfirmFilter>
{
    type Params = HList![
        ClientInfo,
        DecodedClientStateMeta,
        IbcState<ConnectionPath>,
        IbcState<ChannelEndPath>,
    ];

    fn call(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![client_info, client_meta, connection, channel]: Self::Params,
    ) -> Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>> {
        data(ChainEvent {
            chain_id,
            client_info,
            counterparty_chain_id: client_meta.state.chain_id,
            tx_hash,
            provable_height: height,
            event: ChannelOpenConfirm {
                port_id: raw_event.port_id.parse().unwrap(),
                channel_id: raw_event.channel_id.parse().unwrap(),
                counterparty_channel_id: raw_event.counterparty_channel_id.parse().unwrap(),
                counterparty_port_id: raw_event.counterparty_port_id.parse().unwrap(),
                connection: connection.state,
                version: channel.state.version,
            }
            .into(),
        })
    }
}

impl DoCallback<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>
    for EventInfo<ConnectionOpenInitFilter>
{
    type Params = HList![ClientInfo, DecodedClientStateMeta];

    fn call(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![client_info, client_meta]: Self::Params,
    ) -> Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>> {
        data(ChainEvent {
            chain_id,
            client_info,
            counterparty_chain_id: client_meta.state.chain_id,
            tx_hash,
            provable_height: height,
            event: ConnectionOpenInit {
                connection_id: raw_event.connection_id.parse().unwrap(),
                client_id: raw_event.client_id.parse().unwrap(),
                counterparty_client_id: raw_event.counterparty_client_id.parse().unwrap(),
            }
            .into(),
        })
    }
}

impl DoCallback<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>
    for EventInfo<ConnectionOpenTryFilter>
{
    type Params = HList![ClientInfo, DecodedClientStateMeta];

    fn call(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![client_info, client_meta]: Self::Params,
    ) -> Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>> {
        data(ChainEvent {
            chain_id,
            client_info,
            counterparty_chain_id: client_meta.state.chain_id,
            tx_hash,
            provable_height: height,
            event: ConnectionOpenTry {
                connection_id: raw_event.connection_id.parse().unwrap(),
                client_id: raw_event.client_id.parse().unwrap(),
                counterparty_client_id: raw_event.counterparty_client_id.parse().unwrap(),
                counterparty_connection_id: raw_event.counterparty_connection_id.parse().unwrap(),
            }
            .into(),
        })
    }
}

impl DoCallback<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>
    for EventInfo<ConnectionOpenAckFilter>
{
    type Params = HList![ClientInfo, DecodedClientStateMeta];

    fn call(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![client_info, client_meta]: Self::Params,
    ) -> Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>> {
        data(ChainEvent {
            chain_id,
            client_info,
            counterparty_chain_id: client_meta.state.chain_id,
            tx_hash,
            provable_height: height,
            event: ConnectionOpenAck {
                connection_id: raw_event.connection_id.parse().unwrap(),
                client_id: raw_event.client_id.parse().unwrap(),
                counterparty_client_id: raw_event.counterparty_client_id.parse().unwrap(),
                counterparty_connection_id: raw_event.counterparty_connection_id.parse().unwrap(),
            }
            .into(),
        })
    }
}

impl DoCallback<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>
    for EventInfo<ConnectionOpenConfirmFilter>
{
    type Params = HList![ClientInfo, DecodedClientStateMeta];

    fn call(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![client_info, client_meta]: Self::Params,
    ) -> Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>> {
        data(ChainEvent {
            chain_id,
            client_info,
            counterparty_chain_id: client_meta.state.chain_id,
            tx_hash,
            provable_height: height,
            event: ConnectionOpenConfirm {
                connection_id: raw_event.connection_id.parse().unwrap(),
                client_id: raw_event.client_id.parse().unwrap(),
                counterparty_client_id: raw_event.counterparty_client_id.parse().unwrap(),
                counterparty_connection_id: raw_event.counterparty_connection_id.parse().unwrap(),
            }
            .into(),
        })
    }
}

impl DoCallback<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>
    for EventInfo<SendPacketFilter>
{
    type Params = HList![
        ClientInfo,
        DecodedClientStateMeta,
        IbcState<ConnectionPath>,
        IbcState<ConnectionPath>,
        IbcState<ChannelEndPath>,
        IbcState<ChannelEndPath>,
    ];

    fn call(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![
            client_info,
            client_meta,
            connection_a,
            connection_b,
            channel_a,
            channel_b,
        ]: Self::Params,
    ) -> Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>> {
        let source_channel = if *raw_event.source_port == *channel_a.path.port_id
            && *raw_event.source_channel == *channel_a.path.channel_id
        {
            channel_a.clone()
        } else if *raw_event.source_port == *channel_b.path.port_id
            && *raw_event.source_channel == *channel_b.path.channel_id
        {
            channel_b.clone()
        } else {
            panic!("invalid data, neither channel matches")
        };

        let (source_channel, destination_channel) = mk_packet_metadata_from_hlist::<ModuleData>(
            hlist![connection_a, connection_b, channel_a, channel_b],
            chain_id.clone(),
            client_meta.state.chain_id.clone(),
            source_channel.path.port_id,
            source_channel.path.channel_id,
            source_channel.state.counterparty.port_id,
            source_channel
                .state
                .counterparty
                .channel_id
                .parse()
                .unwrap(),
        );

        data(ChainEvent {
            chain_id,
            client_info,
            counterparty_chain_id: client_meta.state.chain_id,
            tx_hash,
            provable_height: height,
            event: SendPacket {
                packet_data: raw_event.data.to_vec(),
                packet: PacketMetadata {
                    sequence: raw_event.sequence.try_into().unwrap(),
                    source_channel,
                    destination_channel,
                    timeout_height: raw_event.timeout_height.into(),
                    timeout_timestamp: raw_event.timeout_timestamp,
                },
            }
            .into(),
        })
    }
}

impl DoCallback<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>
    for EventInfo<RecvPacketFilter>
{
    type Params = HList![
        ClientInfo,
        DecodedClientStateMeta,
        IbcState<ConnectionPath>,
        IbcState<ConnectionPath>,
        IbcState<ChannelEndPath>,
        IbcState<ChannelEndPath>,
    ];

    fn call(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![
            client_info,client_meta,
            ...rest
        ]: Self::Params,
    ) -> Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>> {
        let (source_channel, destination_channel) = mk_packet_metadata_from_hlist::<ModuleData>(
            rest,
            client_meta.state.chain_id.clone(),
            chain_id.clone(),
            raw_event.packet.source_port.parse().unwrap(),
            raw_event.packet.source_channel.parse().unwrap(),
            raw_event.packet.destination_port.parse().unwrap(),
            raw_event.packet.destination_channel.parse().unwrap(),
        );

        data(ChainEvent {
            chain_id,
            client_info,
            counterparty_chain_id: client_meta.state.chain_id,
            tx_hash,
            provable_height: height,
            event: RecvPacket {
                packet_data: raw_event.packet.data.to_vec(),
                packet: PacketMetadata {
                    sequence: raw_event.packet.sequence.try_into().unwrap(),
                    source_channel,
                    destination_channel,
                    timeout_height: raw_event.packet.timeout_height.into(),
                    timeout_timestamp: raw_event.packet.timeout_timestamp,
                },
            }
            .into(),
        })
    }
}

impl DoCallback<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>
    for EventInfo<TimeoutPacketFilter>
{
    type Params = HList![
        ClientInfo,
        DecodedClientStateMeta,
        IbcState<ConnectionPath>,
        IbcState<ConnectionPath>,
        IbcState<ChannelEndPath>,
        IbcState<ChannelEndPath>,
    ];

    fn call(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![
            client_info,
            client_meta,
            ...rest
        ]: Self::Params,
    ) -> Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>> {
        let (source_channel, destination_channel) = mk_packet_metadata_from_hlist::<ModuleData>(
            rest,
            chain_id.clone(),
            client_meta.state.chain_id.clone(),
            raw_event.packet.source_port.parse().unwrap(),
            raw_event.packet.source_channel.parse().unwrap(),
            raw_event.packet.destination_port.parse().unwrap(),
            raw_event.packet.destination_channel.parse().unwrap(),
        );

        data(ChainEvent {
            chain_id,
            client_info,
            counterparty_chain_id: client_meta.state.chain_id,
            tx_hash,
            provable_height: height,
            event: TimeoutPacket {
                packet: PacketMetadata {
                    sequence: raw_event.packet.sequence.try_into().unwrap(),
                    source_channel,
                    destination_channel,
                    timeout_height: raw_event.packet.timeout_height.into(),
                    timeout_timestamp: raw_event.packet.timeout_timestamp,
                },
            }
            .into(),
        })
    }
}

impl DoCallback<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>
    for EventInfo<AcknowledgePacketFilter>
{
    type Params = HList![
        ClientInfo,
        DecodedClientStateMeta,
        IbcState<ConnectionPath>,
        IbcState<ConnectionPath>,
        IbcState<ChannelEndPath>,
        IbcState<ChannelEndPath>,
    ];

    fn call(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![
            client_info,
            client_meta,
            ...rest
        ]: Self::Params,
    ) -> Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>> {
        let (source_channel, destination_channel) = mk_packet_metadata_from_hlist::<ModuleData>(
            rest,
            chain_id.clone(),
            client_meta.state.chain_id.clone(),
            raw_event.packet.source_port.parse().unwrap(),
            raw_event.packet.source_channel.parse().unwrap(),
            raw_event.packet.destination_port.parse().unwrap(),
            raw_event.packet.destination_channel.parse().unwrap(),
        );

        data(ChainEvent {
            chain_id,
            client_info,
            counterparty_chain_id: client_meta.state.chain_id,
            tx_hash,
            provable_height: height,
            event: AcknowledgePacket {
                packet: PacketMetadata {
                    sequence: raw_event.packet.sequence.try_into().unwrap(),
                    source_channel,
                    destination_channel,
                    timeout_height: raw_event.packet.timeout_height.into(),
                    timeout_timestamp: raw_event.packet.timeout_timestamp,
                },
            }
            .into(),
        })
    }
}

impl DoCallback<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>
    for EventInfo<WriteAcknowledgementFilter>
{
    type Params = HList![
        ClientInfo,
        DecodedClientStateMeta,
        IbcState<ConnectionPath>,
        IbcState<ConnectionPath>,
        IbcState<ChannelEndPath>,
        IbcState<ChannelEndPath>,
    ];

    fn call(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![
            client_info,
            client_meta,
            ...rest
        ]: Self::Params,
    ) -> Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>> {
        let (source_channel, destination_channel) = mk_packet_metadata_from_hlist::<ModuleData>(
            rest,
            client_meta.state.chain_id.clone(),
            chain_id.clone(),
            raw_event.packet.source_port.parse().unwrap(),
            raw_event.packet.source_channel.parse().unwrap(),
            raw_event.packet.destination_port.parse().unwrap(),
            raw_event.packet.destination_channel.parse().unwrap(),
        );

        data(ChainEvent {
            chain_id,
            client_info,
            counterparty_chain_id: client_meta.state.chain_id,
            tx_hash,
            provable_height: height,
            event: WriteAcknowledgement {
                packet_ack: raw_event.acknowledgement.to_vec(),
                packet_data: raw_event.packet.data.to_vec(),
                packet: PacketMetadata {
                    sequence: raw_event.packet.sequence.try_into().unwrap(),
                    source_channel,
                    destination_channel,
                    timeout_height: raw_event.packet.timeout_height.into(),
                    timeout_timestamp: raw_event.packet.timeout_timestamp,
                },
            }
            .into(),
        })
    }
}
