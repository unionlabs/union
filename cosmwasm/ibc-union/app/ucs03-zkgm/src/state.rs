use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint256};
use cw_storage_plus::{Item, Map};
use ibc_union_spec::types::Packet;
use unionlabs::primitives::Bytes;

#[cw_serde]
pub struct Config {
    pub ibc_host: Addr,
    pub token_minter: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const TOKEN_ORIGIN: Map<String, Uint256> = Map::new("token_origin");

pub const CHANNEL_BALANCE: Map<(u32, String), Uint256> = Map::new("channel_balance");

pub const EXECUTING_PACKET: Item<Packet> = Item::new("executing_packet");

pub const EXECUTION_ACK: Item<Bytes> = Item::new("execution_ack");

pub const HASH_TO_FOREIGN_TOKEN: Map<String, Bytes> = Map::new("hash_to_foreign_token");
