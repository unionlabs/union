use serde::{Deserialize, Serialize};
use unionlabs::{
    encoding::{EncodeAs, Proto},
    ibc::core::{
        channel::{self, channel::Channel, counterparty::Counterparty, order::Order},
        client::height::Height,
        commitment::merkle_path::MerklePath,
        connection,
    },
    ics24::{
        ChannelEndPath, ConnectionPath, NextSequenceAckPath, NextSequenceRecvPath,
        NextSequenceSendPath,
    },
    id::{ChannelId, ClientId, ConnectionId, PortId},
    validated::ValidateT,
};

use crate::{
    states::connection_handshake::ConnectionEnd, Either, IbcError, IbcEvent, IbcHost, IbcMsg,
    IbcResponse, Runnable, Status,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ChannelOpenInit {
    Init {
        connection_hops: Vec<ConnectionId>,
        port_id: PortId,
        counterparty: Counterparty,
        version: String,
    },

    StatusFetched {
        client_id: ClientId,
        connection_hops: Vec<ConnectionId>,
        port_id: PortId,
        counterparty: Counterparty,
        version: String,
    },

    CallbackCalled {
        channel_id: ChannelId,
        connection_hops: Vec<ConnectionId>,
        port_id: PortId,
        counterparty: Counterparty,
        version: String,
    },
}

impl<T: IbcHost> Runnable<T> for ChannelOpenInit {
    fn process(
        self,
        host: &mut T,
        resp: IbcResponse,
    ) -> Result<Either<(Self, IbcMsg), IbcEvent>, <T as IbcHost>::Error> {
        let res = match (self, resp) {
            (
                ChannelOpenInit::Init {
                    connection_hops,
                    port_id,
                    counterparty,
                    version,
                },
                IbcResponse::Empty,
            ) => {
                let connection: ConnectionEnd = host
                    .read(
                        &ConnectionPath {
                            connection_id: connection_hops[0].clone(),
                        }
                        .into(),
                    )
                    .ok_or(IbcError::ConnectionNotFound(connection_hops[0].to_string()))?;

                if connection.state != connection::state::State::Open {
                    return Err(IbcError::IncorrectConnectionState(
                        connection.state,
                        connection::state::State::Open,
                    )
                    .into());
                }

                // TODO(aeryz): check if port_id is a valid addr here?

                Either::Left((
                    ChannelOpenInit::StatusFetched {
                        client_id: connection.client_id.clone(),
                        connection_hops,
                        port_id,
                        counterparty,
                        version,
                    },
                    IbcMsg::Status {
                        client_id: connection.client_id,
                    },
                ))
            }
            (
                ChannelOpenInit::StatusFetched {
                    client_id,
                    connection_hops,
                    port_id,
                    counterparty,
                    version,
                },
                IbcResponse::Status { status },
            ) => {
                if status != Status::Active {
                    return Err(IbcError::NotActive(client_id, status).into());
                }

                let channel_id = host.next_channel_identifier()?;

                Either::Left((
                    ChannelOpenInit::CallbackCalled {
                        channel_id: channel_id.clone(),
                        connection_hops: connection_hops.clone(),
                        port_id: port_id.clone(),
                        counterparty: counterparty.clone(),
                        version: version.clone(),
                    },
                    IbcMsg::OnChannelOpenInit {
                        order: Order::Unordered,
                        connection_hops,
                        port_id,
                        channel_id,
                        counterparty,
                        version,
                    },
                ))
            }
            (
                ChannelOpenInit::CallbackCalled {
                    channel_id,
                    connection_hops,
                    port_id,
                    counterparty,
                    version,
                },
                IbcResponse::OnChannelOpenAck { err },
            ) => {
                if err {
                    return Err(IbcError::IbcAppCallbackFailed.into());
                }

                let one = 1_u64.to_be_bytes().to_vec();

                let channel = Channel {
                    state: channel::state::State::Init,
                    ordering: Order::Unordered,
                    counterparty: counterparty.clone(),
                    connection_hops: connection_hops.clone(),
                    version: version.clone(),
                };

                host.commit(
                    ChannelEndPath {
                        port_id: port_id.clone(),
                        channel_id: channel_id.clone(),
                    }
                    .into(),
                    channel,
                )?;

                host.commit_raw(
                    NextSequenceSendPath {
                        port_id: port_id.clone(),
                        channel_id: channel_id.clone(),
                    }
                    .into(),
                    one.clone(),
                )?;
                host.commit_raw(
                    NextSequenceRecvPath {
                        port_id: port_id.clone(),
                        channel_id: channel_id.clone(),
                    }
                    .into(),
                    one.clone(),
                )?;
                host.commit_raw(
                    NextSequenceAckPath {
                        port_id: port_id.clone(),
                        channel_id: channel_id.clone(),
                    }
                    .into(),
                    one,
                )?;

                Either::Right(IbcEvent::ChannelOpenInit {
                    port_id,
                    channel_id,
                    counterparty_port_id: counterparty.port_id,
                    connection_id: connection_hops[0].clone().to_string(),
                    version,
                })
            }
            _ => return Err(IbcError::UnexpectedAction.into()),
        };

        Ok(res)
    }
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ChannelOpenTry {
    Init {
        connection_hops: Vec<ConnectionId>,
        port_id: PortId,
        counterparty: channel::counterparty::Counterparty,
        counterparty_version: String,
        version: String,
        proof_init: Vec<u8>,
        proof_height: Height,
    },

    StatusFetched {
        client_id: ClientId,
        connection_hops: Vec<ConnectionId>,
        port_id: PortId,
        counterparty_connection_id: String,
        counterparty: channel::counterparty::Counterparty,
        counterparty_version: String,
        version: String,
        proof_init: Vec<u8>,
        proof_height: Height,
    },

    MembershipVerified {
        connection_hops: Vec<ConnectionId>,
        port_id: PortId,
        counterparty: channel::counterparty::Counterparty,
        version: String,
        counterparty_version: String,
    },

    CallbackCalled {
        channel_id: ChannelId,
        connection_hops: Vec<ConnectionId>,
        port_id: PortId,
        counterparty: channel::counterparty::Counterparty,
        version: String,
    },
}

impl<T: IbcHost> Runnable<T> for ChannelOpenTry {
    fn process(
        self,
        host: &mut T,
        resp: IbcResponse,
    ) -> Result<Either<(Self, IbcMsg), IbcEvent>, <T as IbcHost>::Error> {
        let res = match (self, resp) {
            (
                ChannelOpenTry::Init {
                    connection_hops,
                    port_id,
                    counterparty,
                    counterparty_version,
                    version,
                    proof_init,
                    proof_height,
                },
                IbcResponse::Empty,
            ) => {
                let connection: ConnectionEnd = host
                    .read(
                        &ConnectionPath {
                            connection_id: connection_hops[0].clone(),
                        }
                        .into(),
                    )
                    .ok_or(IbcError::ConnectionNotFound(connection_hops[0].to_string()))?;

                if connection.state != connection::state::State::Open {
                    return Err(IbcError::IncorrectConnectionState(
                        connection.state,
                        connection::state::State::Open,
                    )
                    .into());
                }

                // TODO(aeryz): check if port_id is a valid addr here?

                Either::Left((
                    ChannelOpenTry::StatusFetched {
                        client_id: connection.client_id.clone(),
                        counterparty_connection_id: connection.counterparty.connection_id,
                        connection_hops,
                        port_id,
                        counterparty,
                        version,
                        counterparty_version,
                        proof_init,
                        proof_height,
                    },
                    IbcMsg::Status {
                        client_id: connection.client_id,
                    },
                ))
            }
            (
                ChannelOpenTry::StatusFetched {
                    connection_hops,
                    port_id,
                    counterparty,
                    version,
                    counterparty_version,
                    proof_init,
                    proof_height,
                    client_id,
                    counterparty_connection_id,
                },
                IbcResponse::Status { status },
            ) => {
                if status != Status::Active {
                    return Err(IbcError::NotActive(client_id, status).into());
                }

                let expected_channel = Channel {
                    state: channel::state::State::Init,
                    ordering: Order::Unordered,
                    counterparty: channel::counterparty::Counterparty {
                        port_id: port_id.clone(),
                        channel_id: "".to_string(),
                    },
                    connection_hops: vec![counterparty_connection_id.validate().unwrap()],
                    version: counterparty_version.clone(),
                };

                Either::Left((
                    ChannelOpenTry::MembershipVerified {
                        connection_hops,
                        port_id,
                        counterparty: counterparty.clone(),
                        counterparty_version,
                        version,
                    },
                    IbcMsg::VerifyMembership {
                        client_id,
                        height: proof_height,
                        delay_time_period: 0,
                        delay_block_period: 0,
                        proof: proof_init,
                        path: MerklePath {
                            key_path: vec![
                                "ibc".to_string(),
                                format!(
                                    "channelEnds/ports/{}/channels/{}",
                                    counterparty.port_id, counterparty.channel_id
                                ),
                            ],
                        },
                        value: expected_channel.encode_as::<Proto>(),
                    },
                ))
            }
            (
                ChannelOpenTry::MembershipVerified {
                    connection_hops,
                    port_id,
                    counterparty,
                    version,
                    counterparty_version,
                },
                IbcResponse::VerifyMembership { valid },
            ) => {
                if !valid {
                    return Err(IbcError::MembershipVerificationFailure.into());
                }

                let channel_id = host.next_channel_identifier()?;

                Either::Left((
                    ChannelOpenTry::CallbackCalled {
                        channel_id: channel_id.clone(),
                        connection_hops: connection_hops.clone(),
                        port_id: port_id.clone(),
                        counterparty: counterparty.clone(),
                        version: version.clone(),
                    },
                    IbcMsg::OnChannelOpenTry {
                        order: Order::Unordered,
                        connection_hops,
                        port_id,
                        channel_id,
                        counterparty,
                        counterparty_version,
                    },
                ))
            }
            (
                ChannelOpenTry::CallbackCalled {
                    channel_id,
                    connection_hops,
                    port_id,
                    counterparty,
                    version,
                },
                IbcResponse::OnChannelOpenTry { err },
            ) => {
                if err {
                    return Err(IbcError::IbcAppCallbackFailed.into());
                }

                let one = 1_u64.to_be_bytes().to_vec();

                let channel = Channel {
                    state: channel::state::State::Tryopen,
                    ordering: Order::Unordered,
                    counterparty: counterparty.clone(),
                    connection_hops: connection_hops.clone(),
                    version: version.clone(),
                };

                host.commit(
                    ChannelEndPath {
                        port_id: port_id.clone(),
                        channel_id: channel_id.clone(),
                    }
                    .into(),
                    channel,
                )?;

                host.commit_raw(
                    NextSequenceSendPath {
                        port_id: port_id.clone(),
                        channel_id: channel_id.clone(),
                    }
                    .into(),
                    one.clone(),
                )?;
                host.commit_raw(
                    NextSequenceRecvPath {
                        port_id: port_id.clone(),
                        channel_id: channel_id.clone(),
                    }
                    .into(),
                    one.clone(),
                )?;
                host.commit_raw(
                    NextSequenceAckPath {
                        port_id: port_id.clone(),
                        channel_id: channel_id.clone(),
                    }
                    .into(),
                    one,
                )?;

                Either::Right(IbcEvent::ChannelOpenTry {
                    port_id,
                    channel_id,
                    counterparty_port_id: counterparty.port_id,
                    counterparty_channel_id: counterparty.channel_id,
                    connection_id: connection_hops[0].clone().to_string(),
                    version,
                })
            }
            _ => return Err(IbcError::UnexpectedAction.into()),
        };

        Ok(res)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ChannelOpenAck {
    Init {
        channel_id: ChannelId,
        port_id: PortId,
        counterparty_channel_id: String,
        counterparty_version: String,
        proof_try: Vec<u8>,
        proof_height: Height,
    },

    StatusFetched {
        client_id: ClientId,
        counterparty_connection_id: String,
        channel_id: ChannelId,
        port_id: PortId,
        counterparty_channel_id: String,
        counterparty_port_id: PortId,
        counterparty_version: String,
        proof_try: Vec<u8>,
        proof_height: Height,
    },

    MembershipVerified {
        channel_id: ChannelId,
        port_id: PortId,
        counterparty_channel_id: String,
        counterparty_version: String,
    },

    CallbackCalled {
        channel_id: ChannelId,
        port_id: PortId,
        counterparty_channel_id: String,
        counterparty_version: String,
    },
}

impl<T: IbcHost> Runnable<T> for ChannelOpenAck {
    fn process(
        self,
        host: &mut T,
        resp: IbcResponse,
    ) -> Result<Either<(Self, IbcMsg), IbcEvent>, <T as IbcHost>::Error> {
        let res = match (self, resp) {
            (
                ChannelOpenAck::Init {
                    channel_id,
                    port_id,
                    counterparty_channel_id,
                    counterparty_version,
                    proof_try,
                    proof_height,
                },
                IbcResponse::Empty,
            ) => {
                let channel: Channel = host
                    .read(
                        &ChannelEndPath {
                            port_id: port_id.clone(),
                            channel_id: channel_id.clone(),
                        }
                        .into(),
                    )
                    .ok_or(IbcError::ChannelNotFound(port_id.clone(), channel_id.clone()).into())?;

                if channel.state != channel::state::State::Init {
                    return Err(IbcError::IncorrectChannelState(
                        channel.state,
                        channel::state::State::Init,
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
                        channel.connection_hops[0].to_string(),
                    ))?;

                if connection.state != connection::state::State::Open {
                    return Err(IbcError::IncorrectConnectionState(
                        connection.state,
                        connection::state::State::Open,
                    )
                    .into());
                }

                // TODO(aeryz): check if port_id is a valid addr here?

                Either::Left((
                    ChannelOpenAck::StatusFetched {
                        channel_id,
                        port_id,
                        counterparty_channel_id,
                        counterparty_version,
                        client_id: connection.client_id.clone(),
                        counterparty_connection_id: connection.counterparty.connection_id,
                        proof_try,
                        proof_height,
                        counterparty_port_id: channel.counterparty.port_id,
                    },
                    IbcMsg::Status {
                        client_id: connection.client_id,
                    },
                ))
            }
            (
                ChannelOpenAck::StatusFetched {
                    channel_id,
                    port_id,
                    counterparty_channel_id,
                    counterparty_version,
                    client_id,
                    counterparty_connection_id,
                    proof_try,
                    proof_height,
                    counterparty_port_id,
                },
                IbcResponse::Status { status },
            ) => {
                if status != Status::Active {
                    return Err(IbcError::NotActive(client_id, status).into());
                }

                let expected_channel = Channel {
                    state: channel::state::State::Tryopen,
                    ordering: Order::Unordered,
                    counterparty: channel::counterparty::Counterparty {
                        // TODO(aeryz): make port id a validater type
                        port_id: port_id.clone(),
                        channel_id: channel_id.clone().to_string(),
                    },
                    connection_hops: vec![counterparty_connection_id.validate().unwrap()],
                    version: counterparty_version.clone(),
                };

                Either::Left((
                    ChannelOpenAck::MembershipVerified {
                        channel_id,
                        port_id,
                        counterparty_channel_id: counterparty_channel_id.clone(),
                        counterparty_version,
                    },
                    IbcMsg::VerifyMembership {
                        client_id,
                        height: proof_height,
                        delay_time_period: 0,
                        delay_block_period: 0,
                        proof: proof_try,
                        path: MerklePath {
                            key_path: vec![
                                "ibc".to_string(),
                                format!(
                                    "channelEnds/ports/{counterparty_port_id}/channels/{counterparty_channel_id}",
                                ),
                            ],
                        },
                        value: expected_channel.encode_as::<Proto>(),
                    },
                ))
            }
            (
                ChannelOpenAck::MembershipVerified {
                    channel_id,
                    port_id,
                    counterparty_channel_id,
                    counterparty_version,
                },
                IbcResponse::VerifyMembership { valid },
            ) => {
                if !valid {
                    return Err(IbcError::MembershipVerificationFailure.into());
                }

                Either::Left((
                    ChannelOpenAck::CallbackCalled {
                        channel_id: channel_id.clone(),
                        port_id: port_id.clone(),
                        counterparty_channel_id: counterparty_channel_id.clone(),
                        counterparty_version: counterparty_version.clone(),
                    },
                    IbcMsg::OnChannelOpenAck {
                        port_id,
                        channel_id,
                        counterparty_channel_id,
                        counterparty_version,
                    },
                ))
            }
            (
                ChannelOpenAck::CallbackCalled {
                    channel_id,
                    port_id,
                    counterparty_channel_id,
                    counterparty_version,
                },
                IbcResponse::OnChannelOpenAck { err },
            ) => {
                if err {
                    return Err(IbcError::IbcAppCallbackFailed.into());
                }

                let channel_path = ChannelEndPath {
                    port_id: port_id.clone(),
                    channel_id: channel_id.clone(),
                }
                .into();

                let mut channel: Channel = host.read(&channel_path).ok_or(
                    IbcError::ChannelNotFound(port_id.clone(), channel_id.clone()),
                )?;

                channel.state = channel::state::State::Open;
                channel.version = counterparty_version;
                channel.counterparty.channel_id = counterparty_channel_id.clone();

                let counterparty_port_id = channel.counterparty.port_id.clone();
                let connection_id = channel.connection_hops[0].clone();

                host.commit(channel_path, channel)?;

                Either::Right(IbcEvent::ChannelOpenAck {
                    port_id,
                    channel_id,
                    counterparty_port_id,
                    counterparty_channel_id,
                    connection_id: connection_id.to_string(),
                })
            }
            _ => return Err(IbcError::UnexpectedAction.into()),
        };

        Ok(res)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ChannelOpenConfirm {
    Init {
        channel_id: ChannelId,
        port_id: PortId,
        proof_ack: Vec<u8>,
        proof_height: Height,
    },

    StatusFetched {
        client_id: ClientId,
        counterparty: channel::counterparty::Counterparty,
        counterparty_connection_id: String,
        channel_id: ChannelId,
        port_id: PortId,
        proof_ack: Vec<u8>,
        proof_height: Height,
        version: String,
    },

    MembershipVerified {
        channel_id: ChannelId,
        port_id: PortId,
        counterparty: channel::counterparty::Counterparty,
    },

    CallbackCalled {
        channel_id: ChannelId,
        port_id: PortId,
        counterparty: channel::counterparty::Counterparty,
    },
}

impl<T: IbcHost> Runnable<T> for ChannelOpenConfirm {
    fn process(
        self,
        host: &mut T,
        resp: IbcResponse,
    ) -> Result<Either<(Self, IbcMsg), IbcEvent>, <T as IbcHost>::Error> {
        let res = match (self, resp) {
            (
                ChannelOpenConfirm::Init {
                    channel_id,
                    port_id,
                    proof_ack,
                    proof_height,
                },
                IbcResponse::Empty,
            ) => {
                let channel: Channel = host
                    .read(
                        &ChannelEndPath {
                            port_id: port_id.clone(),
                            channel_id: channel_id.clone(),
                        }
                        .into(),
                    )
                    .ok_or(IbcError::ChannelNotFound(
                        port_id.clone(),
                        channel_id.clone(),
                    ))?;

                if channel.state != channel::state::State::Tryopen {
                    return Err(IbcError::IncorrectChannelState(
                        channel.state,
                        channel::state::State::Tryopen,
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
                    .ok_or(
                        IbcError::ConnectionNotFound(channel.connection_hops[0].to_string()).into(),
                    )?;

                if connection.state != connection::state::State::Open {
                    return Err(IbcError::IncorrectConnectionState(
                        connection.state,
                        connection::state::State::Open,
                    )
                    .into());
                }

                // TODO(aeryz): check if port_id is a valid addr here?

                Either::Left((
                    ChannelOpenConfirm::StatusFetched {
                        channel_id,
                        port_id,
                        client_id: connection.client_id.clone(),
                        counterparty_connection_id: connection.counterparty.connection_id,
                        proof_ack,
                        proof_height,
                        version: channel.version,
                        counterparty: channel.counterparty,
                    },
                    IbcMsg::Status {
                        client_id: connection.client_id,
                    },
                ))
            }
            (
                ChannelOpenConfirm::StatusFetched {
                    client_id,
                    counterparty_connection_id,
                    channel_id,
                    port_id,
                    proof_ack,
                    proof_height,
                    version,
                    counterparty,
                },
                IbcResponse::Status { status },
            ) => {
                if status != Status::Active {
                    return Err(IbcError::NotActive(client_id, status).into());
                }

                let expected_channel = Channel {
                    state: channel::state::State::Open,
                    ordering: Order::Unordered,
                    counterparty: channel::counterparty::Counterparty {
                        // TODO(aeryz): make port id a validater type
                        port_id: port_id.clone(),
                        channel_id: channel_id.clone().to_string(),
                    },
                    connection_hops: vec![counterparty_connection_id.validate().unwrap()],
                    version,
                };

                Either::Left((
                    ChannelOpenConfirm::MembershipVerified {
                        channel_id,
                        port_id,
                        counterparty: counterparty.clone(),
                    },
                    IbcMsg::VerifyMembership {
                        client_id,
                        height: proof_height,
                        delay_time_period: 0,
                        delay_block_period: 0,
                        proof: proof_ack,
                        path: MerklePath {
                            key_path: vec![
                                "ibc".to_string(),
                                format!(
                                    "channelEnds/ports/{}/channels/{}",
                                    counterparty.port_id, counterparty.channel_id,
                                ),
                            ],
                        },
                        value: expected_channel.encode_as::<Proto>(),
                    },
                ))
            }
            (
                ChannelOpenConfirm::MembershipVerified {
                    channel_id,
                    port_id,
                    counterparty,
                },
                IbcResponse::VerifyMembership { valid },
            ) => {
                if !valid {
                    return Err(IbcError::MembershipVerificationFailure.into());
                }

                Either::Left((
                    ChannelOpenConfirm::CallbackCalled {
                        channel_id: channel_id.clone(),
                        port_id: port_id.clone(),
                        counterparty,
                    },
                    IbcMsg::OnChannelOpenConfirm {
                        port_id,
                        channel_id,
                    },
                ))
            }
            (
                ChannelOpenConfirm::CallbackCalled {
                    channel_id,
                    port_id,
                    counterparty,
                },
                IbcResponse::OnChannelOpenConfirm { err },
            ) => {
                if err {
                    return Err(IbcError::IbcAppCallbackFailed.into());
                }

                let channel_path = ChannelEndPath {
                    port_id: port_id.clone(),
                    channel_id: channel_id.clone(),
                }
                .into();

                let mut channel: Channel = host.read(&channel_path).ok_or(
                    IbcError::ChannelNotFound(port_id.clone(), channel_id.clone()),
                )?;

                channel.state = channel::state::State::Open;

                let connection_id = channel.connection_hops[0].clone();

                host.commit(channel_path, channel)?;

                Either::Right(IbcEvent::ChannelOpenConfirm {
                    port_id,
                    channel_id,
                    counterparty_port_id: counterparty.port_id,
                    counterparty_channel_id: counterparty.channel_id,
                    connection_id: connection_id.to_string(),
                })
            }
            _ => return Err(IbcError::UnexpectedAction.into()),
        };

        Ok(res)
    }
}
