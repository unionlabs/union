#![feature(error_in_core)]

use frame_support_procedural::PartialEqNoBound;
use serde::{Deserialize, Serialize};
use states::{
    channel_handshake::{ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry},
    client_state::UpdateClient,
    connection_handshake::{
        ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit, ConnectionOpenTry,
        Counterparty as ConnectionCounterparty,
    },
    CreateClient,
};
use unionlabs::{
    encoding::{Decode, Encode, Proto},
    ibc::core::{
        channel::{self, order::Order, packet::Packet},
        client::height::Height,
        commitment::{merkle_path::MerklePath, merkle_prefix::MerklePrefix},
        connection::{self, version::Version},
    },
    ics24::Path,
    id::{ChannelId, ClientId, ConnectionId, PortId},
};

pub type IbcEvent = unionlabs::events::IbcEvent<ClientId, String, ClientId>;

pub mod states;

lazy_static::lazy_static! {
    // TODO(aeryz): we don't support ordered channels
    pub static ref DEFAULT_IBC_VERSION: Vec<Version> = vec![Version { identifier: String::from("1"), features: vec![Order::Unordered.into()] }];

    pub static ref DEFAULT_MERKLE_PREFIX: MerklePrefix = MerklePrefix { key_prefix: b"ibc".into() };
}

#[derive(thiserror::Error, PartialEqNoBound, Debug)]
pub enum IbcError {
    #[error("client {0} is not active ({1})")]
    NotActive(ClientId, Status),

    // TODO(aeryz): this needs context
    #[error("unexpected action is provided to the state machine")]
    UnexpectedAction,

    // TODO(aeryz): this needs context
    #[error("client message verification failed")]
    ClientMessageVerificationFailed,

    #[error("connection ({0}) not found")]
    ConnectionNotFound(String),

    #[error("connection state is {0} while {1} is expected")]
    IncorrectConnectionState(connection::state::State, connection::state::State),

    // TODO(aeryz): this should have the error
    #[error("ibc app callback failed")]
    IbcAppCallbackFailed,

    // TODO(aeryz): this should have the error
    #[error("membership verification failed")]
    MembershipVerificationFailure,

    #[error("no supported version is found")]
    NoSupportedVersionFound,

    #[error("empty version features")]
    EmptyVersionFeatures,

    #[error("version identifier ({0}) does not match the proposed version ({1})")]
    VersionIdentifiedMismatch(String, String),

    #[error("the proposed version contains an unsupported feature ({0})")]
    UnsupportedFeatureInVersion(Order),

    #[error("the client state is not found for client {0}")]
    ClientStateNotFound(ClientId),

    #[error("channel ({1}) with port {0} is not found")]
    ChannelNotFound(PortId, ChannelId),

    #[error("channel state is {0} while {1} is expected")]
    IncorrectChannelState(channel::state::State, channel::state::State),

    #[error("source port ({0}) does not match the received packet's counterparty port ({1})")]
    SourcePortMismatch(PortId, PortId),

    #[error("destination port ({0}) does not match the received packet's counterparty port ({1})")]
    DestinationPortMismatch(PortId, PortId),

    #[error(
        "source channel ({0}) does not match the received packet's counterparty channel ({1})"
    )]
    SourceChannelMismatch(ChannelId, ChannelId),

    #[error(
        "source channel ({0}) does not match the received packet's counterparty channel ({1})"
    )]
    DestinationChannelMismatch(ChannelId, ChannelId),

    #[error("packet is already timed out")]
    TimedOutPacket,
}

pub trait IbcHost: Sized {
    type Error: core::fmt::Display + core::fmt::Debug + PartialEq + From<IbcError>;

    fn next_client_identifier(&mut self, client_type: &String) -> Result<ClientId, Self::Error>;

    fn next_connection_identifier(&mut self) -> Result<ConnectionId, Self::Error>;

    fn next_channel_identifier(&mut self) -> Result<ChannelId, Self::Error>;

    fn client_state(&self, client_id: &str) -> Option<Vec<u8>>;

    fn read<T: Decode<Proto>>(&self, path: &Path<ClientId, Height>) -> Option<T>;

    fn read_raw(&self, key: &Path<ClientId, Height>) -> Option<Vec<u8>>;

    fn commit_raw(
        &mut self,
        key: Path<ClientId, Height>,
        value: Vec<u8>,
    ) -> Result<(), Self::Error>;

    // TODO(aeryz): generic over encoding
    fn commit<T: Encode<Proto>>(
        &mut self,
        key: Path<ClientId, Height>,
        value: T,
    ) -> Result<(), Self::Error>;

    fn current_height(&self) -> Height;

    fn current_timestamp(&self) -> u64;

    fn sha256(&self, data: Vec<u8>) -> Vec<u8>;
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum Status {
    Active,
    Frozen,
    Expired,
}

impl core::fmt::Display for Status {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
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
    TimestampAtHeight {
        timestamp: u64,
    },
    VerifyMembership {
        valid: bool,
    },
    VerifyClientMessage {
        valid: bool,
    },
    CheckForMisbehaviour {
        misbehaviour_found: bool,
    },
    UpdateStateOnMisbehaviour,
    UpdateState {
        consensus_states: Vec<(Height, Vec<u8>)>,
        client_state: Vec<u8>,
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
    OnRecvPacket {
        // TODO(aeryz): what's gonna be the error type?
        err: bool,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IbcState {
    CreateClient(CreateClient),
    UpdateClient(UpdateClient),
    ConnectionOpenInit(ConnectionOpenInit),
    ConnectionOpenTry(ConnectionOpenTry),
    ConnectionOpenAck(ConnectionOpenAck),
    ConnectionOpenConfirm(ConnectionOpenConfirm),
    ChannelOpenInit(ChannelOpenInit),
    ChannelOpenTry(ChannelOpenTry),
    ChannelOpenAck(ChannelOpenAck),
    ChannelOpenConfirm(ChannelOpenConfirm),
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
        resp: &[IbcResponse],
    ) -> Result<Either<(Self, IbcAction), IbcEvent>, <T as IbcHost>::Error> {
        let res = cast_either!(
            self,
            host,
            resp,
            [
                CreateClient,
                UpdateClient,
                ConnectionOpenInit,
                ConnectionOpenTry,
                ConnectionOpenAck,
                ConnectionOpenConfirm,
                ChannelOpenInit,
                ChannelOpenTry,
                ChannelOpenAck,
                ChannelOpenConfirm
            ]
        );
        Ok(res)
    }
}

impl From<(ClientId, Vec<IbcQuery>)> for IbcAction {
    fn from(value: (ClientId, Vec<IbcQuery>)) -> Self {
        IbcAction::Query(value)
    }
}

impl From<IbcMsg> for IbcAction {
    fn from(value: IbcMsg) -> Self {
        IbcAction::Write(value)
    }
}

#[derive(Deserialize)]
pub enum IbcAction {
    Query((ClientId, Vec<IbcQuery>)),
    Write(IbcMsg),
}

#[derive(Serialize, Deserialize)]
pub enum IbcQuery {
    Status,
    LatestHeight,
    VerifyMembership {
        height: Height,
        // TODO(aeryz): delay times might not be relevant for other chains we could make it optional
        delay_time_period: u64,
        delay_block_period: u64,
        proof: Vec<u8>,
        path: MerklePath,
        value: Vec<u8>,
    },

    VerifyClientMessage(Vec<u8>),

    CheckForMisbehaviour(Vec<u8>),

    TimestampAtHeight(Height),
}

#[derive(Deserialize)]
pub enum IbcMsg {
    Initialize {
        client_id: ClientId,
        client_type: String,
        client_state: Vec<u8>,
        consensus_state: Vec<u8>,
    },
    UpdateStateOnMisbehaviour {
        client_id: ClientId,
        client_msg: Vec<u8>,
    },

    UpdateState {
        client_id: ClientId,
        client_msg: Vec<u8>,
    },

    OnChannelOpenInit {
        order: Order,
        connection_hops: Vec<ConnectionId>,
        port_id: PortId,
        channel_id: ChannelId,
        counterparty: channel::counterparty::Counterparty,
        version: String,
    },

    OnChannelOpenTry {
        order: Order,
        connection_hops: Vec<ConnectionId>,
        port_id: PortId,
        channel_id: ChannelId,
        counterparty: channel::counterparty::Counterparty,
        counterparty_version: String,
    },

    OnChannelOpenAck {
        port_id: PortId,
        channel_id: ChannelId,
        counterparty_channel_id: String,
        counterparty_version: String,
    },

    OnChannelOpenConfirm {
        port_id: PortId,
        channel_id: ChannelId,
    },

    OnRecvPacket {
        packet: Packet,
        // TODO(aeryz): relayer address
    },
}

pub trait Runnable<T: IbcHost>: Serialize + Sized {
    // TODO(aeryz): in most of the cases, we will return a single ibcmsg and it will be known at the compile time,
    // which means heap allocation can totally be emitted. We should make a struct for this.
    fn process(
        self,
        host: &mut T,
        resp: &[IbcResponse],
    ) -> Result<Either<(Self, IbcAction), IbcEvent>, <T as IbcHost>::Error>;
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
    client_id: ClientId,
    counterparty: ConnectionCounterparty,
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
    client_id: ClientId,
    counterparty: ConnectionCounterparty,
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

pub fn channel_open_init(
    connection_hops: Vec<ConnectionId>,
    port_id: PortId,
    counterparty: channel::counterparty::Counterparty,
    version: String,
) -> IbcState {
    IbcState::ChannelOpenInit(ChannelOpenInit::Init {
        connection_hops,
        port_id,
        counterparty,
        version,
    })
}

pub fn channel_open_ack(
    channel_id: ChannelId,
    port_id: PortId,
    counterparty_channel_id: String,
    counterparty_version: String,
    proof_try: Vec<u8>,
    proof_height: Height,
) -> IbcState {
    IbcState::ChannelOpenAck(ChannelOpenAck::Init {
        channel_id,
        port_id,
        counterparty_channel_id,
        counterparty_version,
        proof_try,
        proof_height,
    })
}

pub fn update_client(client_id: ClientId, client_msg: Vec<u8>) -> IbcState {
    IbcState::UpdateClient(UpdateClient::Init {
        client_id,
        client_msg,
    })
}
