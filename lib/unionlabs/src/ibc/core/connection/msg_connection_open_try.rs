use macros::model;

use crate::{
    ibc::core::{
        client::height::Height,
        connection::{counterparty::Counterparty, version::Version},
    },
    id::ClientId,
};

#[model(proto(raw(protos::ibc::core::connection::v1::MsgConnectionOpenTry)))]
#[cfg_attr(feature = "valuable", derive(valuable::Valuable))]
pub struct MsgConnectionOpenTry {
    pub client_id: ClientId,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub client_state: Vec<u8>,
    pub counterparty: Counterparty,
    pub delay_period: u64,
    pub counterparty_versions: Vec<Version>,
    pub proof_height: Height,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub proof_init: Vec<u8>,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub proof_client: Vec<u8>,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub proof_consensus: Vec<u8>,
    /// The height the counterparty trusts of the chain this is being sent to.
    ///
    /// Given a connection handshake between A<->B, if the open try is being sent to B, then this is the trusted height of the B client on A. This is used in self client/consensus state verification, where chain B will construct the expected client/consensus states of itself and verify that it's client on A has stored them correctly.
    ///
    /// This is deprecated in IBC v9.
    pub consensus_height: Height,
}
