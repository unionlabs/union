use frame_support_procedural::PartialEqNoBound;
use ibc_events::IbcEvent;
use serde::{Deserialize, Serialize};
use states::{
    channel_handshake::{ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry},
    client_state::UpdateClient,
    connection_handshake::{
        ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit, ConnectionOpenTry,
    },
    packet::{Acknowledgement, RecvPacket, SendPacket},
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

pub mod states;

lazy_static::lazy_static! {
    pub static ref DEFAULT_IBC_VERSION: Vec<Version> = vec![Version { identifier: String::from("1"), features: vec![Order::Unordered] }];

    // TODO(aeryz): idk if this is enforced by ibc-go or by the spec. Because we don't have merkle prefix in ethereum or near.
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
    ConnectionNotFound(ConnectionId),

    #[error("connection state is {0} while {1} is expected")]
    IncorrectConnectionState(connection::state::State, connection::state::State),

    // TODO(aeryz): this should have the error
    #[error("ibc app callback failed ({0})")]
    IbcAppCallbackFailed(String),

    #[error("acknowledgement with the sequence {0} already exists")]
    AcknowledgementExists(u64),

    #[error("empty acknowledgement")]
    EmptyAcknowledgement,

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

    #[error("zero timeout is not allowed")]
    ZeroTimeout,

    #[error("committed packet ({comm}) does not match the calculated one ({exp_comm})", comm = serde_utils::to_hex(.0), exp_comm= serde_utils::to_hex(.1))]
    PacketCommitmentMismatch(Vec<u8>, Vec<u8>),

    #[error("empty packets received")]
    EmptyPacketsReceived,
}

pub trait IbcHost: Sized {
    type Error: core::fmt::Display + core::fmt::Debug + PartialEq + From<IbcError>;

    fn next_client_identifier(&mut self, client_type: &str) -> Result<ClientId, Self::Error>;

    fn next_connection_identifier(&mut self) -> Result<ConnectionId, Self::Error>;

    fn next_channel_identifier(&mut self) -> Result<ChannelId, Self::Error>;

    fn client_state(&self, client_id: &ClientId) -> Option<Vec<u8>>;

    fn read<T: Decode<Proto>>(&self, path: &Path) -> Option<T>;

    fn read_raw(&self, key: &Path) -> Option<Vec<u8>>;

    fn commit_raw(&mut self, key: Path, value: Vec<u8>) -> Result<(), Self::Error>;

    // TODO(aeryz): generic over encoding
    fn commit<T: Encode<Proto>>(&mut self, key: Path, value: T) -> Result<(), Self::Error>;

    fn delete(&mut self, key: &Path) -> Result<(), Self::Error>;

    fn current_height(&self) -> Height;

    fn current_timestamp(&self) -> u64;

    fn sha256(&self, data: Vec<u8>) -> Vec<u8>;
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
pub enum IbcVmResponse {
    SendPacket { sequence: u64 },
    Empty,
}

pub type CallbackError = Option<String>;

// TODO(aeryz): rename this
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
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
        err: CallbackError,
    },
    OnChannelOpenTry {
        err: CallbackError,
    },
    OnChannelOpenAck {
        err: CallbackError,
    },
    OnChannelOpenConfirm {
        err: CallbackError,
    },
    OnRecvPacket {
        acks: Vec<Vec<u8>>,
    },
    OnAcknowledgePacket {
        err: CallbackError,
    },
}

#[derive(enumorph::Enumorph, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
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
    SendPacket(SendPacket),
    RecvPacket(RecvPacket),
    AcknowledgePacket(Acknowledgement),
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
    ) -> Result<Either<(Self, IbcAction), (Vec<IbcEvent>, IbcVmResponse)>, <T as IbcHost>::Error>
    {
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
                ChannelOpenConfirm,
                SendPacket,
                RecvPacket,
                AcknowledgePacket
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

impl From<Vec<IbcMsg>> for IbcAction {
    fn from(value: Vec<IbcMsg>) -> Self {
        IbcAction::Write(value)
    }
}

impl From<IbcMsg> for IbcAction {
    fn from(value: IbcMsg) -> Self {
        IbcAction::Write(vec![value])
    }
}

#[derive(Deserialize)]
pub enum IbcAction {
    Query((ClientId, Vec<IbcQuery>)),
    Write(Vec<IbcMsg>),
}

#[derive(Serialize, Deserialize)]
pub enum IbcQuery {
    Status,
    LatestHeight,
    VerifyMembership {
        height: Height,
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

    OnAcknowledgePacket {
        packet: Packet,
        ack: Vec<u8>,
    },
}

pub trait Runnable<T: IbcHost>: Serialize + Sized {
    #[allow(clippy::type_complexity)]
    fn process(
        self,
        host: &mut T,
        resp: &[IbcResponse],
    ) -> Result<Either<(Self, IbcAction), (Vec<IbcEvent>, IbcVmResponse)>, <T as IbcHost>::Error>;
}

pub enum Either<L, R> {
    Left(L),
    Right(R),
}
