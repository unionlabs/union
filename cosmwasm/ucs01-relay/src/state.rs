use cosmwasm_schema::cw_serde;
use cosmwasm_std::{IbcEndpoint, Uint512};
use cw_controllers::Admin;
use cw_storage_plus::{Item, KeyDeserialize, Map, Prefixer, PrimaryKey};

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

#[derive(Debug, Clone)]
pub struct IbcEndpointKey(IbcEndpoint);

impl From<IbcEndpoint> for IbcEndpointKey {
    fn from(value: IbcEndpoint) -> Self {
        Self(value)
    }
}

impl<'a> Prefixer<'a> for IbcEndpointKey {
    fn prefix(&self) -> Vec<cw_storage_plus::Key> {
        let mut res = self.0.port_id.prefix();
        res.extend(self.0.channel_id.prefix());
        res
    }
}

impl KeyDeserialize for IbcEndpointKey {
    type Output = <(String, String) as KeyDeserialize>::Output;

    fn from_vec(value: Vec<u8>) -> cosmwasm_std::StdResult<Self::Output> {
        <(String, String) as KeyDeserialize>::from_vec(value)
    }

    // PrimaryKey is made up of 2 elements
    const KEY_ELEMS: u16 = 2;
}

impl<'a> PrimaryKey<'a> for IbcEndpointKey {
    type Prefix = <(String, String) as PrimaryKey<'a>>::Prefix;

    type SubPrefix = <(String, String) as PrimaryKey<'a>>::SubPrefix;

    type Suffix = <(String, String) as PrimaryKey<'a>>::Suffix;

    type SuperSuffix = <(String, String) as PrimaryKey<'a>>::SuperSuffix;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        let mut keys = self.0.port_id.key();
        keys.extend(self.0.channel_id.key());
        keys
    }
}

pub const FOREIGN_DENOM_TO_HASH: Map<(IbcEndpointKey, String), Hash> =
    Map::new("foreign_denom_to_hash");

pub const HASH_TO_FOREIGN_DENOM: Map<(IbcEndpointKey, Hash), String> =
    Map::new("hash_to_foreign_denom");

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
