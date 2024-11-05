use serde::{Deserialize, Serialize};
use unionlabs::{
    encoding::{EncodeAs, Proto},
    ibc::core::{client::height::Height, commitment::merkle_path::MerklePath},
    ics24::{
        self, ChannelEndPath, ConnectionPath, NextSequenceAckPath, NextSequenceRecvPath,
        NextSequenceSendPath,
    },
    id::{ChannelId, ClientId, ConnectionId, PortId},
};

use super::connection_handshake::ensure_connection_state;
use crate::{
    types::{
        channel::{Channel, ChannelOrder, ChannelState},
        connection::ConnectionEnd,
    },
    Either, IbcAction, IbcError, IbcEvent, IbcHost, IbcMsg, IbcQuery, IbcResponse, IbcVmResponse,
    Runnable, Status,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
pub enum ChannelOpenInit {
    Init {
        port_id: PortId,
        counterparty_port_id: PortId,
        connection_id: ConnectionId,
        ordering: ChannelOrder,
        version: String,
    },

    StatusFetched {
        client_id: u32,
        port_id: PortId,
        counterparty_port_id: PortId,
        connection_id: ConnectionId,
        ordering: ChannelOrder,
        version: String,
    },

    CallbackCalled {
        channel_id: ChannelId,
        connection_id: ConnectionId,
        port_id: PortId,
        counterparty_port_id: PortId,
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
                    port_id,
                    counterparty_port_id,
                    connection_id,
                    ordering,
                    version,
                },
                &[IbcResponse::Empty],
            ) => {
                let connection = ensure_connection_state(host, connection_id)?;

                Either::Left((
                    ChannelOpenInit::StatusFetched {
                        client_id: connection.client_id,
                        port_id,
                        counterparty_port_id,
                        connection_id,
                        ordering,
                        version,
                    },
                    (connection.client_id, vec![IbcQuery::Status]).into(),
                ))
            }
            (
                ChannelOpenInit::StatusFetched {
                    client_id,
                    port_id,
                    version,
                    counterparty_port_id,
                    connection_id,
                    ordering,
                },
                &[IbcResponse::Status { status }],
            ) => {
                if *status != Status::Active {
                    return Err(IbcError::NotActive(client_id, *status).into());
                }

                let channel_id = host.next_channel_identifier()?;

                Either::Left((
                    ChannelOpenInit::CallbackCalled {
                        channel_id,
                        connection_id,
                        port_id: port_id.clone(),
                        counterparty_port_id: counterparty_port_id.clone(),
                        version: version.clone(),
                    },
                    IbcMsg::OnChannelOpenInit {
                        order: ordering,
                        channel_id,
                        connection_id,
                        version,
                    }
                    .into(),
                ))
            }
            (
                ChannelOpenInit::CallbackCalled {
                    channel_id,
                    connection_id,
                    port_id,
                    counterparty_port_id,
                    version,
                },
                &[IbcResponse::OnChannelOpenInit { err }],
            ) => {
                if let Some(err) = err {
                    return Err(IbcError::IbcAppCallbackFailed(err.clone()).into());
                }

                let one = 1_u64.to_be_bytes().to_vec();

                let channel = Channel {
                    state: ChannelState::Init,
                    ordering: ChannelOrder::Unordered,
                    version: version.clone(),
                    connection_id,
                    counterparty_channel_id: ChannelId::new(0),
                    counterparty_port_id: counterparty_port_id.clone(),
                };

                host.commit(
                    ics24::ethabi::channel_key(channel_id.id()).as_ref(),
                    channel,
                )?;

                host.commit_raw(
                    ics24::ethabi::next_seq_send_key(channel_id.id()).as_ref(),
                    one.clone(),
                )?;
                host.commit_raw(
                    ics24::ethabi::next_seq_recv_key(channel_id.id()).as_ref(),
                    one.clone(),
                )?;
                host.commit_raw(
                    ics24::ethabi::next_seq_ack_key(channel_id.id()).as_ref(),
                    one,
                )?;

                Either::Right((
                    vec![IbcEvent::ChannelOpenInit(ibc_events::ChannelOpenInit {
                        port_id,
                        channel_id,
                        counterparty_port_id,
                        connection_id,
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
        port_id: PortId,
        channel: Channel,
        counterparty_version: String,
        proof_init: Vec<u8>,
        proof_height: Height,
    },

    LcQueriesMade {
        port_id: PortId,
        channel: Channel,
        client_id: u32,
        counterparty_version: String,
    },

    CallbackCalled {
        port_id: PortId,
        channel_id: ChannelId,
        channel: Channel,
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
                    port_id,
                    channel,
                    counterparty_version,
                    proof_init,
                    proof_height,
                },
                &[IbcResponse::Empty],
            ) => {
                let connection = ensure_connection_state(host, channel.connection_id)?;

                let _expected_channel = Channel {
                    state: ChannelState::Init,
                    connection_id: connection.counterparty_connection_id,
                    ordering: channel.ordering,
                    version: counterparty_version.clone(),
                    counterparty_channel_id: ChannelId::new(0),
                    counterparty_port_id: port_id.clone(),
                };

                // TODO(aeryz): check if port_id is a valid addr here?

                Either::Left((
                    ChannelOpenTry::LcQueriesMade {
                        port_id,
                        channel,
                        client_id: connection.client_id,
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
                                // FIXME(aeryz):
                                path: vec![],
                                // FIXME(aeryz):
                                value: vec![],
                            },
                        ],
                    )
                        .into(),
                ))
            }
            (
                ChannelOpenTry::LcQueriesMade {
                    port_id,
                    channel,
                    client_id,
                    counterparty_version,
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
                let version = channel.version.clone();
                let counterparty_channel_id = channel.counterparty_channel_id.clone();
                let connection_id = channel.connection_id;
                let order = channel.ordering;

                Either::Left((
                    ChannelOpenTry::CallbackCalled {
                        port_id: port_id.clone(),
                        channel_id,
                        channel,
                    },
                    IbcMsg::OnChannelOpenTry {
                        port_id,
                        order,
                        connection_id,
                        channel_id,
                        counterparty_channel_id,
                        version,
                        counterparty_version,
                    }
                    .into(),
                ))
            }
            (
                ChannelOpenTry::CallbackCalled {
                    channel_id,
                    mut channel,
                    port_id,
                },
                &[IbcResponse::OnChannelOpenTry { err }],
            ) => {
                if let Some(err) = err {
                    return Err(IbcError::IbcAppCallbackFailed(err.clone()).into());
                }

                let one = 1_u64.to_be_bytes().to_vec();

                channel.state = ChannelState::TryOpen;

                let counterparty_port_id = channel.counterparty_port_id.clone();
                let version = channel.version.clone();
                let connection_id = channel.connection_id;
                let counterparty_channel_id = channel.counterparty_channel_id;

                host.commit(
                    ics24::ethabi::channel_key(channel_id.id()).as_ref(),
                    channel,
                )?;

                host.commit_raw(
                    ics24::ethabi::next_seq_send_key(channel_id.id()).as_ref(),
                    one.clone(),
                )?;
                host.commit_raw(
                    ics24::ethabi::next_seq_recv_key(channel_id.id()).as_ref(),
                    one.clone(),
                )?;
                host.commit_raw(
                    ics24::ethabi::next_seq_ack_key(channel_id.id()).as_ref(),
                    one,
                )?;

                Either::Right((
                    vec![IbcEvent::ChannelOpenTry(ibc_events::ChannelOpenTry {
                        port_id,
                        channel_id,
                        counterparty_port_id,
                        counterparty_channel_id,
                        connection_id,
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
        counterparty_version: String,
        counterparty_channel_id: String,
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
        // let res = match (self, &resp) {
        //     (
        //         ChannelOpenAck::Init {
        //             channel_id,
        //             counterparty_channel_id,
        //             counterparty_version,
        //             proof_try,
        //             proof_height,
        //         },
        //         &[IbcResponse::Empty],
        //     ) => {
        //         let channel: Channel = host
        //             .read(ics24::ethabi::channel_key(channel_id.id()).as_ref())
        //             .ok_or(IbcError::ChannelNotFound(channel_id.clone()))?;

        //         if channel.state != ChannelState::Init {
        //             return Err(
        //                 IbcError::IncorrectChannelState(channel.state, ChannelState::Init).into(),
        //             );
        //         }

        //     let connection = ensure_connection_state(host, channel.connection_id);

        //     let expected_channel = Channel {
        //         state: ChannelState::TryOpen,
        //         ordering: ChannelOrder::Unordered,
        //         connection_id: connection.counterparty_connection_id,
        //         version: counterparty_version.clone(),
        //         counterparty_channel_id,
        //         counterparty_port_id: channel.port,
        //     };

        //     // TODO(aeryz): check if port_id is a valid addr here?

        //     Either::Left((
        //         ChannelOpenAck::LcQueriesMade {
        //             channel_id,
        //             port_id,
        //             counterparty_channel_id: counterparty_channel_id.clone(),
        //             counterparty_version,
        //             client_id: connection.client_id.clone(),
        //         },
        //         (
        //             connection.client_id,
        //             vec![
        //                 IbcQuery::Status,
        //                 IbcQuery::VerifyMembership {
        //                     height: proof_height,
        //                     delay_time_period: 0,
        //                     delay_block_period: 0,
        //                     proof: proof_try,
        //                     path: MerklePath {
        //                         key_path: vec![
        //                             "ibc".to_string(),
        //                             format!(
        //                             "channelEnds/ports/{}/channels/{counterparty_channel_id}",
        //                             channel.counterparty.port_id,
        //                         ),
        //                         ],
        //                     },
        //                     value: expected_channel.encode_as::<Proto>(),
        //                 },
        //             ],
        //         )
        //             .into(),
        //     ))
        // }
        // (
        //     ChannelOpenAck::LcQueriesMade {
        //         channel_id,
        //         port_id,
        //             counterparty_channel_id,
        //             counterparty_version,
        //             client_id,
        //         },
        //         &[IbcResponse::Status { status }, IbcResponse::VerifyMembership { valid }],
        //     ) => {
        //         if *status != Status::Active {
        //             return Err(IbcError::NotActive(client_id, *status).into());
        //         }

        //         if !valid {
        //             return Err(IbcError::MembershipVerificationFailure.into());
        //         }

        //         Either::Left((
        //             ChannelOpenAck::CallbackCalled {
        //                 channel_id: channel_id.clone(),
        //                 port_id: port_id.clone(),
        //                 counterparty_channel_id: counterparty_channel_id.clone(),
        //                 counterparty_version: counterparty_version.clone(),
        //             },
        //             IbcMsg::OnChannelOpenAck {
        //                 port_id,
        //                 channel_id,
        //                 counterparty_channel_id,
        //                 counterparty_version,
        //             }
        //             .into(),
        //         ))
        //     }
        //     (
        //         ChannelOpenAck::CallbackCalled {
        //             channel_id,
        //             port_id,
        //             counterparty_channel_id,
        //             counterparty_version,
        //         },
        //         &[IbcResponse::OnChannelOpenAck { err }],
        //     ) => {
        //         if let Some(err) = err {
        //             return Err(IbcError::IbcAppCallbackFailed(err.clone()).into());
        //         }

        //         let channel_path = ChannelEndPath {
        //             port_id: port_id.clone(),
        //             channel_id: channel_id.clone(),
        //         }
        //         .into();

        //         let mut channel: Channel = host
        //             .read(&channel_path)
        //             .ok_or(IbcError::ChannelNotFound(channel_id.clone()))?;

        //         channel.state = channel::state::State::Open;
        //         channel.version = counterparty_version;
        //         channel.counterparty.channel_id =
        //             Some(ChannelId::from_str_prefixed(&counterparty_channel_id).unwrap());

        //         let counterparty_port_id = channel.counterparty.port_id.clone();
        //         let connection_id = channel.connection_hops[0].clone();

        //         host.commit(channel_path, channel)?;

        //         Either::Right((
        //             vec![IbcEvent::ChannelOpenAck(ibc_events::ChannelOpenAck {
        //                 port_id,
        //                 channel_id,
        //                 counterparty_port_id,
        //                 counterparty_channel_id: ChannelId::from_str_prefixed(
        //                     &counterparty_channel_id,
        //                 )
        //                 .unwrap(),
        //                 connection_id,
        //             })],
        //             IbcVmResponse::Empty,
        //         ))
        //     }
        //     _ => return Err(IbcError::UnexpectedAction.into()),
        // };

        // Ok(res)
        todo!()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
pub enum ChannelOpenConfirm {
    Init {
        channel_id: ChannelId,
        proof_ack: Vec<u8>,
        proof_height: Height,
    },

    LcQueriesMade {
        client_id: ClientId,
        channel_id: ChannelId,
        port_id: PortId,
    },

    CallbackCalled {
        channel_id: ChannelId,
        port_id: PortId,
    },
}

impl<T: IbcHost> Runnable<T> for ChannelOpenConfirm {
    fn process(
        self,
        host: &mut T,
        resp: &[IbcResponse],
    ) -> Result<Either<(Self, IbcAction), (Vec<IbcEvent>, IbcVmResponse)>, <T as IbcHost>::Error>
    {
        // let res = match (self, &resp) {
        //     (
        //         ChannelOpenConfirm::Init {
        //             channel_id,
        //             port_id,
        //             proof_ack,
        //             proof_height,
        //         },
        //         &[IbcResponse::Empty],
        //     ) => {
        //         let channel: Channel = host
        //             .read(
        //                 &ChannelEndPath {
        //                     port_id: port_id.clone(),
        //                     channel_id: channel_id.clone(),
        //                 }
        //                 .into(),
        //             )
        //             .ok_or(IbcError::ChannelNotFound(channel_id.clone()))?;

        //         if channel.state != channel::state::State::Tryopen {
        //             return Err(IbcError::IncorrectChannelState(
        //                 channel.state,
        //                 channel::state::State::Tryopen,
        //             )
        //             .into());
        //         }

        //         let connection: ConnectionEnd = host
        //             .read(
        //                 &ConnectionPath {
        //                     connection_id: channel.connection_hops[0].clone(),
        //                 }
        //                 .into(),
        //             )
        //             .ok_or(IbcError::ConnectionNotFound(channel.connection_hops[0]))?;

        //         if connection.state != connection::state::State::Open {
        //             return Err(IbcError::IncorrectConnectionState(
        //                 connection.state,
        //                 connection::state::State::Open,
        //             )
        //             .into());
        //         }

        //         let expected_channel = Channel {
        //             state: channel::state::State::Open,
        //             ordering: Order::Unordered,
        //             counterparty: channel::counterparty::Counterparty {
        //                 port_id: port_id.clone(),
        //                 channel_id: Some(channel_id.clone()),
        //             },
        //             connection_hops: vec![connection.counterparty.connection_id.unwrap()],
        //             version: channel.version,
        //             upgrade_sequence: 0,
        //         };

        //         // TODO(aeryz): check if port_id is a valid addr here?

        //         Either::Left((
        //             ChannelOpenConfirm::LcQueriesMade {
        //                 channel_id,
        //                 port_id,
        //                 client_id: connection.client_id.clone(),
        //                 counterparty: channel.counterparty.clone(),
        //             },
        //             (
        //                 connection.client_id,
        //                 vec![
        //                     IbcQuery::Status,
        //                     IbcQuery::VerifyMembership {
        //                         height: proof_height,
        //                         delay_time_period: 0,
        //                         delay_block_period: 0,
        //                         proof: proof_ack,
        //                         path: MerklePath {
        //                             key_path: vec![
        //                                 "ibc".to_string(),
        //                                 format!(
        //                                     "channelEnds/ports/{}/channels/{:#}",
        //                                     channel.counterparty.port_id,
        //                                     channel.counterparty.channel_id.unwrap(),
        //                                 ),
        //                             ],
        //                         },
        //                         value: expected_channel.encode_as::<Proto>(),
        //                     },
        //                 ],
        //             )
        //                 .into(),
        //         ))
        //     }
        //     (
        //         ChannelOpenConfirm::LcQueriesMade {
        //             client_id,
        //             channel_id,
        //             port_id,
        //             counterparty,
        //         },
        //         &[IbcResponse::Status { status }, IbcResponse::VerifyMembership { valid }],
        //     ) => {
        //         if *status != Status::Active {
        //             return Err(IbcError::NotActive(client_id, *status).into());
        //         }

        //         if !valid {
        //             return Err(IbcError::MembershipVerificationFailure.into());
        //         }

        //         Either::Left((
        //             ChannelOpenConfirm::CallbackCalled {
        //                 channel_id: channel_id.clone(),
        //                 port_id: port_id.clone(),
        //                 counterparty,
        //             },
        //             IbcMsg::OnChannelOpenConfirm {
        //                 port_id,
        //                 channel_id,
        //             }
        //             .into(),
        //         ))
        //     }
        //     (
        //         ChannelOpenConfirm::CallbackCalled {
        //             channel_id,
        //             port_id,
        //             counterparty,
        //         },
        //         &[IbcResponse::OnChannelOpenConfirm { err }],
        //     ) => {
        //         if let Some(err) = err {
        //             return Err(IbcError::IbcAppCallbackFailed(err.clone()).into());
        //         }

        //         let channel_path = ChannelEndPath {
        //             port_id: port_id.clone(),
        //             channel_id: channel_id.clone(),
        //         }
        //         .into();

        //         let mut channel: Channel = host
        //             .read(&channel_path)
        //             .ok_or(IbcError::ChannelNotFound(channel_id.clone()))?;

        //         channel.state = channel::state::State::Open;

        //         let connection_id = channel.connection_hops[0].clone();

        //         host.commit(channel_path, channel)?;

        //         Either::Right((
        //             vec![IbcEvent::ChannelOpenConfirm(
        //                 ibc_events::ChannelOpenConfirm {
        //                     port_id,
        //                     channel_id,
        //                     counterparty_port_id: counterparty.port_id,
        //                     counterparty_channel_id: counterparty.channel_id.unwrap(),
        //                     connection_id,
        //                 },
        //             )],
        //             IbcVmResponse::Empty,
        //         ))
        //     }
        //     _ => return Err(IbcError::UnexpectedAction.into()),
        // };

        // Ok(res)
        todo!()
    }
}
