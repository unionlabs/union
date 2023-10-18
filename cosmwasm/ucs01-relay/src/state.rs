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

// TokenFactory limitation
// MaxSubdenomLength = 44
// HASH_LENGTH = (MaxSubdenomLength - size_of("0x")) / 2 = 42
pub const HASH_LENGTH: usize = 21;

pub type Hash = [u8; HASH_LENGTH];

pub const FOREIGN_DENOM_TO_HASH: Map<String, Hash> = Map::new("foreign_denom_to_hash");

pub const HASH_TO_FOREIGN_DENOM: Map<Hash, String> = Map::new("hash_to_foreign_denom");

#[cw_serde]
#[derive(Default)]
pub struct ChannelState {
    pub in_flight: Uint512,
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
