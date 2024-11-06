use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use ibc_solidity::ibc::{Channel, Connection};

pub const CHANNEL_OWNER: Map<u32, Addr> = Map::new("channel_owner");

pub const CHANNELS: Map<u32, Channel> = Map::new("channels");

pub const CONNECTIONS: Map<u32, Connection> = Map::new("connections");

pub const CLIENT_STATES: Map<u32, Vec<u8>> = Map::new("client_states");

pub const CLIENT_CONSENSUS_STATES: Map<(u32, u64), Vec<u8>> = Map::new("client_consensus_states");

// From client type to contract implementation
pub const CLIENT_REGISTRY: Map<&str, Addr> = Map::new("client_registry");

// From client id to client type
pub const CLIENT_TYPES: Map<u32, String> = Map::new("client_types");

// From client id to contract implementation
pub const CLIENT_IMPLS: Map<u32, Addr> = Map::new("client_impls");

pub const NEXT_CLIENT_ID: Item<u32> = Item::new("next_client_id");

pub const NEXT_CONNECTION_ID: Item<u32> = Item::new("next_connection_id");

pub const NEXT_CHANNEL_ID: Item<u32> = Item::new("next_channel_id");
