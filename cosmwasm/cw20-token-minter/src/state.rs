use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub cw20_base_code_id: u64,
}

pub const CONFIG: Item<Config> = Item::new("conf");

/// Goes from the denom identifier to the wrapped cw20 token address
pub const DENOM_TO_ADDR: Map<String, Addr> = Map::new("dta");

/// Goes from the wrapped cw20 token address to the denom identifier
pub const ADDR_TO_DENOM: Map<Addr, String> = Map::new("atd");

/// A temporary storage for the denom identifier. It will later be used
/// in the reply to store the address as well
pub const DENOM_TO_BE_STORED: Item<String> = Item::new("dtbs");
