use serde::{Deserialize, Serialize};
use unionlabs::{
    events,
    ibc::core::{
        channel::{self, channel::Channel, order::Order, packet::Packet},
        client::height::Height,
        commitment::merkle_path::MerklePath,
        connection::{self, connection_end::ConnectionEnd},
    },
    ics24::{
        AcknowledgementPath, ChannelEndPath, CommitmentPath, ConnectionPath, NextSequenceSendPath,
        ReceiptPath,
    },
    id::{ChannelId, ClientId, ConnectionId, PortId},
};

use crate::{
    Either, IbcAction, IbcError, IbcEvent, IbcHost, IbcMsg, IbcQuery, IbcResponse, IbcVmResponse,
    Runnable, Status,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum RecvPacket {
    Init {
        packet: Packet,
        proof_commitment: Vec<u8>,
        proof_height: Height,
    },

    MembershipVerified {
        packet: Packet,
        channel: Channel,
    },

    CallbackCalled {
        packet: Packet,
        channel: Channel,
    },
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
                    packet,
                    proof_commitment,
                    proof_height,
                },
                &[IbcResponse::Empty],
            ) => {
                let channel: Channel = host
                    .read(
                        &ChannelEndPath {
                            port_id: packet.destination_port.clone(),
                            channel_id: packet.destination_channel.clone(),
                        }
                        .into(),
                    )
                    .ok_or(IbcError::ChannelNotFound(
                        packet.destination_port.clone(),
                        packet.destination_channel.clone(),
                    ))?;

                if channel.state != channel::state::State::Open {
                    return Err(IbcError::IncorrectChannelState(
                        channel.state,
                        channel::state::State::Open,
                    )
                    .into());
                }

                if packet.source_port != channel.counterparty.port_id {
                    return Err(IbcError::SourcePortMismatch(
                        packet.source_port,
                        channel.counterparty.port_id,
                    )
                    .into());
                }

                if &packet.destination_channel != channel.counterparty.channel_id.as_ref().unwrap()
                {
                    return Err(IbcError::SourceChannelMismatch(
                        packet.source_channel,
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
                    .ok_or(IbcError::ConnectionNotFound(
                        channel.connection_hops[0].to_string_prefixed(),
                    ))?;

                if connection.state != connection::state::State::Open {
                    return Err(IbcError::IncorrectConnectionState(
                        connection.state,
                        connection::state::State::Open,
                    )
                    .into());
                }

                if packet.timeout_height == Default::default() && packet.timeout_timestamp == 0 {
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

                // TODO(aeryz): recv start sequence check for replay protection

                match host.read_raw(
                    &ReceiptPath {
                        port_id: packet.destination_port.clone(),
                        channel_id: packet.destination_channel.clone(),
                        sequence: packet.sequence,
                    }
                    .into(),
                ) {
                    Some(_) => Either::Right((
                        vec![IbcEvent::RecvPacket(events::RecvPacket {
                            packet_data_hex: packet.data,
                            packet_timeout_height: packet.timeout_height,
                            packet_timeout_timestamp: packet.timeout_timestamp,
                            packet_sequence: packet.sequence,
                            packet_src_port: packet.source_port,
                            packet_src_channel: packet.source_channel,
                            packet_dst_port: packet.destination_port,
                            packet_dst_channel: packet.destination_channel,
                            packet_channel_ordering: channel.ordering,
                            connection_id: channel.connection_hops[0].clone(),
                        })],
                        IbcVmResponse::Empty,
                    )),
                    None => {
                        // TODO(aeryz): known size can be optimized
                        let packet_commitment = packet_commitment(host, &packet);

                        Either::Left((
                            RecvPacket::MembershipVerified {
                                packet: packet.clone(),
                                channel: channel.clone(),
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
                                            CommitmentPath {
                                                port_id: packet.source_port.clone(),
                                                channel_id: packet.source_channel.clone(),
                                                sequence: packet.sequence,
                                            }
                                            .ics24_commitment_path(),
                                        ],
                                    },
                                    value: host.sha256(packet_commitment),
                                }],
                            )
                                .into(),
                        ))
                    }
                }
            }
            (
                RecvPacket::MembershipVerified { packet, channel },
                &[IbcResponse::VerifyMembership { valid }],
            ) => {
                if !valid {
                    return Err(IbcError::MembershipVerificationFailure.into());
                }

                Either::Left((
                    RecvPacket::CallbackCalled {
                        packet: packet.clone(),
                        channel,
                    },
                    IbcMsg::OnRecvPacket {
                        packet: packet.clone(),
                    }
                    .into(),
                ))
            }
            (
                RecvPacket::CallbackCalled { packet, channel },
                &[IbcResponse::OnRecvPacket { ack }],
            ) => {
                host.commit_raw(
                    ReceiptPath {
                        port_id: packet.destination_port.clone(),
                        channel_id: packet.destination_channel.clone(),
                        sequence: packet.sequence,
                    }
                    .into(),
                    vec![1],
                )?;

                let mut events = vec![IbcEvent::RecvPacket(events::RecvPacket {
                    packet_data_hex: packet.data.clone(),
                    packet_timeout_height: packet.timeout_height,
                    packet_timeout_timestamp: packet.timeout_timestamp,
                    packet_sequence: packet.sequence,
                    packet_src_port: packet.source_port.clone(),
                    packet_src_channel: packet.source_channel.clone(),
                    packet_dst_port: packet.destination_port.clone(),
                    packet_dst_channel: packet.destination_channel.clone(),
                    packet_channel_ordering: channel.ordering,
                    connection_id: channel.connection_hops[0].clone(),
                })];

                if !ack.is_empty() {
                    events.push(IbcEvent::WriteAcknowledgement(write_acknowledgement(
                        host,
                        &channel,
                        packet,
                        ack.clone(),
                    )?));
                }

                Either::Right((events, IbcVmResponse::Empty))
            }
            _ => return Err(IbcError::UnexpectedAction.into()),
        };

        Ok(res)
    }
}

pub fn write_acknowledgement<T: IbcHost>(
    host: &mut T,
    channel: &Channel,
    packet: Packet,
    ack: Vec<u8>,
) -> Result<events::WriteAcknowledgement, T::Error> {
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

    Ok(events::WriteAcknowledgement {
        packet_data_hex: packet.data,
        packet_timeout_height: packet.timeout_height,
        packet_timeout_timestamp: packet.timeout_timestamp,
        packet_sequence: packet.sequence,
        packet_src_port: packet.source_port,
        packet_src_channel: packet.source_channel,
        packet_dst_port: packet.destination_port,
        packet_dst_channel: packet.destination_channel,
        packet_ack_hex: hex::encode(ack).into_bytes(),
        connection_id: channel.connection_hops[0].clone(),
    })
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum SendPacket {
    Init {
        source_port: PortId,
        source_channel: ChannelId,
        timeout_height: Height,
        timeout_timestamp: u64,
        // TODO(aeryz): enforce this to be non-empty at type level
        data: Vec<u8>,
    },

    LatestHeightFetched {
        client_id: ClientId,
        connection_id: ConnectionId,
        source_port: PortId,
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
        source_port: PortId,
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
                    source_port,
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

                let channel: Channel = host
                    .read(
                        &ChannelEndPath {
                            port_id: source_port.clone(),
                            channel_id: source_channel.clone(),
                        }
                        .into(),
                    )
                    .ok_or(IbcError::ChannelNotFound(
                        source_port.clone(),
                        source_channel.clone(),
                    ))?;

                if channel.state != channel::state::State::Open {
                    return Err(IbcError::IncorrectChannelState(
                        channel.state,
                        channel::state::State::Open,
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
                    .ok_or(IbcError::ConnectionNotFound(
                        channel.connection_hops[0].to_string_prefixed(),
                    ))?;

                if connection.state != connection::state::State::Open {
                    return Err(IbcError::IncorrectConnectionState(
                        connection.state,
                        connection::state::State::Open,
                    )
                    .into());
                }

                Either::Left((
                    SendPacket::LatestHeightFetched {
                        client_id: connection.client_id.clone(),
                        source_port,
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
                    source_port,
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
                        source_port,
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
                    source_port,
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

                let sequence_path = NextSequenceSendPath {
                    port_id: source_port.clone(),
                    channel_id: source_channel.clone(),
                }
                .into();

                let sequence =
                    u64::from_be_bytes(host.read_raw(&sequence_path).unwrap().try_into().unwrap());

                let packet = Packet {
                    sequence: sequence.try_into().unwrap(),
                    source_port,
                    source_channel,
                    destination_port,
                    destination_channel,
                    data,
                    timeout_height,
                    timeout_timestamp,
                };

                host.commit_raw(
                    sequence_path,
                    sequence.checked_add(1).unwrap().to_be_bytes().to_vec(),
                )?;
                let commitment = packet_commitment(host, &packet);
                host.commit_raw(
                    CommitmentPath {
                        port_id: packet.source_port.clone(),
                        channel_id: packet.source_channel.clone(),
                        sequence: packet.sequence,
                    }
                    .into(),
                    host.sha256(commitment),
                )?;

                Either::Right((
                    vec![IbcEvent::SendPacket(events::SendPacket {
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

fn packet_commitment<T: IbcHost>(host: &mut T, packet: &Packet) -> Vec<u8> {
    let mut packet_commitment = Vec::new();
    packet_commitment.extend_from_slice(packet.timeout_timestamp.to_be_bytes().as_slice());
    packet_commitment.extend_from_slice(packet.timeout_height.height().to_be_bytes().as_slice());
    packet_commitment.extend_from_slice(packet.timeout_height.height().to_be_bytes().as_slice());
    packet_commitment.extend_from_slice(host.sha256(packet.data.clone()).as_slice());
    packet_commitment
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
                    .ok_or(IbcError::ChannelNotFound(
                        packet.source_port.clone(),
                        packet.source_channel.clone(),
                    ))?;

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

                if &packet.destination_channel != channel.counterparty.channel_id.as_ref().unwrap()
                {
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
                    .ok_or(IbcError::ConnectionNotFound(
                        channel.connection_hops[0].to_string_prefixed(),
                    ))?;

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
                        vec![IbcEvent::AcknowledgePacket(events::AcknowledgePacket {
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
                                    .ics24_commitment_path(),
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
                    vec![IbcEvent::AcknowledgePacket(events::AcknowledgePacket {
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
