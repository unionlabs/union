use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, CosmosMsg, IbcChannel, IbcEndpoint, Uint512};
use token_factory_api::TokenFactoryMsg;
use ucs01_relay_api::types::Fees;

use crate::state::ChannelInfo;

#[cw_serde]
pub struct InstantiateMsg {
    /// Default timeout for ics20 packets, specified in seconds
    pub default_timeout: u64,
    /// who can allow more contracts
    pub gov_contract: String,
    /// If set, contract will setup the channel
    pub channel: Option<IbcChannel>,
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    /// This allows us to transfer native tokens
    Transfer(TransferMsg),
    /// Register a denom, this message exist only to create sub-transaction from the top-level IBC call.
    RegisterDenom {
        local_endpoint: IbcEndpoint,
        denom: String,
        hash: Binary,
    },
    /// Change the admin (must be called by current admin)
    UpdateAdmin { admin: String },
    BatchExecute {
        msgs: Vec<CosmosMsg<TokenFactoryMsg>>,
    },
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
    /// Fee associated with the transfer, denominated in transferred coins
    pub fees: Option<Fees>,
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
    #[returns(String)]
    ForeignDenomToLocal {
        source_channel: String,
        denom: String,
    },
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
