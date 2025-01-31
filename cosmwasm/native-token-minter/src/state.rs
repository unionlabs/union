use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const ADMIN: Item<Addr> = Item::new("admin");

pub const WRAPPED_TOKEN_TO_DENOM: Map<String, String> = Map::new("wttd");
