use crate::{cosmwasm_std::Addr, cw_storage_plus::Item};

pub const IBC_HOST: Item<Addr> = Item::new("ibc_host");
