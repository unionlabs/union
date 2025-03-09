#![cfg_attr(not(test), warn(clippy::unwrap_used))]

pub mod contract;
pub mod state;

#[cfg(test)]
mod tests;

use cosmwasm_std::{Addr, StdError};
use ibc_union_msg::lightclient::Status;
use ibc_union_spec::types::{ChannelState, ConnectionState};
use thiserror::Error;
use unionlabs::primitives::Bytes;
use unionlabs_cosmwasm_upgradable::UpgradeError;

#[derive(Error, Debug, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(
    derive(strum::EnumString, strum::Display),
    name(ContractErrorKind),
    strum(prefix = "IBC_UNION_ERR_", serialize_all = "SCREAMING_SNAKE_CASE")
)]
pub enum ContractError {
    #[error("{} std error: {0}", ContractErrorKind::from(self))]
    Std(#[from] StdError),
    #[error("{} migration error", ContractErrorKind::from(self))]
    Migrate(#[from] UpgradeError),
    #[error(
        "{} the client type has been registered already",
        ContractErrorKind::from(self)
    )]
    ClientTypeAlreadyExists,
    #[error("{} an arithmetic overflow occurred", ContractErrorKind::from(self))]
    ArithmeticOverflow,

    #[error(
        "{} connection state is invalid: expected {expected:?}, got {got:?}",
        ContractErrorKind::from(self)
    )]
    ConnectionInvalidState {
        got: ConnectionState,
        expected: ConnectionState,
    },
    #[error(
        "{} channel state is invalid: expected {expected:?}, got {got:?}",
        ContractErrorKind::from(self)
    )]
    ChannelInvalidState {
        got: ChannelState,
        expected: ChannelState,
    },
    #[error(
        "{} received a timed-out packet: (timeout_height ({timeout_height}) \
        <= current_height({current_height})",
        ContractErrorKind::from(self)
    )]
    ReceivedTimedOutPacketHeight {
        timeout_height: u64,
        current_height: u64,
    },
    #[error(
        "{} received a timed-out packet: (timeout timestamp ({timeout_timestamp}) \
        <= current timestamp({current_timestamp})",
        ContractErrorKind::from(self)
    )]
    ReceivedTimedOutPacketTimestamp {
        timeout_timestamp: u64,
        current_timestamp: u64,
    },
    #[error(
        "{} caller ({caller}) is not the owner ({owner}) of the channel ({channel_id})",
        ContractErrorKind::from(self)
    )]
    Unauthorized {
        channel_id: u32,
        owner: Addr,
        caller: Addr,
    },
    #[error("{} packet not received", ContractErrorKind::from(self))]
    PacketNotReceived,
    #[error("{} packet is already acknowledged", ContractErrorKind::from(self))]
    AlreadyAcknowledged,
    #[error("{} timeout must be set", ContractErrorKind::from(self))]
    TimeoutMustBeSet,
    #[error("{} timestamp timeout not yet reached", ContractErrorKind::from(self))]
    TimeoutTimestampNotReached,
    #[error("{} height timeout not yet reached", ContractErrorKind::from(self))]
    TimeoutHeightNotReached,
    #[error("{} channel ({0}) does not exist", ContractErrorKind::from(self))]
    ChannelNotExist(u32),
    #[error(
        "{} packet has been already acknowledged",
        ContractErrorKind::from(self)
    )]
    PacketAlreadyAcknowledged,
    #[error("{} packet commitment not found", ContractErrorKind::from(self))]
    PacketCommitmentNotFound,
    #[error(
        "{} packet timeout proof timestamp not found",
        ContractErrorKind::from(self)
    )]
    TimeoutProofTimestampNotFound,
    #[error("{} no packets provided", ContractErrorKind::from(self))]
    NotEnoughPackets,
    #[error("{} packet acknowledgement is empty", ContractErrorKind::from(self))]
    AcknowledgementIsEmpty,
    #[error(
        "{} packet acknowledgement doesn't match, found {found} but expected {expected}",
        ContractErrorKind::from(self)
    )]
    AcknowledgementMismatch { found: Bytes, expected: Bytes },
    #[error("{} the packet already exist", ContractErrorKind::from(self))]
    PacketCommitmentAlreadyExist,
    #[error(
        "{} caller {caller} don't have permission to migrate the client {client} with id {client_id}", ContractErrorKind::from(self)
    )]
    UnauthorizedMigration {
        client_id: u32,
        caller: Addr,
        client: Addr,
    },
    #[error(
        "{} cannot migrate the client {client_id} when there's no client state",
        ContractErrorKind::from(self)
    )]
    CannotMigrateWithNoClientState { client_id: u32 },
    #[error(
        "{} cannot migrate the client {client_id} when there's no consensus state at height {height}", ContractErrorKind::from(self)
    )]
    CannotMigrateWithNoConsensusState { client_id: u32, height: u64 },
    #[error(
        "{} cannot query light client {client_impl} with {query:?}: {error}",
        ContractErrorKind::from(self)
    )]
    CannotQueryLightClient {
        client_impl: Addr,
        query: Box<ibc_union_msg::lightclient::QueryMsg>,
        error: StdError,
    },
    #[error(
        "{} client {client_id} is not active (status {status:?})",
        ContractErrorKind::from(self)
    )]
    ClientNotActive { client_id: u32, status: Status },
}

impl ContractErrorKind {
    pub fn parse(s: &str) -> Option<Self> {
        s.strip_prefix("IBC_UNION_ERR_")?.parse().ok()
    }
}
