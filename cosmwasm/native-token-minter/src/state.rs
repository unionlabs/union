use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const ADMIN: Item<Addr> = Item::new("admin");
