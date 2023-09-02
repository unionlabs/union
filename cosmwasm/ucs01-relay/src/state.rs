use cosmwasm_schema::cw_serde;
use cosmwasm_std::{IbcEndpoint, Uint512};
use cw_controllers::Admin;
use cw_storage_plus::{Item, Map};

pub const ADMIN: Admin = Admin::new("admin");

pub const CONFIG: Item<Config> = Item::new("config");

/// static info on one channel that doesn't change
pub const CHANNEL_INFO: Map<&str, ChannelInfo> = Map::new("channel_info");

/// indexed by (channel_id, denom) maintaining the balance of the channel in that currency
pub const CHANNEL_STATE: Map<(&str, &str), ChannelState> = Map::new("channel_state");

pub const FOREIGN_TOKEN_CREATED: Map<&str, ()> = Map::new("foreign_tokens");

#[cw_serde]
#[derive(Default)]
pub struct ChannelState {
    pub outstanding: Uint512,
}

#[cw_serde]
pub struct Config {
    pub default_timeout: u64,
}

#[cw_serde]
pub struct ChannelInfo {
    pub endpoint: IbcEndpoint,
    /// the remote channel/port we connect to
    pub counterparty_endpoint: IbcEndpoint,
    /// the connection this exists on (you can use to query client/consensus info)
    pub connection_id: String,
    /// the protocol version, used to branch on the implementation
    pub protocol_version: String,
}
