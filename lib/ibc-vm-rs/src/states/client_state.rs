use serde::{Deserialize, Serialize};
use unionlabs::{
    encoding::{EncodeAs, Proto},
    ibc::core::{
        client::height::Height,
        commitment::{merkle_path::MerklePath, merkle_prefix::MerklePrefix},
        connection::{self, version::Version},
    },
    id::ClientId,
};

use crate::{
    Either, IbcEvent, IbcHost, IbcMsg, IbcResponse, Runnable, Status, DEFAULT_IBC_VERSION,
};

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
    ) -> Result<Either<(Self, IbcMsg), IbcEvent>, ()> {
        let res = match self {
            UpdateClient::Init {
                client_id,
                client_msg,
            } => Either::Left((
                Self::StatusFetched {
                    client_id: client_id.clone(),
                    client_msg,
                },
                IbcMsg::Status { client_id },
            )),
            UpdateClient::StatusFetched {
                client_id,
                client_msg,
            } => {
                let IbcResponse::Status {
                    status: Status::Active,
                } = resp
                else {
                    return Err(());
                };
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
            UpdateClient::ClientMessageVerified {
                client_id,
                client_msg,
            } => {
                let IbcResponse::VerifyClientMessage { valid: true } = resp else {
                    return Err(());
                };
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
            UpdateClient::MisbehaviourChecked {
                client_id,
                client_msg,
            } => {
                let IbcResponse::CheckForMisbehaviour { misbehaviour_found } = resp else {
                    return Err(());
                };

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
            UpdateClient::UpdatedStateOnMisbehaviour { client_id } => {
                if resp != IbcResponse::UpdateStateOnMisbehaviour {
                    return Err(());
                }
                Either::Right(IbcEvent::ClientMisbehaviour { client_id })
            }
            UpdateClient::UpdatedState { client_id } => {
                let IbcResponse::UpdateState {
                    consensus_states,
                    client_state,
                } = resp
                else {
                    return Err(());
                };

                host.commit_raw(format!("clients/{client_id}/clientState"), client_state);

                let consensus_heights = consensus_states
                    .into_iter()
                    .map(|(height, state)| {
                        host.commit_raw(
                            format!("clients/{client_id}/consensusStates/{height}"),
                            state,
                        );
                        height
                    })
                    .collect();

                Either::Right(IbcEvent::UpdateClient {
                    client_id,
                    consensus_heights,
                })
            }
        };
        Ok(res)
    }
}
