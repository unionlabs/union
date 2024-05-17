use serde::{Deserialize, Serialize};
use unionlabs::{
    ics24::{ClientConsensusStatePath, ClientStatePath},
    id::ClientId,
};

use crate::{Either, IbcError, IbcEvent, IbcHost, IbcMsg, IbcResponse, Runnable, Status};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum UpdateClient {
    Init {
        client_id: ClientId,
        client_msg: Vec<u8>,
    },

    StatusFetched {
        client_id: ClientId,
        client_msg: Vec<u8>,
    },

    ClientMessageVerified {
        client_id: ClientId,
        client_msg: Vec<u8>,
    },

    MisbehaviourChecked {
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
        resp: IbcResponse,
    ) -> Result<Either<(Self, IbcMsg), IbcEvent>, <T as IbcHost>::Error> {
        let res = match (self, resp) {
            (
                UpdateClient::Init {
                    client_id,
                    client_msg,
                },
                IbcResponse::Empty,
            ) => Either::Left((
                Self::StatusFetched {
                    client_id: client_id.clone(),
                    client_msg,
                },
                IbcMsg::Status { client_id },
            )),
            (
                UpdateClient::StatusFetched {
                    client_id,
                    client_msg,
                },
                IbcResponse::Status { status },
            ) => {
                if status != Status::Active {
                    return Err(IbcError::NotActive(client_id, status).into());
                }
                Either::Left((
                    Self::ClientMessageVerified {
                        client_id: client_id.clone(),
                        client_msg: client_msg.clone(),
                    },
                    IbcMsg::VerifyClientMessage {
                        client_id,
                        client_msg,
                    },
                ))
            }
            (
                UpdateClient::ClientMessageVerified {
                    client_id,
                    client_msg,
                },
                IbcResponse::VerifyClientMessage { valid },
            ) => {
                if !valid {
                    return Err(IbcError::ClientMessageVerificationFailed.into());
                }

                Either::Left((
                    Self::MisbehaviourChecked {
                        client_id: client_id.clone(),
                        client_msg: client_msg.clone(),
                    },
                    IbcMsg::CheckForMisbehaviour {
                        client_id,
                        client_msg,
                    },
                ))
            }
            (
                UpdateClient::MisbehaviourChecked {
                    client_id,
                    client_msg,
                },
                IbcResponse::CheckForMisbehaviour { misbehaviour_found },
            ) => {
                if misbehaviour_found {
                    Either::Left((
                        Self::UpdatedStateOnMisbehaviour {
                            client_id: client_id.clone(),
                        },
                        IbcMsg::UpdateStateOnMisbehaviour {
                            client_id,
                            client_msg,
                        },
                    ))
                } else {
                    Either::Left((
                        Self::UpdatedState {
                            client_id: client_id.clone(),
                        },
                        IbcMsg::UpdateState {
                            client_id,
                            client_msg,
                        },
                    ))
                }
            }
            (
                UpdateClient::UpdatedStateOnMisbehaviour { client_id },
                IbcResponse::UpdateStateOnMisbehaviour,
            ) => Either::Right(IbcEvent::ClientMisbehaviour { client_id }),
            (
                UpdateClient::UpdatedState { client_id },
                IbcResponse::UpdateState {
                    consensus_states,
                    client_state,
                },
            ) => {
                host.commit_raw(
                    ClientStatePath {
                        client_id: client_id.clone(),
                    }
                    .into(),
                    client_state,
                )?;

                let consensus_heights = consensus_states
                    .into_iter()
                    .map(|(height, state)| {
                        host.commit_raw(
                            ClientConsensusStatePath {
                                client_id: client_id.clone(),
                                height,
                            }
                            .into(),
                            state,
                        )?;
                        Ok(height)
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
