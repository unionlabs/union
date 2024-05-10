use serde::{Deserialize, Serialize};

pub trait IbcHost {
    fn next_client_identifier(&mut self, client_type: &String) -> String;

    fn commit(&mut self, key: String, value: Vec<u8>);
}

#[derive(Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum IbcEvent {
    ClientCreated {
        client_id: String,
        client_type: String,
        initial_height: u64,
    },
}

pub trait Runnable<T: IbcHost>: Serialize + Sized {
    fn process(
        self,
        host: &mut T,
        resp: IbcResponse,
    ) -> Result<Either<(Self, IbcMsg), IbcEvent>, ()>;
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
}

pub enum Either<L, R> {
    Left(L),
    Right(R),
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
                    host.commit(
                        format!("clients/{client_id}/clientState"),
                        client_state.clone(),
                    );
                    host.commit(
                        format!("clients/{client_id}/consensusStates/{height}"),
                        consensus_state.clone(),
                    );
                    Either::Right(IbcEvent::ClientCreated {
                        client_id,
                        client_type,
                        initial_height: height,
                    })
                }
                _ => panic!("invalid action"),
            },
        };

        Ok(res)
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
