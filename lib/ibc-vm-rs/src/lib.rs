use serde::{Deserialize, Serialize};

pub trait IbcHost {
    fn next_client_identifier(&mut self, client_type: &String) -> String;

    fn commit(&mut self, key: String, value: Vec<u8>);
}

#[derive(PartialEq)]
pub enum Status {
    Active,
    Frozen,
    Expired,
}

#[derive(PartialEq)]
pub enum IbcResponse {
    Empty,
    Initialize,
    Status { status: Status },
    LatestHeight { height: u64 },
}

#[derive(Deserialize)]
pub enum IbcMsg {
    Initialize {
        client_id: String,
        client_type: String,
        client_state: Vec<u8>,
        consensus_state: Vec<u8>,
    },
    Status {
        client_id: String,
    },
    LatestHeight {
        client_id: String,
    },
}

pub enum IbcEvent {
    ClientCreated {
        client_id: String,
        client_type: String,
        initial_height: u64,
    },
}

pub trait Runnable<T: IbcHost>: Serialize + Sized {
    fn process(self, host: &mut T, resp: IbcResponse) -> Result<(Self, Option<IbcMsg>), ()>;

    fn should_emit(&self) -> Option<IbcEvent>;
}

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

    CommitState {
        client_id: String,
        client_type: String,
        initial_height: u64,
        client_state: Vec<u8>,
        consensus_state: Vec<u8>,
    },

    EmitEvent {
        client_id: String,
        client_type: String,
        initial_height: u64,
        client_state: Vec<u8>,
        consensus_state: Vec<u8>,
    },
}

impl<T: IbcHost> Runnable<T> for CreateClient {
    fn process(self, host: &mut T, resp: IbcResponse) -> Result<(Self, Option<IbcMsg>), ()> {
        match self {
            CreateClient::Init {
                client_type,
                client_state,
                consensus_state,
            } => match resp {
                IbcResponse::Empty => {
                    let client_id = host.next_client_identifier(&client_type);
                    (
                        CreateClient::Initialize {
                            client_type: client_type.clone(),
                            client_id: client_id.clone(),
                            client_state: client_state.clone(),
                            consensus_state: consensus_state.clone(),
                        },
                        Some(IbcMsg::Initialize {
                            client_id,
                            client_state,
                            consensus_state,
                            client_type,
                        }),
                    )
                }
                _ => panic!("invalid action"),
            },
            CreateClient::Initialize {
                client_id,
                client_type,
                client_state,
                consensus_state,
            } => match resp {
                IbcResponse::Initialize => (
                    CreateClient::FetchStatus {
                        client_id: client_id.clone(),
                        client_type: client_type.clone(),
                        client_state,
                        consensus_state,
                    },
                    Some(IbcMsg::Status { client_id }),
                ),
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
                    (
                        CreateClient::FetchLatestHeight {
                            client_id: client_id.clone(),
                            client_type: client_type.clone(),
                            client_state,
                            consensus_state,
                        },
                        Some(IbcMsg::LatestHeight {
                            client_id: client_id.clone(),
                        }),
                    )
                }
                _ => panic!("invalid action"),
            },
            CreateClient::FetchLatestHeight {
                client_id,
                client_type,
                client_state,
                consensus_state,
            } => match resp {
                IbcResponse::LatestHeight { height } => (
                    CreateClient::EmitEvent {
                        client_id: client_id.clone(),
                        client_type: client_type.clone(),
                        initial_height: height,
                        client_state,
                        consensus_state,
                    },
                    None,
                ),

                _ => panic!("invalid action"),
            },
            CreateClient::CommitState {
                client_id,
                client_type,
                initial_height,
                client_state,
                consensus_state,
            } => {
                host.commit(
                    format!("clients/{client_id}/clientState"),
                    client_state.clone(),
                );
                host.commit(
                    format!("clients/{client_id}/consensusStates/{initial_height}"),
                    consensus_state.clone(),
                );
                (
                    CreateClient::EmitEvent {
                        client_id,
                        client_type,
                        initial_height,
                        client_state,
                        consensus_state,
                    },
                    None,
                )
            }
            CreateClient::EmitEvent { .. } => (self, None),
        };

        Err(())
    }

    fn should_emit(&self) -> Option<IbcEvent> {
        if let CreateClient::EmitEvent {
            client_id,
            client_type,
            initial_height,
            ..
        } = self
        {
            Some(IbcEvent::ClientCreated {
                client_id: client_id.clone(),
                client_type: client_type.clone(),
                initial_height: *initial_height,
            })
        } else {
            None
        }
    }
}

pub fn create_client(
    client_type: String,
    client_state: Vec<u8>,
    consensus_state: Vec<u8>,
) -> CreateClient {
    CreateClient::Init {
        client_type,
        client_state,
        consensus_state,
    }
}
