use serde::{Deserialize, Serialize};
use unionlabs::{
    encoding::{EncodeAs, Proto},
    ibc::core::{
        channel::{self, channel::Channel, counterparty::Counterparty, order::Order},
        client::height::Height,
        commitment::merkle_path::MerklePath,
        connection::{self, connection_end::ConnectionEnd},
    },
    ics24::{
        ChannelEndPath, ConnectionPath, NextSequenceAckPath, NextSequenceRecvPath,
        NextSequenceSendPath,
    },
    id::{ChannelId, ClientId, ConnectionId, PortId},
};

use crate::{
    Either, IbcAction, IbcError, IbcEvent, IbcHost, IbcMsg, IbcQuery, IbcResponse, IbcVmResponse,
    Runnable, Status,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
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
        resp: &[IbcResponse],
    ) -> Result<Either<(Self, IbcAction), (Vec<IbcEvent>, IbcVmResponse)>, <T as IbcHost>::Error>
    {
        let res = match (self, &resp) {
            (
                ChannelOpenInit::Init {
                    connection_hops,
                    port_id,
                    counterparty,
                    version,
                },
                &[IbcResponse::Empty],
            ) => {
                let connection: ConnectionEnd = host
                    .read(
                        &ConnectionPath {
                            connection_id: connection_hops[0].clone(),
                        }
                        .into(),
                    )
                    .ok_or(IbcError::ConnectionNotFound(connection_hops[0]))?;

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
                    (connection.client_id, vec![IbcQuery::Status]).into(),
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
                &[IbcResponse::Status { status }],
            ) => {
                if *status != Status::Active {
                    return Err(IbcError::NotActive(client_id, *status).into());
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
                    }
                    .into(),
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
                &[IbcResponse::OnChannelOpenInit { err }],
            ) => {
                if let Some(err) = err {
                    return Err(IbcError::IbcAppCallbackFailed(err.clone()).into());
                }

                let one = 1_u64.to_be_bytes().to_vec();

                let channel = Channel {
                    state: channel::state::State::Init,
                    ordering: Order::Unordered,
                    counterparty: counterparty.clone(),
                    connection_hops: connection_hops.clone(),
                    version: version.clone(),
                    upgrade_sequence: 0,
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

                Either::Right((
                    vec![IbcEvent::ChannelOpenInit(ibc_events::ChannelOpenInit {
                        port_id,
                        channel_id,
                        counterparty_port_id: counterparty.port_id,
                        connection_id: connection_hops[0].clone(),
                        version,
                    })],
                    IbcVmResponse::Empty,
                ))
            }
            _ => return Err(IbcError::UnexpectedAction.into()),
        };

        Ok(res)
    }
}
#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
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

    LcQueriesMade {
        client_id: ClientId,
        connection_hops: Vec<ConnectionId>,
        port_id: PortId,
        counterparty: channel::counterparty::Counterparty,
        counterparty_version: String,
        version: String,
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
        resp: &[IbcResponse],
    ) -> Result<Either<(Self, IbcAction), (Vec<IbcEvent>, IbcVmResponse)>, <T as IbcHost>::Error>
    {
        let res = match (self, &resp) {
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
                &[IbcResponse::Empty],
            ) => {
                let connection: ConnectionEnd = host
                    .read(
                        &ConnectionPath {
                            connection_id: connection_hops[0].clone(),
                        }
                        .into(),
                    )
                    .ok_or(IbcError::ConnectionNotFound(connection_hops[0]))?;

                if connection.state != connection::state::State::Open {
                    return Err(IbcError::IncorrectConnectionState(
                        connection.state,
                        connection::state::State::Open,
                    )
                    .into());
                }

                let expected_channel = Channel {
                    state: channel::state::State::Init,
                    ordering: Order::Unordered,
                    counterparty: channel::counterparty::Counterparty {
                        port_id: port_id.clone(),
                        channel_id: None,
                    },
                    connection_hops: vec![connection.counterparty.connection_id.unwrap()],
                    version: counterparty_version.clone(),
                    upgrade_sequence: 0,
                };

                // TODO(aeryz): check if port_id is a valid addr here?

                Either::Left((
                    ChannelOpenTry::LcQueriesMade {
                        client_id: connection.client_id.clone(),
                        connection_hops,
                        port_id,
                        counterparty: counterparty.clone(),
                        version,
                        counterparty_version,
                    },
                    (
                        connection.client_id,
                        vec![
                            IbcQuery::Status,
                            IbcQuery::VerifyMembership {
                                height: proof_height,
                                delay_time_period: 0,
                                delay_block_period: 0,
                                proof: proof_init,
                                path: MerklePath {
                                    key_path: vec![
                                        "ibc".to_string(),
                                        format!(
                                            "channelEnds/ports/{}/channels/{:#}",
                                            counterparty.port_id,
                                            counterparty.channel_id.unwrap()
                                        ),
                                    ],
                                },
                                value: expected_channel.encode_as::<Proto>(),
                            },
                        ],
                    )
                        .into(),
                ))
            }
            (
                ChannelOpenTry::LcQueriesMade {
                    connection_hops,
                    port_id,
                    counterparty,
                    version,
                    counterparty_version,
                    client_id,
                },
                &[IbcResponse::Status { status }, IbcResponse::VerifyMembership { valid }],
            ) => {
                if *status != Status::Active {
                    return Err(IbcError::NotActive(client_id, *status).into());
                }

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
                    }
                    .into(),
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
                &[IbcResponse::OnChannelOpenTry { err }],
            ) => {
                if let Some(err) = err {
                    return Err(IbcError::IbcAppCallbackFailed(err.clone()).into());
                }

                let one = 1_u64.to_be_bytes().to_vec();

                let channel = Channel {
                    state: channel::state::State::Tryopen,
                    ordering: Order::Unordered,
                    counterparty: counterparty.clone(),
                    connection_hops: connection_hops.clone(),
                    version: version.clone(),
                    upgrade_sequence: 0,
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

                Either::Right((
                    vec![IbcEvent::ChannelOpenTry(ibc_events::ChannelOpenTry {
                        port_id,
                        channel_id,
                        counterparty_port_id: counterparty.port_id,
                        counterparty_channel_id: counterparty.channel_id.unwrap(),
                        connection_id: connection_hops[0].clone(),
                        version,
                    })],
                    IbcVmResponse::Empty,
                ))
            }
            _ => return Err(IbcError::UnexpectedAction.into()),
        };

        Ok(res)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
pub enum ChannelOpenAck {
    Init {
        channel_id: ChannelId,
        port_id: PortId,
        counterparty_channel_id: String,
        counterparty_version: String,
        proof_try: Vec<u8>,
        proof_height: Height,
    },

    LcQueriesMade {
        client_id: ClientId,
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
        resp: &[IbcResponse],
    ) -> Result<Either<(Self, IbcAction), (Vec<IbcEvent>, IbcVmResponse)>, <T as IbcHost>::Error>
    {
        let res = match (self, &resp) {
            (
                ChannelOpenAck::Init {
                    channel_id,
                    port_id,
                    counterparty_channel_id,
                    counterparty_version,
                    proof_try,
                    proof_height,
                },
                &[IbcResponse::Empty],
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
                    .ok_or(IbcError::ConnectionNotFound(channel.connection_hops[0]))?;

                if connection.state != connection::state::State::Open {
                    return Err(IbcError::IncorrectConnectionState(
                        connection.state,
                        connection::state::State::Open,
                    )
                    .into());
                }

                let expected_channel = Channel {
                    state: channel::state::State::Tryopen,
                    ordering: Order::Unordered,
                    counterparty: channel::counterparty::Counterparty {
                        port_id: port_id.clone(),
                        channel_id: Some(channel_id.clone()),
                    },
                    connection_hops: vec![connection.counterparty.connection_id.unwrap()],
                    version: counterparty_version.clone(),
                    upgrade_sequence: 0,
                };

                // TODO(aeryz): check if port_id is a valid addr here?

                Either::Left((
                    ChannelOpenAck::LcQueriesMade {
                        channel_id,
                        port_id,
                        counterparty_channel_id: counterparty_channel_id.clone(),
                        counterparty_version,
                        client_id: connection.client_id.clone(),
                    },
                    (
                        connection.client_id,
                        vec![
                            IbcQuery::Status,
                            IbcQuery::VerifyMembership {
                                height: proof_height,
                                delay_time_period: 0,
                                delay_block_period: 0,
                                proof: proof_try,
                                path: MerklePath {
                                    key_path: vec![
                                        "ibc".to_string(),
                                        format!(
                                        "channelEnds/ports/{}/channels/{counterparty_channel_id}",
                                        channel.counterparty.port_id,
                                    ),
                                    ],
                                },
                                value: expected_channel.encode_as::<Proto>(),
                            },
                        ],
                    )
                        .into(),
                ))
            }
            (
                ChannelOpenAck::LcQueriesMade {
                    channel_id,
                    port_id,
                    counterparty_channel_id,
                    counterparty_version,
                    client_id,
                },
                &[IbcResponse::Status { status }, IbcResponse::VerifyMembership { valid }],
            ) => {
                if *status != Status::Active {
                    return Err(IbcError::NotActive(client_id, *status).into());
                }

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
                    }
                    .into(),
                ))
            }
            (
                ChannelOpenAck::CallbackCalled {
                    channel_id,
                    port_id,
                    counterparty_channel_id,
                    counterparty_version,
                },
                &[IbcResponse::OnChannelOpenAck { err }],
            ) => {
                if let Some(err) = err {
                    return Err(IbcError::IbcAppCallbackFailed(err.clone()).into());
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
                channel.counterparty.channel_id =
                    Some(ChannelId::from_str_prefixed(&counterparty_channel_id).unwrap());

                let counterparty_port_id = channel.counterparty.port_id.clone();
                let connection_id = channel.connection_hops[0].clone();

                host.commit(channel_path, channel)?;

                Either::Right((
                    vec![IbcEvent::ChannelOpenAck(ibc_events::ChannelOpenAck {
                        port_id,
                        channel_id,
                        counterparty_port_id,
                        counterparty_channel_id: ChannelId::from_str_prefixed(
                            &counterparty_channel_id,
                        )
                        .unwrap(),
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

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
pub enum ChannelOpenConfirm {
    Init {
        channel_id: ChannelId,
        port_id: PortId,
        proof_ack: Vec<u8>,
        proof_height: Height,
    },

    LcQueriesMade {
        client_id: ClientId,
        counterparty: channel::counterparty::Counterparty,
        channel_id: ChannelId,
        port_id: PortId,
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
        resp: &[IbcResponse],
    ) -> Result<Either<(Self, IbcAction), (Vec<IbcEvent>, IbcVmResponse)>, <T as IbcHost>::Error>
    {
        let res = match (self, &resp) {
            (
                ChannelOpenConfirm::Init {
                    channel_id,
                    port_id,
                    proof_ack,
                    proof_height,
                },
                &[IbcResponse::Empty],
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
                    .ok_or(IbcError::ConnectionNotFound(channel.connection_hops[0]))?;

                if connection.state != connection::state::State::Open {
                    return Err(IbcError::IncorrectConnectionState(
                        connection.state,
                        connection::state::State::Open,
                    )
                    .into());
                }

                let expected_channel = Channel {
                    state: channel::state::State::Open,
                    ordering: Order::Unordered,
                    counterparty: channel::counterparty::Counterparty {
                        port_id: port_id.clone(),
                        channel_id: Some(channel_id.clone()),
                    },
                    connection_hops: vec![connection.counterparty.connection_id.unwrap()],
                    version: channel.version,
                    upgrade_sequence: 0,
                };

                // TODO(aeryz): check if port_id is a valid addr here?

                Either::Left((
                    ChannelOpenConfirm::LcQueriesMade {
                        channel_id,
                        port_id,
                        client_id: connection.client_id.clone(),
                        counterparty: channel.counterparty.clone(),
                    },
                    (
                        connection.client_id,
                        vec![
                            IbcQuery::Status,
                            IbcQuery::VerifyMembership {
                                height: proof_height,
                                delay_time_period: 0,
                                delay_block_period: 0,
                                proof: proof_ack,
                                path: MerklePath {
                                    key_path: vec![
                                        "ibc".to_string(),
                                        format!(
                                            "channelEnds/ports/{}/channels/{:#}",
                                            channel.counterparty.port_id,
                                            channel.counterparty.channel_id.unwrap(),
                                        ),
                                    ],
                                },
                                value: expected_channel.encode_as::<Proto>(),
                            },
                        ],
                    )
                        .into(),
                ))
            }
            (
                ChannelOpenConfirm::LcQueriesMade {
                    client_id,
                    channel_id,
                    port_id,
                    counterparty,
                },
                &[IbcResponse::Status { status }, IbcResponse::VerifyMembership { valid }],
            ) => {
                if *status != Status::Active {
                    return Err(IbcError::NotActive(client_id, *status).into());
                }

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
                    }
                    .into(),
                ))
            }
            (
                ChannelOpenConfirm::CallbackCalled {
                    channel_id,
                    port_id,
                    counterparty,
                },
                &[IbcResponse::OnChannelOpenConfirm { err }],
            ) => {
                if let Some(err) = err {
                    return Err(IbcError::IbcAppCallbackFailed(err.clone()).into());
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

                Either::Right((
                    vec![IbcEvent::ChannelOpenConfirm(
                        ibc_events::ChannelOpenConfirm {
                            port_id,
                            channel_id,
                            counterparty_port_id: counterparty.port_id,
                            counterparty_channel_id: counterparty.channel_id.unwrap(),
                            connection_id,
                        },
                    )],
                    IbcVmResponse::Empty,
                ))
            }
            _ => return Err(IbcError::UnexpectedAction.into()),
        };

        Ok(res)
    }
}
