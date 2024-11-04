use serde::{Deserialize, Serialize};
use unionlabs::{
    hash::H256,
    ibc::core::{
        channel::{self, channel::Channel, order::Order, packet::Packet},
        client::height::Height,
        commitment::merkle_path::MerklePath,
        connection::{self, connection_end::ConnectionEnd},
    },
    ics24::{
        self, ethabi::COMMITMENT_MAGIC, AcknowledgementPath, ChannelEndPath, CommitmentPath,
        ConnectionPath, NextSequenceSendPath, ReceiptPath,
    },
    id::{ChannelId, ClientId, ConnectionId, PortId},
};

use super::connection_handshake::ensure_connection_state;
use crate::{
    Either, IbcAction, IbcError, IbcEvent, IbcHost, IbcMsg, IbcQuery, IbcResponse, IbcVmResponse,
    Runnable, Status,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum RecvPacket {
    Init {
        packets: Vec<Packet>,
        relayer_msgs: Vec<Vec<u8>>,
        relayer: Vec<u8>,
        proof_commitment: Vec<u8>,
        proof_height: Height,
        intent: bool,
    },

    MembershipVerified {
        packets: Vec<Packet>,
        relayer_msgs: Vec<Vec<u8>>,
        relayer: Vec<u8>,
        channel: Channel,
        intent: bool,
    },

    CallbackCalled {
        packets: Vec<Packet>,
        channel: Channel,
    },
}

fn ensure_channel_state<T: IbcHost>(
    ibc_host: &T,
    channel_id: ChannelId,
) -> Result<Channel, IbcError> {
    let channel: Channel = ibc_host
        .read2(ics24::ethabi::channel_key(channel_id.id()).as_ref())
        .ok_or(IbcError::ChannelNotFound(channel_id))?;

    if channel.state != channel::state::State::Open {
        return Err(IbcError::IncorrectChannelState(
            channel.state,
            channel::state::State::Open,
        ));
    }

    Ok(channel)
}

impl<T: IbcHost> Runnable<T> for RecvPacket {
    fn process(
        self,
        host: &mut T,
        resp: &[IbcResponse],
    ) -> Result<Either<(Self, IbcAction), (Vec<IbcEvent>, IbcVmResponse)>, <T as IbcHost>::Error>
    {
        let res = match (self, &resp) {
            (
                RecvPacket::Init {
                    packets,
                    relayer_msgs,
                    relayer,
                    proof_commitment,
                    proof_height,
                    intent,
                },
                &[IbcResponse::Empty],
            ) => {
                if packets.is_empty() {
                    return Err(IbcError::EmptyPacketsReceived.into());
                }

                let destination_channel = packets[0].destination_channel;
                let (source_channel, source_port) =
                    (packets[0].source_channel, &packets[0].source_port.clone());

                let channel = ensure_channel_state(host, destination_channel)?;

                let _proof_commitment_key = match packets.len() {
                    1 => ics24::ethabi::batch_receipts_commitment_key(
                        source_channel.id(),
                        commit_packet(host, &packets[0]),
                    ),
                    _ => ics24::ethabi::batch_receipts_commitment_key(
                        source_channel.id(),
                        commit_packets(host, &packets),
                    ),
                };

                let connection = ensure_connection_state(host, channel.connection_hops[0])?;
                let sequence = packets[0].sequence;

                if intent {
                    // bypassing the membership verification if there's intents
                    RecvPacket::MembershipVerified {
                        packets,
                        channel,
                        intent,
                        relayer_msgs,
                        relayer,
                    }
                    .process(host, &[IbcResponse::VerifyMembership { valid: true }])?
                } else {
                    Either::Left((
                        RecvPacket::MembershipVerified {
                            packets,
                            channel,
                            intent,
                            relayer_msgs,
                            relayer,
                        },
                        (
                            connection.client_id,
                            vec![IbcQuery::VerifyMembership {
                                height: proof_height,
                                delay_time_period: 0,
                                delay_block_period: 0,
                                proof: proof_commitment,
                                path: MerklePath {
                                    key_path: vec![
                                        "ibc".into(),
                                        // TODO(aeryz): ditch this
                                        CommitmentPath {
                                            port_id: source_port.clone(),
                                            channel_id: source_channel,
                                            sequence,
                                        }
                                        .to_string(),
                                    ],
                                },
                                // TODO(aeryz): leave this as  H256
                                value: COMMITMENT_MAGIC.into(),
                            }],
                        )
                            .into(),
                    ))
                }
            }
            (
                RecvPacket::MembershipVerified {
                    packets,
                    channel,
                    intent,
                    relayer_msgs,
                    relayer,
                },
                &[IbcResponse::VerifyMembership { valid }],
            ) => {
                if !valid {
                    return Err(IbcError::MembershipVerificationFailure.into());
                }

                let destination_channel = packets[0].destination_channel;

                let mut recv_callbacks = vec![];

                for (i, packet) in packets.iter().enumerate() {
                    if packet.source_port != channel.counterparty.port_id {
                        return Err(IbcError::SourcePortMismatch(
                            packet.source_port.clone(),
                            channel.counterparty.port_id,
                        )
                        .into());
                    }

                    if Some(&packet.source_channel) != channel.counterparty.channel_id.as_ref() {
                        return Err(IbcError::SourceChannelMismatch(
                            packet.source_channel,
                            channel.counterparty.channel_id.unwrap(),
                        )
                        .into());
                    }

                    if packet.timeout_height == Default::default() && packet.timeout_timestamp == 0
                    {
                        return Err(IbcError::ZeroTimeout.into());
                    }

                    if packet.timeout_height != Default::default()
                        && host.current_height() >= packet.timeout_height
                    {
                        return Err(IbcError::TimedOutPacket.into());
                    }

                    if packet.timeout_timestamp != 0
                        && host.current_timestamp() >= packet.timeout_timestamp
                    {
                        return Err(IbcError::TimedOutPacket.into());
                    }

                    let commitment_key = ics24::ethabi::batch_receipts_commitment_key(
                        destination_channel.id(),
                        commit_packet(host, &packet),
                    );

                    let already_received = match channel.ordering {
                        Order::Unordered => set_packet_receive(commitment_key),
                        Order::Ordered => {
                            if intent {
                                return Err(IbcError::IntentOrderedPacket.into());
                            }
                            set_next_sequence_recv(destination_channel, packet.sequence)
                        }
                        _ => panic!("not possible"),
                    };

                    if !already_received {
                        if intent {
                            recv_callbacks.push(IbcMsg::OnRecvIntentPacket {
                                packet: packet.clone(),
                                maker: relayer.clone(),
                                maker_msg: relayer_msgs[i].clone(),
                            })
                        } else {
                            recv_callbacks.push(IbcMsg::OnRecvPacket {
                                packet: packet.clone(),
                            });
                        }
                    }
                }

                Either::Left((
                    RecvPacket::CallbackCalled { packets, channel },
                    recv_callbacks.into(),
                ))
            }
            (
                RecvPacket::CallbackCalled { packets, channel },
                &[IbcResponse::OnRecvPacket { acks }],
            ) => {
                let mut events = vec![];
                for (ack, packet) in acks.into_iter().zip(packets) {
                    events.push(IbcEvent::RecvPacket(ibc_events::RecvPacket {
                        packet_data_hex: packet.data.clone(),
                        packet_timeout_height: packet.timeout_height,
                        packet_timeout_timestamp: packet.timeout_timestamp,
                        packet_sequence: packet.sequence,
                        packet_src_port: packet.source_port.clone(),
                        packet_src_channel: packet.source_channel,
                        packet_dst_port: packet.destination_port.clone(),
                        packet_dst_channel: packet.destination_channel,
                        packet_channel_ordering: channel.ordering,
                        connection_id: channel.connection_hops[0],
                    }));

                    if !ack.is_empty() {
                        events.push(IbcEvent::WriteAcknowledgement(write_acknowledgement(
                            host,
                            &channel,
                            packet,
                            ack.clone(),
                        )?));
                    }
                }
                Either::Right((events, IbcVmResponse::Empty))
            }
            _ => return Err(IbcError::UnexpectedAction.into()),
        };

        Ok(res)
    }
}

fn set_next_sequence_recv(
    destination_channel: ChannelId,
    sequence: std::num::NonZero<u64>,
) -> bool {
    todo!()
}

fn set_packet_receive(commitment_key: unionlabs::hash::hash_v2::Hash<32>) -> bool {
    todo!()
}

pub fn write_acknowledgement<T: IbcHost>(
    host: &mut T,
    channel: &Channel,
    packet: Packet,
    ack: Vec<u8>,
) -> Result<ibc_events::WriteAcknowledgement, T::Error> {
    let ack_key = AcknowledgementPath {
        port_id: packet.destination_port.clone(),
        channel_id: packet.destination_channel.clone(),
        sequence: packet.sequence,
    }
    .into();
    if host.read_raw(&ack_key).is_some() {
        return Err(IbcError::AcknowledgementExists(packet.sequence.into()).into());
    }

    if ack.is_empty() {
        return Err(IbcError::EmptyAcknowledgement.into());
    }

    host.commit_raw(ack_key, host.sha256(ack.clone()))?;

    Ok(ibc_events::WriteAcknowledgement {
        packet_data_hex: packet.data,
        packet_timeout_height: packet.timeout_height,
        packet_timeout_timestamp: packet.timeout_timestamp,
        packet_sequence: packet.sequence,
        packet_src_port: packet.source_port,
        packet_src_channel: packet.source_channel,
        packet_dst_port: packet.destination_port,
        packet_dst_channel: packet.destination_channel,
        packet_ack_hex: hex::encode(ack).into_bytes().into(),
        connection_id: channel.connection_hops[0].clone(),
    })
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum SendPacket {
    Init {
        source_channel: ChannelId,
        timeout_height: Height,
        timeout_timestamp: u64,
        // TODO(aeryz): enforce this to be non-empty at type level
        data: Vec<u8>,
    },

    LatestHeightFetched {
        client_id: ClientId,
        connection_id: ConnectionId,
        source_channel: ChannelId,
        destination_port: PortId,
        destination_channel: ChannelId,
        timeout_height: Height,
        timeout_timestamp: u64,
        data: Vec<u8>,
    },

    TimestampFetched {
        height: Height,
        connection_id: ConnectionId,
        source_channel: ChannelId,
        destination_port: PortId,
        destination_channel: ChannelId,
        timeout_height: Height,
        timeout_timestamp: u64,
        data: Vec<u8>,
    },
}

impl<T: IbcHost> Runnable<T> for SendPacket {
    fn process(
        self,
        host: &mut T,
        resp: &[IbcResponse],
    ) -> Result<Either<(Self, IbcAction), (Vec<IbcEvent>, IbcVmResponse)>, <T as IbcHost>::Error>
    {
        let res = match (self, resp) {
            (
                SendPacket::Init {
                    source_channel,
                    timeout_height,
                    timeout_timestamp,
                    data,
                },
                &[IbcResponse::Empty],
            ) => {
                if timeout_height == Default::default() && timeout_timestamp == 0 {
                    return Err(IbcError::ZeroTimeout.into());
                }

                // TODO(aeryz): authenticate channel owner here

                let channel = ensure_channel_state(host, source_channel)?;

                let connection = ensure_connection_state(host, channel.connection_hops[0])?;

                Either::Left((
                    SendPacket::LatestHeightFetched {
                        client_id: connection.client_id.clone(),
                        source_channel,
                        timeout_height,
                        timeout_timestamp,
                        destination_port: channel.counterparty.port_id,
                        destination_channel: channel.counterparty.channel_id.unwrap(),
                        data,
                        connection_id: channel.connection_hops[0].clone(),
                    },
                    (
                        connection.client_id,
                        vec![IbcQuery::Status, IbcQuery::LatestHeight],
                    )
                        .into(),
                ))
            }
            (
                SendPacket::LatestHeightFetched {
                    client_id,
                    source_channel,
                    timeout_height,
                    timeout_timestamp,
                    destination_port: destination_port_id,
                    destination_channel,
                    data,
                    connection_id,
                },
                &[IbcResponse::Status { status }, IbcResponse::LatestHeight { height }],
            ) => {
                if status != Status::Active {
                    return Err(IbcError::NotActive(client_id, status).into());
                }
                Either::Left((
                    SendPacket::TimestampFetched {
                        height,
                        source_channel,
                        timeout_height,
                        timeout_timestamp,
                        destination_port: destination_port_id,
                        destination_channel,
                        data,
                        connection_id,
                    },
                    (client_id, vec![IbcQuery::TimestampAtHeight(height)]).into(),
                ))
            }
            (
                SendPacket::TimestampFetched {
                    height,
                    source_channel,
                    timeout_height,
                    timeout_timestamp,
                    destination_port,
                    destination_channel,
                    data,
                    connection_id,
                },
                &[IbcResponse::TimestampAtHeight { timestamp }],
            ) => {
                // TODO(aeryz): if the timestamp is not specified, we don't need to fetch it. could be a nice optimization.
                if timeout_height != Default::default() && height >= timeout_height {
                    return Err(IbcError::TimedOutPacket.into());
                }

                if timeout_timestamp != 0 && timestamp >= timeout_timestamp {
                    return Err(IbcError::TimedOutPacket.into());
                }

                let sequence: u64 = generate_packet_sequence(host, source_channel)?;

                let packet = Packet {
                    sequence: sequence.try_into().unwrap(),
                    // TODO(aeryz): what about this?
                    source_port: PortId::new("00").unwrap(),
                    source_channel,
                    destination_port,
                    destination_channel,
                    data: data.into(),
                    timeout_height,
                    timeout_timestamp,
                };

                host.commit_raw2(
                    ics24::ethabi::batch_receipts_commitment_key(
                        source_channel.id(),
                        commit_packet(host, &packet),
                    )
                    .as_ref(),
                    COMMITMENT_MAGIC.into_bytes(),
                )?;

                Either::Right((
                    vec![IbcEvent::SendPacket(ibc_events::SendPacket {
                        packet_data_hex: packet.data,
                        packet_timeout_height: timeout_height,
                        packet_timeout_timestamp: timeout_timestamp,
                        packet_sequence: packet.sequence,
                        packet_src_port: packet.source_port,
                        packet_src_channel: packet.source_channel,
                        packet_dst_port: packet.destination_port,
                        packet_dst_channel: packet.destination_channel,
                        packet_channel_ordering: Order::Unordered,
                        connection_id,
                    })],
                    IbcVmResponse::SendPacket {
                        sequence: packet.sequence.into(),
                    },
                ))
            }
            _ => return Err(IbcError::UnexpectedAction.into()),
        };
        Ok(res)
    }
}

fn generate_packet_sequence<T: IbcHost>(
    host: &mut T,
    channel_id: ChannelId,
) -> Result<u64, T::Error> {
    let commitment_key = ics24::ethabi::next_seq_send_key(channel_id.id());

    // TODO(aeryz): make this u256
    let seq = u64::from_be_bytes(
        host.read_raw2(commitment_key.as_ref())
            .expect("impossible")
            .as_slice()
            .try_into()
            .expect("impossible"),
    );

    host.commit_raw2(commitment_key.as_ref(), (seq + 1).to_be_bytes().to_vec())?;

    Ok(seq)
}

fn packet_commitment<T: IbcHost>(host: &mut T, packet: &Packet) -> Vec<u8> {
    let mut packet_commitment = Vec::new();
    packet_commitment.extend_from_slice(packet.timeout_timestamp.to_be_bytes().as_slice());
    packet_commitment.extend_from_slice(packet.timeout_height.revision().to_be_bytes().as_slice());
    packet_commitment.extend_from_slice(packet.timeout_height.height().to_be_bytes().as_slice());
    packet_commitment.extend_from_slice(host.sha256(packet.data.clone().into_vec()).as_slice());
    packet_commitment
}

fn commit_packet<T: IbcHost>(_host: &T, packet: &Packet) -> H256 {
    Default::default()
}

fn commit_packets<T: IbcHost>(_host: &T, packet: &[Packet]) -> H256 {
    Default::default()
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Acknowledgement {
    Init {
        packet: Packet,
        ack: Vec<u8>,
        proof_ack: Vec<u8>,
        proof_height: Height,
    },

    MembershipVerified {
        packet: Packet,
        ack: Vec<u8>,
        connection_id: ConnectionId,
    },

    CallbackCalled {
        packet: Packet,
        connection_id: ConnectionId,
    },
}

impl<T: IbcHost> Runnable<T> for Acknowledgement {
    fn process(
        self,
        host: &mut T,
        resp: &[IbcResponse],
    ) -> Result<Either<(Self, IbcAction), (Vec<IbcEvent>, IbcVmResponse)>, <T as IbcHost>::Error>
    {
        let res = match (self, &resp) {
            (
                Acknowledgement::Init {
                    packet,
                    ack,
                    proof_ack,
                    proof_height,
                },
                &[IbcResponse::Empty],
            ) => {
                let channel: Channel = host
                    .read(
                        &ChannelEndPath {
                            port_id: packet.source_port.clone(),
                            channel_id: packet.source_channel.clone(),
                        }
                        .into(),
                    )
                    .ok_or(IbcError::ChannelNotFound(packet.source_channel.clone()))?;

                // TODO(aeryz): flushing state?
                if channel.state != channel::state::State::Open {
                    return Err(IbcError::IncorrectChannelState(
                        channel.state,
                        channel::state::State::Open,
                    )
                    .into());
                }

                if packet.destination_port != channel.counterparty.port_id {
                    return Err(IbcError::DestinationPortMismatch(
                        packet.destination_port,
                        channel.counterparty.port_id,
                    )
                    .into());
                }

                if Some(&packet.destination_channel) != channel.counterparty.channel_id.as_ref() {
                    return Err(IbcError::DestinationChannelMismatch(
                        packet.destination_channel,
                        channel.counterparty.channel_id.unwrap(),
                    )
                    .into());
                }

                let connection: ConnectionEnd = host
                    .read(
                        &ConnectionPath {
                            connection_id: channel.connection_hops[0].clone(),
                        }
                        .into(),
                    )
                    .ok_or(IbcError::ConnectionNotFound(channel.connection_hops[0]))?;

                if connection.state != connection::state::State::Open {
                    return Err(IbcError::IncorrectConnectionState(
                        connection.state,
                        connection::state::State::Open,
                    )
                    .into());
                }

                let Some(commitment) = host.read_raw(
                    &CommitmentPath {
                        port_id: packet.source_port.clone(),
                        channel_id: packet.source_channel.clone(),
                        sequence: packet.sequence,
                    }
                    .into(),
                ) else {
                    return Ok(Either::Right((
                        vec![IbcEvent::AcknowledgePacket(ibc_events::AcknowledgePacket {
                            packet_timeout_height: packet.timeout_height,
                            packet_timeout_timestamp: packet.timeout_timestamp,
                            packet_sequence: packet.sequence,
                            packet_src_port: packet.source_port,
                            packet_src_channel: packet.source_channel,
                            packet_dst_port: packet.destination_port,
                            packet_dst_channel: packet.destination_channel,
                            packet_channel_ordering: Order::Unordered,
                            connection_id: channel.connection_hops[0].clone(),
                        })],
                        IbcVmResponse::Empty,
                    )));
                };

                let packet_commitment = packet_commitment(host, &packet);
                let packet_commitment = host.sha256(packet_commitment);
                if commitment != packet_commitment {
                    return Err(
                        IbcError::PacketCommitmentMismatch(commitment, packet_commitment).into(),
                    );
                }

                Either::Left((
                    Acknowledgement::MembershipVerified {
                        packet: packet.clone(),
                        connection_id: channel.connection_hops[0].clone(),
                        ack: ack.clone(),
                    },
                    (
                        connection.client_id,
                        vec![IbcQuery::VerifyMembership {
                            height: proof_height,
                            delay_time_period: 0,
                            delay_block_period: 0,
                            proof: proof_ack,
                            path: MerklePath {
                                key_path: vec![
                                    "ibc".to_string(),
                                    AcknowledgementPath {
                                        port_id: packet.destination_port,
                                        channel_id: packet.destination_channel,
                                        sequence: packet.sequence,
                                    }
                                    .to_string(),
                                ],
                            },
                            value: host.sha256(ack),
                        }],
                    )
                        .into(),
                ))
            }
            (
                Acknowledgement::MembershipVerified {
                    packet,
                    ack,
                    connection_id,
                },
                &[IbcResponse::VerifyMembership { valid }],
            ) => {
                if !valid {
                    return Err(IbcError::MembershipVerificationFailure.into());
                }

                Either::Left((
                    Acknowledgement::CallbackCalled {
                        packet: packet.clone(),
                        connection_id,
                    },
                    IbcMsg::OnAcknowledgePacket { packet, ack }.into(),
                ))
            }
            (
                Acknowledgement::CallbackCalled {
                    packet,
                    connection_id,
                },
                &[IbcResponse::OnAcknowledgePacket { err }],
            ) => {
                if let Some(err) = err {
                    return Err(IbcError::IbcAppCallbackFailed(err.clone()).into());
                }

                host.delete(
                    &CommitmentPath {
                        port_id: packet.source_port.clone(),
                        channel_id: packet.source_channel.clone(),
                        sequence: packet.sequence,
                    }
                    .into(),
                )?;

                Either::Right((
                    vec![IbcEvent::AcknowledgePacket(ibc_events::AcknowledgePacket {
                        packet_timeout_height: packet.timeout_height,
                        packet_timeout_timestamp: packet.timeout_timestamp,
                        packet_sequence: packet.sequence,
                        packet_src_port: packet.source_port,
                        packet_src_channel: packet.source_channel,
                        packet_dst_port: packet.destination_port,
                        packet_dst_channel: packet.destination_channel,
                        packet_channel_ordering: Order::Unordered,
                        connection_id,
                    })],
                    IbcVmResponse::Empty,
                ))
            }
            _ => return Err(IbcError::UnexpectedAction.into()),
        };

        Ok(res)
    }
}
