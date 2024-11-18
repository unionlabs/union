use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub ibc_host: Addr,
    pub seconds_before_timeout: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
