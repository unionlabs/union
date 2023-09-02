use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Uint512};

use crate::state::ChannelInfo;

#[cw_serde]
pub struct InitMsg {
    /// Default timeout for ics20 packets, specified in seconds
    pub default_timeout: u64,
    /// who can allow more contracts
    pub gov_contract: String,
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    /// This allows us to transfer native tokens
    Transfer(TransferMsg),
    /// Change the admin (must be called by current admin)
    UpdateAdmin { admin: String },
    /// Execute the receive phase 1 of the relay protocol. The packet is opaque and
    /// fully handled by the underlying implementation.
    ReceivePhase1(ReceivePhase1Msg),
}

/// This is the message we accept via Receive
#[cw_serde]
pub struct TransferMsg {
    /// The local channel to send the packets on
    pub channel: String,
    /// The remote address to send to.
    pub receiver: String,
    /// How long the packet lives in seconds. If not specified, use default_timeout
    pub timeout: Option<u64>,
    /// The memo
    pub memo: String,
}

#[cw_serde]
pub struct ReceivePhase1Msg {
    pub channel: String,
    pub raw_packet: Binary,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Return the port ID bound by this contract.
    #[returns(PortResponse)]
    Port {},
    /// Show all channels we have connected to.
    #[returns(ListChannelsResponse)]
    ListChannels {},
    /// Returns the details of the name channel, error if not created.
    #[returns(ChannelResponse)]
    Channel { id: String },
    /// Show the Config.
    #[returns(ConfigResponse)]
    Config {},
    #[returns(cw_controllers::AdminResponse)]
    Admin {},
}

#[cw_serde]
pub struct ListChannelsResponse {
    pub channels: Vec<ChannelInfo>,
}

#[cw_serde]
pub struct ChannelResponse {
    /// Information on the channel's connection
    pub info: ChannelInfo,
    /// How many tokens we currently have pending over this channel
    pub balances: Vec<(String, Uint512)>,
}

#[cw_serde]
pub struct PortResponse {
    pub port_id: String,
}

#[cw_serde]
pub struct ConfigResponse {
    pub default_timeout: u64,
    pub gov_contract: String,
}
