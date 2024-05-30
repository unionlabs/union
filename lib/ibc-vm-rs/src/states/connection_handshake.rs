use serde::{Deserialize, Serialize};
use unionlabs::{
    encoding::{EncodeAs, Proto},
    events,
    ibc::core::{
        client::height::Height,
        commitment::merkle_path::MerklePath,
        connection::{self, version::Version},
    },
    ics24::ConnectionPath,
    id::ClientId,
    validated::ValidateT,
};

use crate::{
    Either, IbcAction, IbcError, IbcEvent, IbcHost, IbcQuery, IbcResponse, IbcVmResponse, Runnable,
    Status, DEFAULT_IBC_VERSION, DEFAULT_MERKLE_PREFIX,
};

pub type Counterparty = connection::counterparty::Counterparty<ClientId, String>;
pub type ConnectionEnd = connection::connection_end::ConnectionEnd<ClientId, ClientId, String>;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
pub enum ConnectionOpenInit {
    Init {
        client_id: ClientId,
        counterparty: Counterparty,
        version: Version,
        delay_period: u64,
    },

    CheckStatus {
        client_id: ClientId,
        counterparty: Counterparty,
        versions: Vec<Version>,
        delay_period: u64,
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
                    counterparty,
                    version,
                    delay_period,
                },
                &[IbcResponse::Empty],
            ) => {
                // supported version
                verify_version_supported(&DEFAULT_IBC_VERSION, &version)?;
                Either::Left((
                    ConnectionOpenInit::CheckStatus {
                        client_id: client_id.clone(),
                        counterparty,
                        versions: DEFAULT_IBC_VERSION.clone(),
                        delay_period,
                    },
                    (client_id, vec![IbcQuery::Status]).into(),
                ))
            }
            (
                ConnectionOpenInit::CheckStatus {
                    client_id,
                    counterparty,
                    versions,
                    delay_period,
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
                    .ok_or(IbcError::ClientStateNotFound(client_id.clone()))?;

                // TODO(aeryz): We commit all connections here. Check if this is needed
                // k.SetClientConnectionPaths(ctx, clientID, conns)

                let counterparty_client_id = counterparty.client_id.clone();

                let end = ConnectionEnd {
                    client_id: client_id.clone(),
                    versions,
                    state: connection::state::State::Init,
                    counterparty,
                    delay_period,
                };

                host.commit(
                    ConnectionPath {
                        connection_id: connection_id.clone(),
                    }
                    .into(),
                    end,
                )?;

                Either::Right((
                    vec![IbcEvent::ConnectionOpenInit(events::ConnectionOpenInit {
                        connection_id,
                        client_id,
                        counterparty_client_id,
                    })],
                    IbcVmResponse::Empty,
                ))
            }
            _ => return Err(IbcError::UnexpectedAction.into()),
        };

        Ok(res)
    }
}

fn verify_version_supported(
    supported_versions: &[Version],
    proposed_version: &Version,
) -> Result<(), IbcError> {
    let Some(supported_version) = find_supported_version(proposed_version, supported_versions)
    else {
        return Err(IbcError::NoSupportedVersionFound);
    };

    verify_proposed_version(supported_version, proposed_version)
}

fn find_supported_version<'a>(
    version: &Version,
    supported_versions: &'a [Version],
) -> Option<&'a Version> {
    supported_versions
        .into_iter()
        .find(|v| v.identifier == version.identifier)
}

fn verify_proposed_version(version: &Version, proposed_version: &Version) -> Result<(), IbcError> {
    if version.identifier != proposed_version.identifier {
        return Err(IbcError::VersionIdentifiedMismatch(
            version.identifier.clone(),
            proposed_version.identifier.clone(),
        ));
    }

    // we don't allow nil feature
    if proposed_version.features.is_empty() {
        return Err(IbcError::EmptyVersionFeatures);
    }

    for feat in &proposed_version.features {
        if !version.features.contains(feat) {
            return Err(IbcError::UnsupportedFeatureInVersion(*feat));
        }
    }

    Ok(())
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
pub enum ConnectionOpenTry {
    Init {
        client_id: ClientId,
        counterparty: Counterparty,
        counterparty_versions: Vec<Version>,
        connection_end_proof: Vec<u8>,
        proof_height: Height,
        delay_period: u64,
    },

    ConnectionStateVerified {
        client_id: ClientId,
        counterparty: Counterparty,
        delay_period: u64,
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
                    counterparty,
                    counterparty_versions,
                    connection_end_proof,
                    proof_height,
                    delay_period,
                },
                &[IbcResponse::Empty],
            ) => {
                let expected_counterparty = ConnectionEnd {
                    client_id: counterparty.client_id.clone(),
                    versions: counterparty_versions.clone(),
                    state: connection::state::State::Init,
                    counterparty: Counterparty {
                        client_id: client_id.clone(),
                        connection_id: String::new(),
                        prefix: DEFAULT_MERKLE_PREFIX.clone(),
                    },
                    delay_period,
                };

                let counterparty_connection_id = counterparty.connection_id.clone();

                Either::Left((
                    ConnectionOpenTry::ConnectionStateVerified {
                        client_id: client_id.clone(),
                        counterparty,
                        delay_period,
                    },
                    (
                        client_id,
                        vec![IbcQuery::VerifyMembership {
                            height: proof_height,
                            delay_time_period: 0,
                            delay_block_period: 0,
                            proof: connection_end_proof,
                            path: MerklePath {
                                key_path: vec![
                                    "ibc".to_string(),
                                    format!("connections/{}", counterparty_connection_id),
                                ],
                            },
                            // TODO(aeryz): generic over the encoding
                            value: expected_counterparty.encode_as::<Proto>(),
                        }],
                    )
                        .into(),
                ))
            }
            (
                ConnectionOpenTry::ConnectionStateVerified {
                    client_id,
                    counterparty,
                    delay_period,
                },
                &[IbcResponse::VerifyMembership { valid }],
            ) => {
                if !valid {
                    return Err(IbcError::MembershipVerificationFailure.into());
                }
                let connection_id = host.next_connection_identifier()?;
                let end = ConnectionEnd {
                    client_id: client_id.clone(),
                    // we only support the default ibc version with unordered channels
                    versions: DEFAULT_IBC_VERSION.clone(),
                    state: connection::state::State::Tryopen,
                    counterparty: counterparty.clone(),
                    delay_period,
                };
                // TODO(aeryz): we don't do `addConnectionToClient` but idk why would we do it because if we want to check if connection exists for a client,
                // we can just read the connection and check the client id
                host.commit(
                    ConnectionPath {
                        connection_id: connection_id.clone(),
                    }
                    .into(),
                    end,
                )?;
                Either::Right((
                    vec![IbcEvent::ConnectionOpenTry(events::ConnectionOpenTry {
                        connection_id,
                        client_id,
                        counterparty_client_id: counterparty.client_id,
                        counterparty_connection_id: counterparty.connection_id.validate().unwrap(),
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
        connection_id: String,
        version: Version,
        counterparty_connection_id: String,
        connection_end_proof: Vec<u8>,
        proof_height: Height,
    },

    ConnectionStateVerified {
        client_id: ClientId,
        connection_id: String,
        counterparty_connection_id: String,
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
                    version,
                    counterparty_connection_id,
                    connection_end_proof,
                    proof_height,
                },
                &[IbcResponse::Empty],
            ) => {
                let connection: ConnectionEnd = host
                    .read(
                        &ConnectionPath {
                            connection_id: connection_id.clone().validate().unwrap(),
                        }
                        .into(),
                    )
                    .ok_or(IbcError::ConnectionNotFound(connection_id.clone()))?;

                if connection.state != connection::state::State::Init {
                    return Err(IbcError::IncorrectConnectionState(
                        connection.state,
                        connection::state::State::Init,
                    )
                    .into());
                }

                verify_version_supported(&connection.versions, &version)?;

                let client_id = connection.client_id.clone();

                let expected_counterparty = ConnectionEnd {
                    client_id: connection.counterparty.client_id.clone(),
                    versions: DEFAULT_IBC_VERSION.clone(),
                    state: connection::state::State::Tryopen,
                    counterparty: Counterparty {
                        client_id: client_id.clone(),
                        connection_id: connection_id.clone(),
                        prefix: DEFAULT_MERKLE_PREFIX.clone(),
                    },
                    delay_period: connection.delay_period,
                };

                Either::Left((
                    ConnectionOpenAck::ConnectionStateVerified {
                        client_id: connection.client_id.clone(),
                        counterparty_connection_id: counterparty_connection_id.clone(),
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
                            path: MerklePath {
                                key_path: vec![
                                    "ibc".to_string(),
                                    format!("connections/{counterparty_connection_id}"),
                                ],
                            },
                            // TODO(aeryz): generic encoding
                            value: expected_counterparty.encode_as::<Proto>(),
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
                connection.state = connection::state::State::Open;
                connection.counterparty.connection_id = counterparty_connection_id.clone();

                let counterparty_client_id = connection.counterparty.client_id.clone();

                host.commit(
                    ConnectionPath {
                        connection_id: connection_id.clone().validate().unwrap(),
                    }
                    .into(),
                    connection,
                )?;

                Either::Right((
                    vec![IbcEvent::ConnectionOpenAck(events::ConnectionOpenAck {
                        connection_id: connection_id.validate().unwrap(),
                        client_id,
                        counterparty_client_id,
                        counterparty_connection_id: counterparty_connection_id.validate().unwrap(),
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
        connection_id: String,
        connection_end_proof: Vec<u8>,
        proof_height: Height,
    },

    ConnectionStateVerified {
        client_id: ClientId,
        connection_id: String,
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
                    .read(
                        &ConnectionPath {
                            connection_id: connection_id.clone().validate().unwrap(),
                        }
                        .into(),
                    )
                    .ok_or(IbcError::ConnectionNotFound(connection_id.clone()))?;

                if connection.state != connection::state::State::Tryopen {
                    return Err(IbcError::IncorrectConnectionState(
                        connection.state,
                        connection::state::State::Tryopen,
                    )
                    .into());
                }

                let client_id = connection.client_id.clone();

                let expected_counterparty = ConnectionEnd {
                    client_id: connection.counterparty.client_id.clone(),
                    versions: DEFAULT_IBC_VERSION.clone(),
                    state: connection::state::State::Open,
                    counterparty: Counterparty {
                        client_id: client_id.clone(),
                        connection_id: connection_id.clone(),
                        prefix: DEFAULT_MERKLE_PREFIX.clone(),
                    },
                    delay_period: connection.delay_period,
                };

                let counterparty_connection_id = connection.counterparty.connection_id.clone();

                Either::Left((
                    ConnectionOpenConfirm::ConnectionStateVerified {
                        client_id: connection.client_id.clone(),
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
                            path: MerklePath {
                                key_path: vec![
                                    "ibc".to_string(),
                                    format!("connections/{counterparty_connection_id}"),
                                ],
                            },
                            // TODO(aeryz): generic encoding
                            value: expected_counterparty.encode_as::<Proto>(),
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

                let counterparty_client_id = connection.counterparty.client_id.clone();
                let counterparty_connection_id = connection.counterparty.connection_id.clone();

                connection.state = connection::state::State::Open;
                host.commit(
                    ConnectionPath {
                        connection_id: connection_id.clone().validate().unwrap(),
                    }
                    .into(),
                    connection,
                )?;

                Either::Right((
                    vec![IbcEvent::ConnectionOpenConfirm(
                        events::ConnectionOpenConfirm {
                            connection_id: connection_id.validate().unwrap(),
                            client_id,
                            counterparty_client_id,
                            counterparty_connection_id: counterparty_connection_id
                                .validate()
                                .unwrap(),
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
