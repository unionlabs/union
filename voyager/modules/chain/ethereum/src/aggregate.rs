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
use queue_msg::{aggregation::UseAggregate, data, queue_msg, Op, SubsetOf};
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

use crate::{data::ModuleData, fetch::ModuleFetch};

#[queue_msg]
#[derive(Enumorph, SubsetOf)]
pub enum ModuleAggregate {
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

impl UseAggregate<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>>
    for EventInfo<ClientCreatedFilter>
{
    type AggregatedData = HList![ClientInfo, DecodedClientStateMeta];

    fn aggregate(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![client_info, client_meta]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>> {
        data(ChainEvent {
            chain_id,
            client_info: client_info.clone(),
            counterparty_chain_id: client_meta.state.chain_id,
            tx_hash,
            provable_height: height,
            event: CreateClient {
                client_id: raw_event.client_id.parse().unwrap(),
                client_type: client_info.client_type.to_string(),
                consensus_height: client_meta.state.height,
            }
            .into(),
        })
    }
}

impl UseAggregate<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>>
    for EventInfo<ClientUpdatedFilter>
{
    type AggregatedData = HList![ClientInfo, DecodedClientStateMeta];

    fn aggregate(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![client_info, client_meta]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>> {
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

impl UseAggregate<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>>
    for EventInfo<ChannelOpenInitFilter>
{
    type AggregatedData = HList![ClientInfo, DecodedClientStateMeta, IbcState<ConnectionPath>];

    fn aggregate(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![client_info, client_meta, connection]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>> {
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

impl UseAggregate<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>>
    for EventInfo<ChannelOpenTryFilter>
{
    type AggregatedData = HList![ClientInfo, DecodedClientStateMeta, IbcState<ConnectionPath>];

    fn aggregate(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![client_info, client_meta, connection]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>> {
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

impl UseAggregate<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>>
    for EventInfo<ChannelOpenAckFilter>
{
    type AggregatedData = HList![
        ClientInfo,
        DecodedClientStateMeta,
        IbcState<ConnectionPath>,
        IbcState<ChannelEndPath>,
    ];

    fn aggregate(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![client_info, client_meta, connection, channel]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>> {
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

impl UseAggregate<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>>
    for EventInfo<ChannelOpenConfirmFilter>
{
    type AggregatedData = HList![
        ClientInfo,
        DecodedClientStateMeta,
        IbcState<ConnectionPath>,
        IbcState<ChannelEndPath>,
    ];

    fn aggregate(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![client_info, client_meta, connection, channel]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>> {
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

impl UseAggregate<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>>
    for EventInfo<ConnectionOpenInitFilter>
{
    type AggregatedData = HList![ClientInfo, DecodedClientStateMeta];

    fn aggregate(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![client_info, client_meta]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>> {
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

impl UseAggregate<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>>
    for EventInfo<ConnectionOpenTryFilter>
{
    type AggregatedData = HList![ClientInfo, DecodedClientStateMeta];

    fn aggregate(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![client_info, client_meta]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>> {
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

impl UseAggregate<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>>
    for EventInfo<ConnectionOpenAckFilter>
{
    type AggregatedData = HList![ClientInfo, DecodedClientStateMeta];

    fn aggregate(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![client_info, client_meta]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>> {
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

impl UseAggregate<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>>
    for EventInfo<ConnectionOpenConfirmFilter>
{
    type AggregatedData = HList![ClientInfo, DecodedClientStateMeta];

    fn aggregate(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![client_info, client_meta]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>> {
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

impl UseAggregate<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>>
    for EventInfo<SendPacketFilter>
{
    type AggregatedData = HList![
        ClientInfo,
        DecodedClientStateMeta,
        IbcState<ConnectionPath>,
        IbcState<ConnectionPath>,
        IbcState<ChannelEndPath>,
        IbcState<ChannelEndPath>,
    ];

    fn aggregate(
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
        ]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>> {
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

impl UseAggregate<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>>
    for EventInfo<RecvPacketFilter>
{
    type AggregatedData = HList![
        ClientInfo,
        DecodedClientStateMeta,
        IbcState<ConnectionPath>,
        IbcState<ConnectionPath>,
        IbcState<ChannelEndPath>,
        IbcState<ChannelEndPath>,
    ];

    fn aggregate(
        EventInfo {
            chain_id,
            height,
            tx_hash,
            raw_event,
        }: Self,
        hlist_pat![
            client_info,client_meta,
            ...rest
        ]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>> {
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

impl UseAggregate<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>>
    for EventInfo<TimeoutPacketFilter>
{
    type AggregatedData = HList![
        ClientInfo,
        DecodedClientStateMeta,
        IbcState<ConnectionPath>,
        IbcState<ConnectionPath>,
        IbcState<ChannelEndPath>,
        IbcState<ChannelEndPath>,
    ];

    fn aggregate(
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
        ]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>> {
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

impl UseAggregate<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>>
    for EventInfo<AcknowledgePacketFilter>
{
    type AggregatedData = HList![
        ClientInfo,
        DecodedClientStateMeta,
        IbcState<ConnectionPath>,
        IbcState<ConnectionPath>,
        IbcState<ChannelEndPath>,
        IbcState<ChannelEndPath>,
    ];

    fn aggregate(
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
        ]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>> {
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

impl UseAggregate<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>>
    for EventInfo<WriteAcknowledgementFilter>
{
    type AggregatedData = HList![
        ClientInfo,
        DecodedClientStateMeta,
        IbcState<ConnectionPath>,
        IbcState<ConnectionPath>,
        IbcState<ChannelEndPath>,
        IbcState<ChannelEndPath>,
    ];

    fn aggregate(
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
        ]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>> {
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
