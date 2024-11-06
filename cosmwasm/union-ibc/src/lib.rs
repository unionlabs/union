pub mod contract;
pub mod ibc;
pub mod lightclient;
pub mod module;
pub mod msg;
pub mod query;
pub mod state;

use cosmwasm_std::StdError;
use ibc_solidity::ibc::{ChannelOrder, ChannelState, ConnectionState};
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
    #[error("Channel ordering is invalid: got {got:?}")]
    ChannelInvalidOrdering { got: ChannelOrder },
    #[error("Channel state is invalid: expected {expected:?}, got {got:?}")]
    ChannelInvalidState {
        got: ChannelState,
        expected: ChannelState,
    },
}
