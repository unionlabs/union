use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub number_of_block_before_pong_timeout: u64,
    pub revision_number: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
