pub mod channel_handshake;
pub mod client_state;
pub mod connection_handshake;
pub mod packet;

use serde::{Deserialize, Serialize};
use unionlabs::{
    ics24::{ClientConsensusStatePath, ClientStatePath},
    id::ClientId,
};

use crate::{Either, IbcError, IbcEvent, IbcHost, IbcMsg, IbcResponse, Runnable, Status};

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

    FetchLcData {
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
        resp: &[IbcResponse],
    ) -> Result<Either<(Self, Vec<IbcMsg>), IbcEvent>, <T as IbcHost>::Error> {
        let res = match (self, resp) {
            (
                CreateClient::Init {
                    client_type,
                    client_state,
                    consensus_state,
                },
                &[IbcResponse::Empty],
            ) => {
                let client_id = host.next_client_identifier(&client_type)?;
                Either::Left((
                    CreateClient::Initialize {
                        client_type: client_type.clone(),
                        client_id: client_id.clone(),
                        client_state: client_state.clone(),
                        consensus_state: consensus_state.clone(),
                    },
                    vec![IbcMsg::Initialize {
                        client_id,
                        client_state,
                        consensus_state,
                        client_type,
                    }],
                ))
            }
            (
                CreateClient::Initialize {
                    client_id,
                    client_type,
                    client_state,
                    consensus_state,
                },
                &[IbcResponse::Initialize],
            ) => Either::Left((
                CreateClient::FetchLcData {
                    client_id: client_id.clone(),
                    client_type: client_type.clone(),
                    client_state,
                    consensus_state,
                },
                vec![
                    IbcMsg::Status {
                        client_id: client_id.clone(),
                    },
                    IbcMsg::LatestHeight { client_id },
                ],
            )),
            (
                CreateClient::FetchLcData {
                    client_id,
                    client_type,
                    client_state,
                    consensus_state,
                },
                &[IbcResponse::Status { status }, IbcResponse::LatestHeight { height }],
            ) => {
                if status != Status::Active {
                    return Err(IbcError::NotActive(client_id, status).into());
                }
                let client_id = client_id.clone();
                host.commit_raw(
                    ClientStatePath {
                        client_id: client_id.clone(),
                    }
                    .into(),
                    client_state.clone(),
                )?;
                host.commit_raw(
                    ClientConsensusStatePath {
                        client_id: client_id.clone(),
                        height,
                    }
                    .into(),
                    consensus_state.clone(),
                )?;
                Either::Right(IbcEvent::ClientCreated {
                    client_id,
                    client_type,
                    initial_height: height.revision_height,
                })
            }
            _ => return Err(IbcError::UnexpectedAction.into()),
        };

        Ok(res)
    }
}
