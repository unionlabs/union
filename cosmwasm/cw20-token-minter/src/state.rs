use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub cw20_code_id: u64,
}

pub const CONFIG: Item<Config> = Item::new("conf");

pub const DENOM_TO_ADDR: Map<String, Addr> = Map::new("dta");

pub const ADDR_TO_DENOM: Map<Addr, String> = Map::new("atd");

pub const DENOM_TO_BE_STORED: Item<String> = Item::new("dtbs");
