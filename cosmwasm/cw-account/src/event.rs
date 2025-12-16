use cosmwasm_event::Event;
use cosmwasm_std::Addr;
use ibc_union_spec::ChannelId;
use unionlabs_primitives::{Bytes, U256, encoding::HexPrefixed};

use crate::types::Admin;

#[derive(Event)]
#[event("dispatch")]
pub struct Dispatch {
    pub admin: String,
}

#[derive(Event)]
#[event("set_zkgm")]
pub struct SetZkgm<'a> {
    pub zkgm: &'a Addr,
    pub admin: String,
}

#[derive(Event)]
#[event("add_admin")]
pub struct AddAdmin<'a> {
    pub new_admin: &'a Admin,
    pub admin: String,
}

#[derive(Event)]
#[event("remove_admin")]
pub struct RemoveAdmin<'a> {
    pub removed_admin: &'a Admin,
    pub admin: String,
}

#[derive(Event)]
#[event("remote_execute")]
pub struct RemoteExecute {
    pub sender: Bytes<HexPrefixed>,
    pub channel_id: ChannelId,
    pub path: U256,
}
