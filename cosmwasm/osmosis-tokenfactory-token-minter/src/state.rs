use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

/// Operator of the contract, who is the only one who can do state changing operations
pub const OPERATOR: Item<Addr> = Item::new("operator");

pub const ZKGM_ADDR: Item<Addr> = Item::new("zkgm_addr");

pub const TOKEN_OWNERS: Map<String, Addr> = Map::new("owners");
