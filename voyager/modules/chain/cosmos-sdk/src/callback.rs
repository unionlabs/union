use std::collections::VecDeque;

use enumorph::Enumorph;
use queue_msg::{
    aggregation::{pluck, PluckResult},
    data, noop, queue_msg, Op,
};
use unionlabs::{
    events::IbcEvent,
    hash::H256,
    ibc::core::client::height::{Height, IsHeight},
    ics24::{ChannelEndPath, ConnectionPath},
};
use voyager_message::{
    callback::mk_packet_metadata,
    data::{ChainEvent, ClientInfo, Data, DecodedClientStateMeta, IbcState, PacketMetadata},
    ChainId, VoyagerMessage,
};

use crate::{call::ModuleCall, data::ModuleData};

#[queue_msg]
#[derive(Enumorph)]
pub enum ModuleCallback {
    MakeFullEvent(MakeFullEvent),
}

#[queue_msg]
pub struct MakeFullEvent {
    pub chain_id: ChainId<'static>,
    pub tx_hash: H256,
    pub height: Height,
    pub event: IbcEvent,
}

impl MakeFullEvent {
    pub fn do_aggregate(
        self,
        datas: VecDeque<Data<ModuleData>>,
    ) -> Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>> {
        let MakeFullEvent {
            chain_id,
            tx_hash,
            height,
            event,
        } = self;

        let PluckResult::Found(client_info, datas) = pluck::<ClientInfo, _>(datas) else {
            panic!("missing ClientInfo");
        };

        let PluckResult::Found(client_meta, datas) = pluck::<DecodedClientStateMeta, _>(datas)
        else {
            panic!("missing DecodedClientStateMeta");
        };

        let mk_event = |full_event| ChainEvent {
            chain_id: chain_id.clone(),
            client_info,
            counterparty_chain_id: client_meta.state.chain_id.clone(),
            tx_hash,
            // don't ask
            provable_height: height.increment(),
            event: full_event,
        };

        let event = match event {
            IbcEvent::CreateClient(event) => {
                assert!(datas.is_empty());

                mk_event(
                    voyager_message::data::CreateClient {
                        client_id: event.client_id,
                        client_type: event.client_type,
                        consensus_height: event.consensus_height,
                    }
                    .into(),
                )
            }
            IbcEvent::UpdateClient(event) => {
                assert!(datas.is_empty());

                mk_event(
                    voyager_message::data::UpdateClient {
                        client_id: event.client_id,
                        client_type: event.client_type,
                        consensus_heights: event.consensus_heights,
                    }
                    .into(),
                )
            }
            IbcEvent::ClientMisbehaviour(_) => {
                assert!(datas.is_empty());

                return noop();
            }
            IbcEvent::SubmitEvidence(_) => {
                assert!(datas.is_empty());

                return noop();
            }
            IbcEvent::ConnectionOpenInit(event) => {
                assert!(datas.is_empty());

                mk_event(
                    voyager_message::data::ConnectionOpenInit {
                        connection_id: event.connection_id,
                        client_id: event.client_id,
                        counterparty_client_id: event.counterparty_client_id,
                    }
                    .into(),
                )
            }
            IbcEvent::ConnectionOpenTry(event) => {
                assert!(datas.is_empty());

                mk_event(
                    voyager_message::data::ConnectionOpenTry {
                        connection_id: event.connection_id,
                        client_id: event.client_id,
                        counterparty_client_id: event.counterparty_client_id,
                        counterparty_connection_id: event.counterparty_connection_id,
                    }
                    .into(),
                )
            }
            IbcEvent::ConnectionOpenAck(event) => {
                assert!(datas.is_empty());

                mk_event(
                    voyager_message::data::ConnectionOpenAck {
                        connection_id: event.connection_id,
                        client_id: event.client_id,
                        counterparty_client_id: event.counterparty_client_id,
                        counterparty_connection_id: event.counterparty_connection_id,
                    }
                    .into(),
                )
            }
            IbcEvent::ConnectionOpenConfirm(event) => {
                assert!(datas.is_empty());

                mk_event(
                    voyager_message::data::ConnectionOpenConfirm {
                        connection_id: event.connection_id,
                        client_id: event.client_id,
                        counterparty_client_id: event.counterparty_client_id,
                        counterparty_connection_id: event.counterparty_connection_id,
                    }
                    .into(),
                )
            }
            IbcEvent::ChannelOpenInit(event) => {
                let PluckResult::Found(connection, datas) =
                    pluck::<IbcState<ConnectionPath>, _>(datas)
                else {
                    panic!("missing IbcState<ConnectionPath>");
                };

                assert!(datas.is_empty());

                mk_event(
                    voyager_message::data::ChannelOpenInit {
                        port_id: event.port_id,
                        channel_id: event.channel_id,
                        counterparty_port_id: event.counterparty_port_id,
                        connection: connection.state,
                        version: event.version,
                    }
                    .into(),
                )
            }
            IbcEvent::ChannelOpenTry(event) => {
                let PluckResult::Found(connection, datas) =
                    pluck::<IbcState<ConnectionPath>, _>(datas)
                else {
                    panic!("missing IbcState<ConnectionPath>");
                };

                assert!(datas.is_empty());

                mk_event(
                    voyager_message::data::ChannelOpenTry {
                        port_id: event.port_id,
                        channel_id: event.channel_id,
                        counterparty_port_id: event.counterparty_port_id,
                        counterparty_channel_id: event.counterparty_channel_id,
                        connection: connection.state,
                        version: event.version,
                    }
                    .into(),
                )
            }
            IbcEvent::ChannelOpenAck(event) => {
                let PluckResult::Found(connection, data) =
                    pluck::<IbcState<ConnectionPath>, _>(datas)
                else {
                    panic!("missing IbcState<ConnectionPath>");
                };

                let PluckResult::Found(channel, datas) = pluck::<IbcState<ChannelEndPath>, _>(data)
                else {
                    panic!("missing IbcState<ChannelEndPath>");
                };

                assert!(datas.is_empty());

                mk_event(
                    voyager_message::data::ChannelOpenAck {
                        port_id: event.port_id,
                        channel_id: event.channel_id,
                        counterparty_port_id: event.counterparty_port_id,
                        counterparty_channel_id: event.counterparty_channel_id,
                        connection: connection.state,
                        version: channel.state.version,
                    }
                    .into(),
                )
            }
            IbcEvent::ChannelOpenConfirm(event) => {
                let PluckResult::Found(connection, data) =
                    pluck::<IbcState<ConnectionPath>, _>(datas)
                else {
                    panic!("missing IbcState<ConnectionPath>");
                };

                let PluckResult::Found(channel, datas) = pluck::<IbcState<ChannelEndPath>, _>(data)
                else {
                    panic!("missing IbcState<ChannelEndPath>");
                };

                assert!(datas.is_empty());

                mk_event(
                    voyager_message::data::ChannelOpenConfirm {
                        port_id: event.port_id,
                        channel_id: event.channel_id,
                        counterparty_port_id: event.counterparty_port_id,
                        counterparty_channel_id: event.counterparty_channel_id,
                        connection: connection.state,
                        version: channel.state.version,
                    }
                    .into(),
                )
            }
            IbcEvent::WriteAcknowledgement(event) => {
                let (source_channel, destination_channel) = mk_packet_metadata::<ModuleData>(
                    datas,
                    client_meta.state.chain_id.clone(),
                    chain_id.clone(),
                    event.packet_src_port,
                    event.packet_src_channel,
                    event.packet_dst_port,
                    event.packet_dst_channel,
                );

                mk_event(
                    voyager_message::data::WriteAcknowledgement {
                        packet_data: event.packet_data_hex,
                        packet: PacketMetadata {
                            sequence: event.packet_sequence,
                            source_channel,
                            destination_channel,
                            timeout_height: event.packet_timeout_height,
                            timeout_timestamp: event.packet_timeout_timestamp,
                        },
                        packet_ack: event.packet_ack_hex,
                    }
                    .into(),
                )
            }
            IbcEvent::RecvPacket(event) => {
                let (source_channel, destination_channel) = mk_packet_metadata::<ModuleData>(
                    datas,
                    client_meta.state.chain_id.clone(),
                    chain_id.clone(),
                    event.packet_src_port,
                    event.packet_src_channel,
                    event.packet_dst_port,
                    event.packet_dst_channel,
                );

                mk_event(
                    voyager_message::data::RecvPacket {
                        packet_data: event.packet_data_hex,
                        packet: PacketMetadata {
                            sequence: event.packet_sequence,
                            source_channel,
                            destination_channel,
                            timeout_height: event.packet_timeout_height,
                            timeout_timestamp: event.packet_timeout_timestamp,
                        },
                    }
                    .into(),
                )
            }
            IbcEvent::SendPacket(event) => {
                let (source_channel, destination_channel) = mk_packet_metadata::<ModuleData>(
                    datas,
                    chain_id.clone(),
                    client_meta.state.chain_id.clone(),
                    event.packet_src_port,
                    event.packet_src_channel,
                    event.packet_dst_port,
                    event.packet_dst_channel,
                );

                mk_event(
                    voyager_message::data::SendPacket {
                        packet_data: event.packet_data_hex,
                        packet: PacketMetadata {
                            sequence: event.packet_sequence,
                            source_channel,
                            destination_channel,
                            timeout_height: event.packet_timeout_height,
                            timeout_timestamp: event.packet_timeout_timestamp,
                        },
                    }
                    .into(),
                )
            }
            IbcEvent::AcknowledgePacket(event) => {
                let (source_channel, destination_channel) = mk_packet_metadata::<ModuleData>(
                    datas,
                    chain_id.clone(),
                    client_meta.state.chain_id.clone(),
                    event.packet_src_port,
                    event.packet_src_channel,
                    event.packet_dst_port,
                    event.packet_dst_channel,
                );

                mk_event(
                    voyager_message::data::AcknowledgePacket {
                        packet: PacketMetadata {
                            sequence: event.packet_sequence,
                            source_channel,
                            destination_channel,
                            timeout_height: event.packet_timeout_height,
                            timeout_timestamp: event.packet_timeout_timestamp,
                        },
                    }
                    .into(),
                )
            }
            IbcEvent::TimeoutPacket(event) => {
                let (source_channel, destination_channel) = mk_packet_metadata::<ModuleData>(
                    datas,
                    chain_id.clone(),
                    client_meta.state.chain_id.clone(),
                    event.packet_src_port,
                    event.packet_src_channel,
                    event.packet_dst_port,
                    event.packet_dst_channel,
                );

                mk_event(
                    voyager_message::data::TimeoutPacket {
                        packet: PacketMetadata {
                            sequence: event.packet_sequence,
                            source_channel,
                            destination_channel,
                            timeout_height: event.packet_timeout_height,
                            timeout_timestamp: event.packet_timeout_timestamp,
                        },
                    }
                    .into(),
                )
            }
        };

        data(event)
    }
}
