use serde::{Deserialize, Serialize};
use unionlabs::{
    encoding::{Encode, EncodeAs, Proto},
    ibc::core::{
        client::height::Height,
        commitment::{merkle_path::MerklePath, merkle_prefix::MerklePrefix},
        connection::{
            self, connection_end::ConnectionEnd, counterparty::Counterparty, version::Version,
        },
    },
};

use crate::{
    Either, IbcEvent, IbcHost, IbcMsg, IbcResponse, Runnable, Status, DEFAULT_IBC_VERSION,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum CreateClient {
    Init {
        client_type: String,
        client_state: Vec<u8>,
        consensus_state: Vec<u8>,
    },

    Initialize {
        client_id: String,
        client_type: String,
        client_state: Vec<u8>,
        consensus_state: Vec<u8>,
    },

    FetchStatus {
        client_id: String,
        client_type: String,
        client_state: Vec<u8>,
        consensus_state: Vec<u8>,
    },

    FetchLatestHeight {
        client_id: String,
        client_type: String,
        client_state: Vec<u8>,
        consensus_state: Vec<u8>,
    },
}

impl<T: IbcHost> Runnable<T> for CreateClient {
    fn process(
        self,
        host: &mut T,
        resp: IbcResponse,
    ) -> Result<Either<(Self, IbcMsg), IbcEvent>, ()> {
        let res = match self {
            CreateClient::Init {
                client_type,
                client_state,
                consensus_state,
            } => match resp {
                IbcResponse::Empty => {
                    let client_id = host.next_client_identifier(&client_type);
                    Either::Left((
                        CreateClient::Initialize {
                            client_type: client_type.clone(),
                            client_id: client_id.clone(),
                            client_state: client_state.clone(),
                            consensus_state: consensus_state.clone(),
                        },
                        IbcMsg::Initialize {
                            client_id,
                            client_state,
                            consensus_state,
                            client_type,
                        },
                    ))
                }
                _ => panic!("invalid action"),
            },
            CreateClient::Initialize {
                client_id,
                client_type,
                client_state,
                consensus_state,
            } => match resp {
                IbcResponse::Initialize => Either::Left((
                    CreateClient::FetchStatus {
                        client_id: client_id.clone(),
                        client_type: client_type.clone(),
                        client_state,
                        consensus_state,
                    },
                    IbcMsg::Status { client_id },
                )),
                _ => panic!("invalid action"),
            },
            CreateClient::FetchStatus {
                client_id,
                client_type,
                client_state,
                consensus_state,
            } => match resp {
                IbcResponse::Status { status } => {
                    if status != Status::Active {
                        return Err(());
                    }
                    let client_id = client_id.clone();
                    Either::Left((
                        CreateClient::FetchLatestHeight {
                            client_id: client_id.clone(),
                            client_type: client_type.clone(),
                            client_state,
                            consensus_state,
                        },
                        IbcMsg::LatestHeight {
                            client_id: client_id.clone(),
                        },
                    ))
                }
                _ => panic!("invalid action"),
            },
            CreateClient::FetchLatestHeight {
                client_id,
                client_type,
                client_state,
                consensus_state,
            } => match resp {
                IbcResponse::LatestHeight { height } => {
                    host.commit_raw(
                        format!("clients/{client_id}/clientState"),
                        client_state.clone(),
                    );
                    host.commit_raw(
                        format!("clients/{client_id}/consensusStates/{height}"),
                        consensus_state.clone(),
                    );
                    Either::Right(IbcEvent::ClientCreated {
                        client_id,
                        client_type,
                        initial_height: height.revision_height,
                    })
                }
                _ => panic!("invalid action"),
            },
        };

        Ok(res)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ConnectionOpenInit {
    Init {
        client_id: String,
        counterparty: Counterparty<String, String>,
        version: Version,
        delay_period: u64,
    },

    CheckStatus {
        client_id: String,
        counterparty: Counterparty<String, String>,
        versions: Vec<Version>,
        delay_period: u64,
    },
}

impl<T: IbcHost> Runnable<T> for ConnectionOpenInit {
    fn process(
        self,
        host: &mut T,
        resp: IbcResponse,
    ) -> Result<Either<(Self, IbcMsg), IbcEvent>, ()> {
        let res = match self {
            ConnectionOpenInit::Init {
                client_id,
                counterparty,
                version,
                delay_period,
            } => {
                // supported version
                verify_version_supported(&DEFAULT_IBC_VERSION, &version)?;
                Either::Left((
                    ConnectionOpenInit::CheckStatus {
                        client_id: client_id.clone(),
                        counterparty,
                        versions: DEFAULT_IBC_VERSION.clone(),
                        delay_period,
                    },
                    IbcMsg::Status { client_id },
                ))
            }
            ConnectionOpenInit::CheckStatus {
                client_id,
                counterparty,
                versions,
                delay_period,
            } => {
                let IbcResponse::Status { status } = resp else {
                    return Err(());
                };

                if status != Status::Active {
                    return Err(());
                }

                let connection_id = host.next_connection_identifier();

                // TODO(aeryz): maybe add `client_exists` here?
                let _ = host.client_state(&client_id).ok_or(())?;

                // TODO(aeryz): We commit all connections here. Check if this is needed
                // k.SetClientConnectionPaths(ctx, clientID, conns)

                let counterparty_client_id = counterparty.client_id.clone();

                let end = ConnectionEnd::<String, String, String> {
                    client_id: client_id.clone(),
                    versions,
                    state: connection::state::State::Init,
                    counterparty,
                    delay_period,
                };

                host.commit(format!("connections/{connection_id}"), end);

                Either::Right(IbcEvent::ConnectionOpenInit {
                    connection_id,
                    client_id,
                    counterparty_client_id,
                })
            }
        };

        Ok(res)
    }
}

fn verify_version_supported(
    supported_versions: &[Version],
    proposed_version: &Version,
) -> Result<(), ()> {
    let Some(supported_version) = find_supported_version(proposed_version, supported_versions)
    else {
        return Err(());
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

fn verify_proposed_version(version: &Version, proposed_version: &Version) -> Result<(), ()> {
    if version.identifier != proposed_version.identifier {
        return Err(());
    }

    // we don't allow nil feature
    if proposed_version.features.is_empty() {
        return Err(());
    }

    for feat in &proposed_version.features {
        if !version.features.contains(feat) {
            return Err(());
        }
    }

    Ok(())
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ConnectionOpenTry {
    Init {
        client_id: String,
        counterparty: Counterparty<String, String>,
        counterparty_versions: Vec<Version>,
        connection_end_proof: Vec<u8>,
        proof_height: Height,
        delay_period: u64,
    },

    ConnectionStateVerified {
        client_id: String,
        counterparty: Counterparty<String, String>,
        delay_period: u64,
    },
}

impl<T: IbcHost> Runnable<T> for ConnectionOpenTry {
    fn process(
        self,
        host: &mut T,
        resp: IbcResponse,
    ) -> Result<Either<(Self, IbcMsg), IbcEvent>, ()> {
        let res = match self {
            ConnectionOpenTry::Init {
                client_id,
                counterparty,
                counterparty_versions,
                connection_end_proof,
                proof_height,
                delay_period,
            } => {
                // TODO(aeryz): do we want to do `validateSelfClient`?
                let expected_counterparty = ConnectionEnd::<String, String, String> {
                    client_id: counterparty.client_id.clone(),
                    versions: counterparty_versions.clone(),
                    state: connection::state::State::Init,
                    counterparty: Counterparty {
                        client_id: client_id.clone(),
                        connection_id: String::new(),
                        prefix: MerklePrefix {
                            // TODO(aeryz): make this a global constant or configurable per host?
                            key_prefix: b"ibc".into(),
                        },
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
                    IbcMsg::VerifyMembership {
                        client_id,
                        height: proof_height,
                        delay_time_period: 0,
                        delay_block_period: 0,
                        proof: connection_end_proof,
                        path: MerklePath {
                            key_path: vec![format!("connections/{}", counterparty_connection_id)],
                        },
                        // TODO(aeryz): generic over the encoding
                        value: expected_counterparty.encode_as::<Proto>(),
                    },
                ))
            }
            ConnectionOpenTry::ConnectionStateVerified {
                client_id,
                counterparty,
                delay_period,
            } => {
                let IbcResponse::VerifyMembership { valid: true } = resp else {
                    return Err(());
                };
                let connection_id = host.next_connection_identifier();
                let end = ConnectionEnd::<String, String, String> {
                    client_id: client_id.clone(),
                    // we only support the default ibc version with unordered channels
                    versions: DEFAULT_IBC_VERSION.clone(),
                    state: connection::state::State::Tryopen,
                    counterparty: counterparty.clone(),
                    delay_period,
                };
                // TODO(aeryz): we don't do `addConnectionToClient` but idk why would we do it because if we want to check if connection exists for a client,
                // we can just read the connection and check the client id
                host.commit(format!("connections/{connection_id}"), end);
                Either::Right(IbcEvent::ConnectionOpenTry {
                    connection_id,
                    client_id,
                    counterparty_client_id: counterparty.client_id,
                    counterparty_connection_id: counterparty.connection_id,
                })
            }
        };

        Ok(res)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ConnectionOpenAck {
    Init {
        connection_id: String,
        version: Version,
        counterparty_connection_id: String,
        connection_end_proof: Vec<u8>,
        proof_height: Height,
    },

    ConnectionStateVerified {
        client_id: String,
        connection_id: String,
        counterparty_connection_id: String,
        connection: ConnectionEnd<String, String, String>,
    },
}

impl<T: IbcHost> Runnable<T> for ConnectionOpenAck {
    fn process(
        self,
        host: &mut T,
        resp: IbcResponse,
    ) -> Result<Either<(Self, IbcMsg), IbcEvent>, ()> {
        let res = match self {
            ConnectionOpenAck::Init {
                connection_id,
                version,
                counterparty_connection_id,
                connection_end_proof,
                proof_height,
            } => {
                let connection: ConnectionEnd<String, String, String> = host
                    .read(&format!("connections/{connection_id}"))
                    .ok_or(())?;

                if connection.state != connection::state::State::Init {
                    return Err(());
                }

                verify_version_supported(&connection.versions, &version)?;

                let client_id = connection.client_id.clone();

                let expected_counterparty = ConnectionEnd {
                    client_id: connection.client_id.clone(),
                    versions: DEFAULT_IBC_VERSION.clone(),
                    state: connection::state::State::Tryopen,
                    counterparty: Counterparty {
                        client_id: client_id.clone(),
                        connection_id: connection_id.clone(),
                        prefix: MerklePrefix {
                            // TODO(aeryz): global const
                            key_prefix: b"ibc".into(),
                        },
                    },
                    delay_period: connection.delay_period,
                };

                Either::Left((
                    ConnectionOpenAck::ConnectionStateVerified {
                        client_id: connection.client_id.clone(),
                        counterparty_connection_id,
                        connection_id: connection_id.clone(),
                        connection,
                    },
                    IbcMsg::VerifyMembership {
                        client_id,
                        height: proof_height,
                        delay_time_period: 0,
                        delay_block_period: 0,
                        proof: connection_end_proof,
                        path: MerklePath {
                            key_path: vec![format!("connections/{connection_id}")],
                        },
                        // TODO(aeryz): generic encoding
                        value: expected_counterparty.encode_as::<Proto>(),
                    },
                ))
            }
            ConnectionOpenAck::ConnectionStateVerified {
                client_id,
                mut connection,
                connection_id,
                counterparty_connection_id,
            } => {
                let IbcResponse::VerifyMembership { valid: true } = resp else {
                    return Err(());
                };
                connection.state = connection::state::State::Open;
                connection.counterparty.connection_id = counterparty_connection_id.clone();

                let counterparty_client_id = connection.counterparty.client_id.clone();

                host.commit(format!("connections/{connection_id}"), connection);

                Either::Right(IbcEvent::ConnectionOpenAck {
                    connection_id,
                    client_id,
                    counterparty_client_id,
                    counterparty_connection_id,
                })
            }
        };

        Ok(res)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ConnectionOpenConfirm {
    Init {
        connection_id: String,
        connection_end_proof: Vec<u8>,
        proof_height: Height,
    },

    ConnectionStateVerified {
        client_id: String,
        connection_id: String,
        connection: ConnectionEnd<String, String, String>,
    },
}

impl<T: IbcHost> Runnable<T> for ConnectionOpenConfirm {
    fn process(
        self,
        host: &mut T,
        resp: IbcResponse,
    ) -> Result<Either<(Self, IbcMsg), IbcEvent>, ()> {
        let res = match self {
            ConnectionOpenConfirm::Init {
                connection_id,
                connection_end_proof,
                proof_height,
            } => {
                let connection: ConnectionEnd<String, String, String> = host
                    .read(&format!("connections/{connection_id}"))
                    .ok_or(())?;

                if connection.state != connection::state::State::Tryopen {
                    return Err(());
                }

                let client_id = connection.client_id.clone();

                let expected_counterparty = ConnectionEnd {
                    client_id: connection.counterparty.client_id.clone(),
                    versions: DEFAULT_IBC_VERSION.clone(),
                    state: connection::state::State::Open,
                    counterparty: Counterparty {
                        client_id: client_id.clone(),
                        connection_id: connection_id.clone(),
                        prefix: MerklePrefix {
                            // TODO(aeryz): global const
                            key_prefix: b"ibc".into(),
                        },
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
                    IbcMsg::VerifyMembership {
                        client_id,
                        height: proof_height,
                        delay_time_period: 0,
                        delay_block_period: 0,
                        proof: connection_end_proof,
                        path: MerklePath {
                            key_path: vec![format!("connections/{}", counterparty_connection_id)],
                        },
                        // TODO(aeryz): generic encoding
                        value: expected_counterparty.encode_as::<Proto>(),
                    },
                ))
            }
            ConnectionOpenConfirm::ConnectionStateVerified {
                client_id,
                connection_id,
                mut connection,
            } => {
                let IbcResponse::VerifyMembership { valid: true } = resp else {
                    return Err(());
                };

                let counterparty_client_id = connection.counterparty.client_id.clone();
                let counterparty_connection_id = connection.counterparty.connection_id.clone();

                connection.state = connection::state::State::Open;
                host.commit(format!("connections/{connection_id}"), connection);

                Either::Right(IbcEvent::ConnectionOpenConfirm {
                    connection_id,
                    client_id,
                    counterparty_client_id,
                    counterparty_connection_id,
                })
            }
        };

        Ok(res)
    }
}
