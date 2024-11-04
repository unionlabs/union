pub mod channel_handshake;
pub mod client_state;
pub mod connection_handshake;
pub mod packet;

use serde::{Deserialize, Serialize};
use unionlabs::{
    ics24::{self, ClientConsensusStatePath, ClientStatePath},
    id::ClientId,
};

use crate::{
    Either, IbcAction, IbcError, IbcEvent, IbcHost, IbcMsg, IbcQuery, IbcResponse, IbcVmResponse,
    Runnable, Status,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
pub enum CreateClient {
    Init {
        client_type: String,
        client_state: Vec<u8>,
        consensus_state: Vec<u8>,
    },

    Initialize {
        client_id: u32,
        client_type: String,
        client_state: Vec<u8>,
        consensus_state: Vec<u8>,
    },

    FetchLcData {
        client_id: u32,
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
    ) -> Result<Either<(Self, IbcAction), (Vec<IbcEvent>, IbcVmResponse)>, <T as IbcHost>::Error>
    {
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
                        client_id,
                        client_state: client_state.clone(),
                        consensus_state: consensus_state.clone(),
                    },
                    IbcMsg::Initialize {
                        client_id,
                        client_state,
                        consensus_state,
                        client_type,
                    }
                    .into(),
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
                    client_id,
                    client_type: client_type.clone(),
                    client_state,
                    consensus_state,
                },
                (client_id, vec![IbcQuery::Status, IbcQuery::LatestHeight]).into(),
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
                    ics24::ethabi::client_state_key(client_id).as_ref(),
                    client_state.clone(),
                )?;
                host.commit_raw(
                    ics24::ethabi::consensus_state_key(client_id, height.height()).as_ref(),
                    consensus_state.clone(),
                )?;
                Either::Right((
                    vec![IbcEvent::CreateClient(ibc_events::CreateClient {
                        client_id: ClientId::new("TODO", client_id),
                        client_type,
                        consensus_height: height,
                    })],
                    IbcVmResponse::Empty,
                ))
            }
            _ => return Err(IbcError::UnexpectedAction.into()),
        };

        Ok(res)
    }
}
