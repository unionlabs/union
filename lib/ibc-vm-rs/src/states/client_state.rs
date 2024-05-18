use serde::{Deserialize, Serialize};
use unionlabs::{
    ics24::{ClientConsensusStatePath, ClientStatePath},
    id::ClientId,
};

use crate::{
    Either, IbcAction, IbcError, IbcEvent, IbcHost, IbcMsg, IbcQuery, IbcResponse, Runnable, Status,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum UpdateClient {
    Init {
        client_id: ClientId,
        client_msg: Vec<u8>,
    },

    LcQueriesMade {
        client_id: ClientId,
        client_msg: Vec<u8>,
    },

    UpdatedStateOnMisbehaviour {
        client_id: ClientId,
    },

    UpdatedState {
        client_id: ClientId,
    },
}

impl<T: IbcHost> Runnable<T> for UpdateClient {
    fn process(
        self,
        host: &mut T,
        resp: &[IbcResponse],
    ) -> Result<Either<(Self, IbcAction), IbcEvent>, <T as IbcHost>::Error> {
        let res = match (self, &resp) {
            (
                UpdateClient::Init {
                    client_id,
                    client_msg,
                },
                &[IbcResponse::Empty],
            ) => Either::Left((
                Self::LcQueriesMade {
                    client_id: client_id.clone(),
                    client_msg: client_msg.clone(),
                },
                (
                    client_id,
                    vec![
                        IbcQuery::Status,
                        IbcQuery::VerifyClientMessage(client_msg.clone()),
                        IbcQuery::CheckForMisbehaviour(client_msg),
                    ],
                )
                    .into(),
            )),
            (
                UpdateClient::LcQueriesMade {
                    client_id,
                    client_msg,
                },
                &[IbcResponse::Status { status }, IbcResponse::VerifyClientMessage { valid }, IbcResponse::CheckForMisbehaviour { misbehaviour_found }],
            ) => {
                if *status != Status::Active {
                    return Err(IbcError::NotActive(client_id, *status).into());
                }
                if !valid {
                    return Err(IbcError::ClientMessageVerificationFailed.into());
                }
                if *misbehaviour_found {
                    Either::Left((
                        Self::UpdatedStateOnMisbehaviour {
                            client_id: client_id.clone(),
                        },
                        IbcMsg::UpdateStateOnMisbehaviour {
                            client_id,
                            client_msg,
                        }
                        .into(),
                    ))
                } else {
                    Either::Left((
                        Self::UpdatedState {
                            client_id: client_id.clone(),
                        },
                        IbcMsg::UpdateState {
                            client_id,
                            client_msg,
                        }
                        .into(),
                    ))
                }
            }
            (
                UpdateClient::UpdatedStateOnMisbehaviour { client_id },
                &[IbcResponse::UpdateStateOnMisbehaviour],
            ) => Either::Right(IbcEvent::ClientMisbehaviour { client_id }),
            (
                UpdateClient::UpdatedState { client_id },
                &[IbcResponse::UpdateState {
                    consensus_states,
                    client_state,
                }],
            ) => {
                host.commit_raw(
                    ClientStatePath {
                        client_id: client_id.clone(),
                    }
                    .into(),
                    client_state.clone(),
                )?;

                let consensus_heights = consensus_states
                    .into_iter()
                    .map(|(height, state)| {
                        host.commit_raw(
                            ClientConsensusStatePath {
                                client_id: client_id.clone(),
                                height: *height,
                            }
                            .into(),
                            state.clone(),
                        )?;
                        Ok(*height)
                    })
                    .collect::<Result<Vec<_>, <T as IbcHost>::Error>>()?;

                Either::Right(IbcEvent::UpdateClient {
                    client_id,
                    consensus_heights,
                })
            }
            (_, _) => return Err(IbcError::UnexpectedAction.into()),
        };
        Ok(res)
    }
}
