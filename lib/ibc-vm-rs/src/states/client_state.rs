use serde::{Deserialize, Serialize};
use unionlabs::{
    events,
    ibc::core::client::height::Height,
    ics24::{ClientConsensusStatePath, ClientStatePath},
    id::ClientId,
};

use crate::{
    Either, IbcAction, IbcError, IbcEvent, IbcHost, IbcMsg, IbcQuery, IbcResponse, IbcVmResponse,
    Runnable, Status,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
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
    ) -> Result<Either<(Self, IbcAction), (Vec<IbcEvent>, IbcVmResponse)>, <T as IbcHost>::Error>
    {
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
                &[IbcResponse::Status { status }, IbcResponse::VerifyClientMessage { error }, IbcResponse::CheckForMisbehaviour { misbehaviour_found }],
            ) => {
                if *status != Status::Active {
                    return Err(IbcError::NotActive(client_id, *status).into());
                }

                if let Some(error) = error {
                    return Err(IbcError::ClientMessageVerificationFailed(error.clone()).into());
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
            ) => Either::Right((
                vec![IbcEvent::ClientMisbehaviour(events::ClientMisbehaviour {
                    client_id,
                    // TODO(aeryz): why????
                    client_type: "TODO(aeryz) why in the hell do we have this here".to_string(),
                    // TODO(aeryz): this should be deprecated, can't see it in the latest ibc
                    consensus_height: Height::default(),
                })],
                IbcVmResponse::Empty,
            )),
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
                    .iter()
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

                Either::Right((
                    vec![IbcEvent::UpdateClient(events::UpdateClient {
                        client_id,
                        client_type: "TODO(aeryz): I hate this".to_string(),
                        consensus_heights,
                    })],
                    IbcVmResponse::Empty,
                ))
            }
            (_, _) => return Err(IbcError::UnexpectedAction.into()),
        };
        Ok(res)
    }
}
