use serde::{Deserialize, Serialize};
use states::{
    ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit, ConnectionOpenTry, CreateClient,
};
use unionlabs::{
    encoding::{Decode, Encode, Proto},
    ibc::core::{
        channel::{self, order::Order},
        client::height::Height,
        commitment::merkle_path::MerklePath,
        connection::{self, version::Version},
    },
    id::ConnectionId,
};

pub mod states;

lazy_static::lazy_static! {
    // TODO(aeryz): we don't support ordered channels
    pub static ref DEFAULT_IBC_VERSION: Vec<Version> = vec![Version { identifier: String::from("1"), features: vec![Order::Unordered.into()] }];
}

pub trait IbcHost {
    fn next_client_identifier(&mut self, client_type: &String) -> String;

    fn next_connection_identifier(&mut self) -> String;

    fn next_channel_identifier(&mut self) -> String;

    fn client_state(&self, client_id: &str) -> Option<Vec<u8>>;

    fn read<T: Decode<Proto>>(&self, key: &str) -> Option<T>;

    fn commit_raw(&mut self, key: String, value: Vec<u8>);

    // TODO(aeryz): generic over encoding
    fn commit<T: Encode<Proto>>(&mut self, key: String, value: T);
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Status {
    Active,
    Frozen,
    Expired,
}

#[derive(PartialEq)]
pub enum IbcResponse {
    Empty,
    Initialize,
    Status {
        status: Status,
    },
    LatestHeight {
        height: Height,
    },
    VerifyMembership {
        valid: bool,
    },
    OnChannelOpenInit {
        // TODO(aeryz): what's gonna be the error type?
        err: bool,
    },
    OnChannelOpenTry {
        // TODO(aeryz): what's gonna be the error type?
        err: bool,
    },
    OnChannelOpenAck {
        // TODO(aeryz): what's gonna be the error type?
        err: bool,
    },
    OnChannelOpenConfirm {
        // TODO(aeryz): what's gonna be the error type?
        err: bool,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IbcState {
    CreateClient(states::CreateClient),
    ConnectionOpenInit(states::ConnectionOpenInit),
    ConnectionOpenTry(states::ConnectionOpenTry),
    ConnectionOpenAck(states::ConnectionOpenAck),
    ConnectionOpenConfirm(states::ConnectionOpenConfirm),
}

macro_rules! cast_either {
    ($this:ident, $host:ident, $resp:ident, [ $($arm:ident), *]) => {
        match $this {
            $(IbcState::$arm(s) => match s.process($host, $resp)? {
                Either::Left((substate, msg)) => Either::Left((IbcState::$arm(substate), msg)),
                Either::Right(right) => Either::Right(right),
            },)*
        }
    };
}

impl<T: IbcHost> Runnable<T> for IbcState {
    fn process(
        self,
        host: &mut T,
        resp: IbcResponse,
    ) -> Result<Either<(Self, IbcMsg), IbcEvent>, ()> {
        let res = cast_either!(
            self,
            host,
            resp,
            [
                CreateClient,
                ConnectionOpenInit,
                ConnectionOpenTry,
                ConnectionOpenAck,
                ConnectionOpenConfirm
            ]
        );
        Ok(res)
    }
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

    VerifyMembership {
        client_id: String,
        height: Height,
        // TODO(aeryz): delay times might not be relevant for other chains we could make it optional
        delay_time_period: u64,
        delay_block_period: u64,
        proof: Vec<u8>,
        path: MerklePath,
        value: Vec<u8>,
    },

    OnChannelOpenInit {
        order: Order,
        connection_hops: Vec<ConnectionId>,
        port_id: String,
        channel_id: String,
        counterparty: channel::counterparty::Counterparty,
        version: String,
    },

    OnChannelOpenTry {
        order: Order,
        connection_hops: Vec<ConnectionId>,
        port_id: String,
        channel_id: String,
        counterparty: channel::counterparty::Counterparty,
        counterparty_version: String,
    },

    OnChannelOpenAck {
        port_id: String,
        channel_id: String,
        counterparty_channel_id: String,
        counterparty_version: String,
    },

    OnChannelOpenConfirm {
        port_id: String,
        channel_id: String,
    },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum IbcEvent {
    ClientCreated {
        client_id: String,
        client_type: String,
        initial_height: u64,
    },

    ConnectionOpenInit {
        connection_id: String,
        client_id: String,
        counterparty_client_id: String,
    },

    ConnectionOpenTry {
        connection_id: String,
        client_id: String,
        counterparty_client_id: String,
        counterparty_connection_id: String,
    },

    ConnectionOpenAck {
        connection_id: String,
        client_id: String,
        counterparty_client_id: String,
        counterparty_connection_id: String,
    },

    ConnectionOpenConfirm {
        connection_id: String,
        client_id: String,
        counterparty_client_id: String,
        counterparty_connection_id: String,
    },

    ChannelOpenInit {
        port_id: String,
        channel_id: String,
        counterparty_port_id: String,
        connection_id: String,
        version: String,
    },

    ChannelOpenTry {
        port_id: String,
        channel_id: String,
        counterparty_port_id: String,
        counterparty_channel_id: String,
        connection_id: String,
        version: String,
    },

    ChannelOpenAck {
        port_id: String,
        channel_id: String,
        counterparty_port_id: String,
        counterparty_channel_id: String,
        connection_id: String,
    },

    ChannelOpenConfirm {
        port_id: String,
        channel_id: String,
        counterparty_port_id: String,
        counterparty_channel_id: String,
        connection_id: String,
    },
}

pub trait Runnable<T: IbcHost>: Serialize + Sized {
    fn process(
        self,
        host: &mut T,
        resp: IbcResponse,
    ) -> Result<Either<(Self, IbcMsg), IbcEvent>, ()>;
}

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

pub fn create_client(
    client_type: String,
    client_state: Vec<u8>,
    consensus_state: Vec<u8>,
) -> IbcState {
    IbcState::CreateClient(CreateClient::Init {
        client_type,
        client_state,
        consensus_state,
    })
}

pub fn connection_open_init(
    client_id: String,
    counterparty: connection::counterparty::Counterparty<String, String>,
    version: Version,
    delay_period: u64,
) -> IbcState {
    IbcState::ConnectionOpenInit(ConnectionOpenInit::Init {
        client_id,
        counterparty,
        version,
        delay_period,
    })
}

pub fn connection_open_try(
    client_id: String,
    counterparty: connection::counterparty::Counterparty<String, String>,
    counterparty_versions: Vec<Version>,
    connection_end_proof: Vec<u8>,
    proof_height: Height,
    delay_period: u64,
) -> IbcState {
    IbcState::ConnectionOpenTry(ConnectionOpenTry::Init {
        client_id,
        counterparty,
        counterparty_versions,
        connection_end_proof,
        proof_height,
        delay_period,
    })
}

pub fn connection_open_ack(
    connection_id: String,
    version: Version,
    counterparty_connection_id: String,
    connection_end_proof: Vec<u8>,
    proof_height: Height,
) -> IbcState {
    IbcState::ConnectionOpenAck(ConnectionOpenAck::Init {
        connection_id,
        version,
        counterparty_connection_id,
        connection_end_proof,
        proof_height,
    })
}

pub fn connection_open_confirm(
    connection_id: String,
    connection_end_proof: Vec<u8>,
    proof_height: Height,
) -> IbcState {
    IbcState::ConnectionOpenConfirm(ConnectionOpenConfirm::Init {
        connection_id,
        connection_end_proof,
        proof_height,
    })
}
