use serde::{Deserialize, Serialize};
use unionlabs::{
    ibc::core::client::height::Height,
    ics24,
    id::{ClientId, ConnectionId},
};

use crate::{
    types::connection::{ConnectionEnd, ConnectionState},
    Either, IbcAction, IbcError, IbcEvent, IbcHost, IbcQuery, IbcResponse, IbcVmResponse, Runnable,
    Status,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
pub enum ConnectionOpenInit {
    Init {
        client_id: u32,
        counterparty_client_id: u32,
        counterparty_connection_id: ConnectionId,
    },

    CheckStatus {
        client_id: u32,
        counterparty_client_id: u32,
        counterparty_connection_id: ConnectionId,
    },
}

impl<T: IbcHost> Runnable<T> for ConnectionOpenInit {
    fn process(
        self,
        host: &mut T,
        resp: &[IbcResponse],
    ) -> Result<Either<(Self, IbcAction), (Vec<IbcEvent>, IbcVmResponse)>, <T as IbcHost>::Error>
    {
        let res = match (self, resp) {
            (
                ConnectionOpenInit::Init {
                    client_id,
                    counterparty_client_id,
                    counterparty_connection_id,
                },
                &[IbcResponse::Empty],
            ) => Either::Left((
                ConnectionOpenInit::CheckStatus {
                    client_id,
                    counterparty_client_id,
                    counterparty_connection_id,
                },
                (client_id, vec![IbcQuery::Status]).into(),
            )),
            (
                ConnectionOpenInit::CheckStatus {
                    client_id,
                    counterparty_client_id,
                    counterparty_connection_id,
                },
                &[IbcResponse::Status { status }],
            ) => {
                if status != Status::Active {
                    return Err(IbcError::NotActive(client_id, status).into());
                }

                let connection_id = host.next_connection_identifier()?;

                // TODO(aeryz): maybe add `client_exists` here?
                let _ = host
                    .client_state(&client_id)
                    .ok_or(IbcError::ClientStateNotFound(client_id))?;

                let end = ConnectionEnd {
                    client_id,
                    state: ConnectionState::Init,
                    counterparty_client_id,
                    counterparty_connection_id,
                };

                host.commit(
                    ics24::ethabi::connection_key(connection_id.id()).as_ref(),
                    end,
                )?;

                Either::Right((
                    vec![IbcEvent::ConnectionOpenInit(
                        ibc_events::ConnectionOpenInit {
                            connection_id,
                            // TODO(aeryz): use events specific to this impl or convert clientid to rawclientid
                            client_id: ClientId::new("TODO", client_id),
                            counterparty_client_id: ClientId::new("TODO", counterparty_client_id),
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

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
pub enum ConnectionOpenTry {
    Init {
        client_id: u32,
        counterparty_client_id: u32,
        counterparty_connection_id: ConnectionId,
        proof_init: Vec<u8>,
        proof_height: Height,
    },

    ConnectionStateVerified {
        client_id: u32,
        counterparty_client_id: u32,
        counterparty_connection_id: ConnectionId,
    },
}

impl<T: IbcHost> Runnable<T> for ConnectionOpenTry {
    fn process(
        self,
        host: &mut T,
        resp: &[IbcResponse],
    ) -> Result<Either<(Self, IbcAction), (Vec<IbcEvent>, IbcVmResponse)>, <T as IbcHost>::Error>
    {
        let res = match (self, resp) {
            (
                ConnectionOpenTry::Init {
                    client_id,
                    counterparty_client_id,
                    counterparty_connection_id,
                    proof_init: connection_end_proof,
                    proof_height,
                },
                &[IbcResponse::Empty],
            ) => {
                let _expected_counterparty = ConnectionEnd {
                    client_id: counterparty_client_id,
                    state: ConnectionState::Init,
                    counterparty_client_id: client_id,
                    counterparty_connection_id: ConnectionId::new(0),
                };

                Either::Left((
                    ConnectionOpenTry::ConnectionStateVerified {
                        client_id,
                        counterparty_client_id,
                        counterparty_connection_id,
                    },
                    (
                        client_id,
                        vec![IbcQuery::VerifyMembership {
                            height: proof_height,
                            delay_time_period: 0,
                            delay_block_period: 0,
                            proof: connection_end_proof,
                            path: ics24::ethabi::connection_key(counterparty_connection_id.id())
                                .into_bytes(),
                            // TODO(aeryz): ethabi encode
                            // value: expected_counterparty.into(),
                            value: vec![],
                        }],
                    )
                        .into(),
                ))
            }
            (
                ConnectionOpenTry::ConnectionStateVerified {
                    client_id,
                    counterparty_client_id,
                    counterparty_connection_id,
                },
                &[IbcResponse::VerifyMembership { valid }],
            ) => {
                if !valid {
                    return Err(IbcError::MembershipVerificationFailure.into());
                }
                let connection_id = host.next_connection_identifier()?;
                let end = ConnectionEnd {
                    client_id,
                    state: ConnectionState::TryOpen,
                    counterparty_client_id,
                    counterparty_connection_id,
                };
                // TODO(aeryz): we don't do `addConnectionToClient` but idk why would we do it because if we want to check if connection exists for a client,
                // we can just read the connection and check the client id
                host.commit(
                    ics24::ethabi::connection_key(connection_id.id()).as_ref(),
                    end,
                )?;
                Either::Right((
                    vec![IbcEvent::ConnectionOpenTry(ibc_events::ConnectionOpenTry {
                        connection_id,
                        client_id: ClientId::new("TODO", client_id),
                        counterparty_client_id: ClientId::new("TODO", counterparty_client_id),
                        counterparty_connection_id,
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
pub enum ConnectionOpenAck {
    Init {
        connection_id: ConnectionId,
        counterparty_connection_id: ConnectionId,
        connection_end_proof: Vec<u8>,
        proof_height: Height,
    },

    ConnectionStateVerified {
        client_id: u32,
        connection_id: ConnectionId,
        counterparty_connection_id: ConnectionId,
        connection: ConnectionEnd,
    },
}

impl<T: IbcHost> Runnable<T> for ConnectionOpenAck {
    fn process(
        self,
        host: &mut T,
        resp: &[IbcResponse],
    ) -> Result<Either<(Self, IbcAction), (Vec<IbcEvent>, IbcVmResponse)>, <T as IbcHost>::Error>
    {
        let res = match (self, resp) {
            (
                ConnectionOpenAck::Init {
                    connection_id,
                    counterparty_connection_id,
                    connection_end_proof,
                    proof_height,
                },
                &[IbcResponse::Empty],
            ) => {
                let connection: ConnectionEnd = host
                    .read(ics24::ethabi::connection_key(connection_id.id()).as_ref())
                    .ok_or(IbcError::ConnectionNotFound(connection_id.clone()))?;

                if connection.state != ConnectionState::Init {
                    return Err(IbcError::IncorrectConnectionState(
                        connection.state,
                        ConnectionState::Init,
                    )
                    .into());
                }

                let client_id = connection.client_id.clone();

                let _expected_counterparty = ConnectionEnd {
                    client_id: connection.counterparty_client_id,
                    state: ConnectionState::TryOpen,
                    counterparty_client_id: client_id,
                    counterparty_connection_id: connection_id,
                };

                Either::Left((
                    ConnectionOpenAck::ConnectionStateVerified {
                        client_id: connection.client_id,
                        counterparty_connection_id,
                        connection_id,
                        connection,
                    },
                    (
                        client_id,
                        vec![IbcQuery::VerifyMembership {
                            height: proof_height,
                            delay_time_period: 0,
                            delay_block_period: 0,
                            proof: connection_end_proof,
                            // TODO(aeryz): fixme
                            path: vec![],
                            // TODO(aeryz): fixme
                            value: vec![],
                        }],
                    )
                        .into(),
                ))
            }
            (
                ConnectionOpenAck::ConnectionStateVerified {
                    client_id,
                    mut connection,
                    connection_id,
                    counterparty_connection_id,
                },
                &[IbcResponse::VerifyMembership { valid }],
            ) => {
                if !valid {
                    return Err(IbcError::MembershipVerificationFailure.into());
                }
                connection.state = ConnectionState::Open;
                connection.counterparty_connection_id = counterparty_connection_id;

                let counterparty_client_id =
                    ClientId::new("TODO", connection.counterparty_client_id);

                host.commit(
                    ics24::ethabi::connection_key(connection_id.id()).as_ref(),
                    connection,
                )?;

                Either::Right((
                    vec![IbcEvent::ConnectionOpenAck(ibc_events::ConnectionOpenAck {
                        connection_id,
                        client_id: ClientId::new("TODO", client_id),
                        counterparty_client_id,
                        counterparty_connection_id,
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
pub enum ConnectionOpenConfirm {
    Init {
        connection_id: ConnectionId,
        connection_end_proof: Vec<u8>,
        proof_height: Height,
    },

    ConnectionStateVerified {
        client_id: u32,
        connection_id: ConnectionId,
        connection: ConnectionEnd,
    },
}

impl<T: IbcHost> Runnable<T> for ConnectionOpenConfirm {
    fn process(
        self,
        host: &mut T,
        resp: &[IbcResponse],
    ) -> Result<Either<(Self, IbcAction), (Vec<IbcEvent>, IbcVmResponse)>, <T as IbcHost>::Error>
    {
        let res = match (self, resp) {
            (
                ConnectionOpenConfirm::Init {
                    connection_id,
                    connection_end_proof,
                    proof_height,
                },
                &[IbcResponse::Empty],
            ) => {
                let connection: ConnectionEnd = host
                    .read(&ics24::ethabi::connection_key(connection_id.id()).as_ref())
                    .ok_or(IbcError::ConnectionNotFound(connection_id.clone()))?;

                if connection.state != ConnectionState::TryOpen {
                    return Err(IbcError::IncorrectConnectionState(
                        connection.state,
                        ConnectionState::TryOpen,
                    )
                    .into());
                }

                let client_id = connection.client_id.clone();

                let _expected_counterparty = ConnectionEnd {
                    client_id: connection.counterparty_client_id,
                    state: ConnectionState::Open,
                    counterparty_client_id: client_id,
                    counterparty_connection_id: connection_id,
                };

                Either::Left((
                    ConnectionOpenConfirm::ConnectionStateVerified {
                        client_id: connection.client_id,
                        connection_id,
                        connection,
                    },
                    (
                        client_id,
                        vec![IbcQuery::VerifyMembership {
                            height: proof_height,
                            delay_time_period: 0,
                            delay_block_period: 0,
                            proof: connection_end_proof,
                            // TODO(aeryz): fixme
                            path: vec![],
                            // FIXME(aeryz):
                            value: vec![],
                        }],
                    )
                        .into(),
                ))
            }
            (
                ConnectionOpenConfirm::ConnectionStateVerified {
                    client_id,
                    connection_id,
                    mut connection,
                },
                &[IbcResponse::VerifyMembership { valid }],
            ) => {
                if !valid {
                    return Err(IbcError::MembershipVerificationFailure.into());
                }

                let counterparty_client_id = connection.counterparty_client_id;
                let counterparty_connection_id = connection.counterparty_connection_id;

                connection.state = ConnectionState::Open;
                host.commit(
                    ics24::ethabi::connection_key(connection_id.id()).as_ref(),
                    connection,
                )?;

                Either::Right((
                    vec![IbcEvent::ConnectionOpenConfirm(
                        ibc_events::ConnectionOpenConfirm {
                            connection_id,
                            client_id: ClientId::new("TODO", client_id),
                            counterparty_client_id: ClientId::new("TODO", counterparty_client_id),
                            counterparty_connection_id,
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

pub fn ensure_connection_state<T: IbcHost>(
    ibc_host: &T,
    connection_id: ConnectionId,
) -> Result<ConnectionEnd, IbcError> {
    let connection: ConnectionEnd = ibc_host
        .read(ics24::ethabi::connection_key(connection_id.id()).as_ref())
        .ok_or(IbcError::ConnectionNotFound(connection_id))?;

    if connection.state != ConnectionState::Open {
        return Err(
            IbcError::IncorrectConnectionState(connection.state, ConnectionState::Open).into(),
        );
    }

    Ok(connection)
}
