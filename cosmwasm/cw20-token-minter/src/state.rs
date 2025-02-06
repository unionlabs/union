use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use unionlabs::primitives::H256;

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub dummy_code_id: u64,
    pub dummy_code_hash: H256,
    pub cw20_base_code_id: u64,
}

pub const CONFIG: Item<Config> = Item::new("conf");
