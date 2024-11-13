use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const IBC_HOST: Item<Addr> = Item::new("ibc_host");
