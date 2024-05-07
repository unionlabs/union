use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const COMMITMENTS: Map<String, Vec<u8>> = Map::new("commitments");

pub const CLIENT_INDEX: Item<u64> = Item::new("client_index");

pub const CLIENTS: Map<String, Addr> = Map::new("clients");

// TODO(aeryz): limit key size
pub const CODE_IDS: Map<String, u64> = Map::new("code_ids");
