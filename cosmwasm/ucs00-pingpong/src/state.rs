use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub seconds_before_timeout: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
