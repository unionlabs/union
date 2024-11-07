pub mod contract;
pub mod lightclient;
pub mod module;
pub mod msg;
pub mod query;
pub mod state;

use cosmwasm_std::StdError;
use ibc_solidity::cosmwasm::types::ibc::{ChannelState, ConnectionState};
use thiserror::Error;

#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),
    #[error("Invalid IBC version, got {version}")]
    InvalidIbcVersion { version: String },
    #[error("Only supports unordered channel")]
    OnlyOrderedChannel {},
    #[error("The client type has been registered already")]
    ClientTypeAlreadyExists,
    #[error("An arithmetic overflow occured")]
    ArithmeticOverflow,

    #[error("Connection state is invalid: expected {expected:?}, got {got:?}")]
    ConnectionInvalidState {
        got: ConnectionState,
        expected: ConnectionState,
    },
    #[error("Channel state is invalid: expected {expected:?}, got {got:?}")]
    ChannelInvalidState {
        got: ChannelState,
        expected: ChannelState,
    },
    #[error("No packets are received")]
    NoPacketsReceived,
    #[error("Received a timed-out packet: (timeout_height ({timeout_height}) >= current_height({current_height})")]
    ReceivedTimedOutPacketHeight {
        timeout_height: u64,
        current_height: u64,
    },
    #[error("Received a timed-out packet: (timeout timestamp ({timeout_timestamp}) >= current timestamp({current_timestamp})")]
    ReceivedTimedOutPacketTimestamp {
        timeout_timestamp: u64,
        current_timestamp: u64,
    },
    #[error("Caller is not the owner of the channel")]
    Unauthorized,
    #[error("Packet not received")]
    PacketNotReceived,
    #[error("Packet is already acknowledged")]
    AlreadyAcknowledged,
    #[error("Timeout must be set")]
    TimeoutMustBeSet,
    #[error("Timestamp timeout not yet reached")]
    TimeoutTimestampNotReached,
    #[error("Height timeout not yet reached")]
    TimeoutHeightNotReached,
    #[error("channel ({0}) does not exist")]
    ChannelNotExist(u32),
    #[error("Packet commitment not found")]
    PacketCommitmentNotFound,
    #[error("Packet timeout proof timestamp not found")]
    TimeoutProofTimestampNotFound,
    #[error("No packets provided")]
    NotEnoughPackets,
    #[error("Packet acknowledgement is empty")]
    AcknowledgementIsEmpty,
    #[error("Packet acknowledgement doesn't match")]
    AcknowledgementMismatch,
    #[error("The packet already exist")]
    PacketCommitmentAlreadyExist,
}
