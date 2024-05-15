pub mod channel_handshake;
pub mod client_state;
pub mod connection_handshake;
pub mod packet;

use serde::{Deserialize, Serialize};
use unionlabs::id::ClientId;

use crate::{Either, IbcEvent, IbcHost, IbcMsg, IbcResponse, Runnable, Status};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum CreateClient {
    Init {
        client_type: String,
        client_state: Vec<u8>,
        consensus_state: Vec<u8>,
    },

    Initialize {
        client_id: ClientId,
        client_type: String,
        client_state: Vec<u8>,
        consensus_state: Vec<u8>,
    },

    FetchStatus {
        client_id: ClientId,
        client_type: String,
        client_state: Vec<u8>,
        consensus_state: Vec<u8>,
    },

    FetchLatestHeight {
        client_id: ClientId,
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
                    let client_id = host.next_client_identifier(&client_type)?;
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
