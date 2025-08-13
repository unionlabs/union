use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub dummy_code_id: u64,
    #[serde(alias = "cw20_base_code_id")]
    pub cw20_impl_code_id: u64,
}

pub const CONFIG: Item<Config> = Item::new("conf");

pub const CW20_ADMIN: Item<Addr> = Item::new("admin");
