use std::num::NonZeroU64;

use crate::{cosmos::base::Coin, ibc::core::client::height::Height};

#[derive(Debug, Clone, PartialEq)]
pub struct MsgTransfer {
    /// the port on which the packet will be sent
    pub source_port: String,
    /// the channel by which the packet will be sent
    pub source_channel: String,
    /// the tokens to be transferred
    pub token: Coin,
    /// the sender address
    pub sender: String,
    /// the recipient address on the destination chain
    pub receiver: String,
    /// Timeout height relative to the current block height.
    /// The timeout is disabled when set to 0.
    pub timeout_height: Height,
    /// Timeout timestamp in absolute nanoseconds since unix epoch.
    /// The timeout is disabled when set to 0.
    pub timeout_timestamp: Option<NonZeroU64>,
    /// optional memo
    pub memo: String,
}
